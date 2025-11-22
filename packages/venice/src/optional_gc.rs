use std::{alloc::GlobalAlloc, cell::UnsafeCell};

use micropython_rs::gc::Gc;

pub struct OptionalGc {
    gc: UnsafeCell<Option<Gc>>,
}

impl OptionalGc {
    pub const fn new(gc: Option<Gc>) -> Self {
        Self {
            gc: UnsafeCell::new(gc),
        }
    }

    pub fn gc(&self) -> Option<&Gc> {
        unsafe { &*self.gc.get() }.as_ref()
    }

    /// # Safety
    ///
    /// Caller must ensure that no active [`Gc`] references exist
    pub const unsafe fn set_gc(&self, gc: Option<Gc>) {
        unsafe { *self.gc.get() = gc }
    }
}

unsafe impl Sync for OptionalGc {}

unsafe impl GlobalAlloc for OptionalGc {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        unsafe { self.gc().expect("gc not initialized").alloc(layout) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        unsafe { self.gc().expect("gc not initialized").dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: std::alloc::Layout, new_size: usize) -> *mut u8 {
        unsafe {
            self.gc()
                .expect("gc not initialized")
                .realloc(ptr, layout, new_size)
        }
    }
}
