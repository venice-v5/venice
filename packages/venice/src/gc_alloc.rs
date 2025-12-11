use std::{
    alloc::{GlobalAlloc, Layout},
    cmp::Ordering,
    ptr::{NonNull, null_mut},
    sync::{Mutex, OnceLock},
};

use micropython_rs::{
    gc::{self, dealloc},
    init::InitToken,
};
use talc::{ErrOnOom, Talc};

pub struct GcAlloc {
    token: OnceLock<InitToken>,
    // GC lock is used to synchronize this
    fallback_alloc: Mutex<Talc<ErrOnOom>>,
}

impl GcAlloc {
    pub const fn new() -> Self {
        Self {
            token: OnceLock::new(),
            fallback_alloc: Mutex::new(Talc::new(ErrOnOom)),
        }
    }

    pub fn init(&self, token: InitToken) -> Result<(), InitToken> {
        self.token.set(token)
    }

    pub fn fallback_alloc(&self) -> &Mutex<Talc<ErrOnOom>> {
        &self.fallback_alloc
    }

    pub fn collect_garbage(&self) -> Result<(), ()> {
        self.token
            .get()
            .map(|token| gc::collect_garbage(*token))
            .ok_or(())
    }

    // stolen from talc source code
    unsafe fn fallback_realloc(
        &self,
        ptr: *mut u8,
        old_layout: Layout,
        new_size: usize,
    ) -> *mut u8 {
        unsafe {
            let nn_ptr = NonNull::new_unchecked(ptr);

            match new_size.cmp(&old_layout.size()) {
                Ordering::Greater => {
                    // first try to grow in-place before manually re-allocating

                    if let Ok(nn) = self
                        .fallback_alloc
                        .lock()
                        .unwrap()
                        .grow_in_place(nn_ptr, old_layout, new_size)
                    {
                        return nn.as_ptr();
                    }

                    // grow in-place failed, reallocate manually

                    let new_layout =
                        Layout::from_size_align_unchecked(new_size, old_layout.align());

                    let mut lock = self.fallback_alloc.lock().unwrap();
                    let allocation = match lock.malloc(new_layout) {
                        Ok(ptr) => ptr,
                        Err(_) => return null_mut(),
                    };

                    const RELEASE_LOCK_ON_REALLOC_LIMIT: usize = 0x10000;
                    if old_layout.size() > RELEASE_LOCK_ON_REALLOC_LIMIT {
                        drop(lock);
                        allocation
                            .as_ptr()
                            .copy_from_nonoverlapping(ptr, old_layout.size());
                        lock = self.fallback_alloc.lock().unwrap();
                    } else {
                        allocation
                            .as_ptr()
                            .copy_from_nonoverlapping(ptr, old_layout.size());
                    }

                    lock.free(nn_ptr, old_layout);
                    allocation.as_ptr()
                }

                Ordering::Less => {
                    self.fallback_alloc.lock().unwrap().shrink(
                        NonNull::new_unchecked(ptr),
                        old_layout,
                        new_size,
                    );
                    ptr
                }

                Ordering::Equal => ptr,
            }
        }
    }
}

unsafe impl GlobalAlloc for GcAlloc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        unsafe {
            if layout.align() < 4 {
                let token = match self.token.get() {
                    Some(token) => token,
                    None => return null_mut(),
                };

                gc::alloc(*token, layout.size(), false)
            } else {
                self.fallback_alloc
                    .lock()
                    .unwrap()
                    .malloc(layout)
                    .map_or(null_mut(), |nn| nn.as_ptr())
            }
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        unsafe {
            if layout.align() < 4 {
                let token = match self.token.get() {
                    Some(token) => token,
                    // fix: silent fail
                    None => return,
                };

                dealloc(*token, ptr);
            } else {
                self.fallback_alloc
                    .lock()
                    .unwrap()
                    .free(NonNull::new_unchecked(ptr), layout);
            }
        }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
        unsafe {
            if layout.align() < 4 {
                let token = match self.token.get() {
                    Some(token) => token,
                    None => return null_mut(),
                };

                gc::realloc(*token, ptr, new_size)
            } else {
                self.fallback_realloc(ptr, layout, new_size)
            }
        }
    }
}
