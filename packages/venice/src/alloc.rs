use std::{
    alloc::{AllocError, Allocator, Layout},
    ptr::NonNull,
};

use micropython_rs::{
    gc::{self},
    init::InitToken,
};
use talc::{ErrOnOom, Talc, Talck};

#[global_allocator]
pub static ALLOCATOR: Talck<spin::Mutex<()>, ErrOnOom> = Talck::new(Talc::new(ErrOnOom));

#[derive(Debug, Clone, Copy)]
pub struct Gc {
    pub token: InitToken,
}

// this may actually be 32 instead of 4, because gc blocks are 32 bytes wide
// TODO: check by allocating and checking the maximum pointer alignment
pub const GC_MAX_ALIGN: usize = 4;

impl Gc {
    unsafe fn realloc(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        assert!(
            old_layout.align() <= GC_MAX_ALIGN,
            "attempt to reallocate impossible allocation (align > 4)"
        );

        if new_layout.align() < GC_MAX_ALIGN {
            let ptr = unsafe { gc::realloc(self.token, ptr.as_ptr(), new_layout.size()) };
            match NonNull::new(ptr) {
                Some(nn) => Ok(NonNull::slice_from_raw_parts(nn, new_layout.size())),
                None => Err(AllocError),
            }
        } else {
            Err(AllocError)
        }
    }
}

unsafe impl Allocator for Gc {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, std::alloc::AllocError> {
        if layout.align() <= GC_MAX_ALIGN {
            let ptr = unsafe { gc::alloc(self.token, layout.size(), false) };
            match NonNull::new(ptr) {
                Some(nn) => Ok(NonNull::slice_from_raw_parts(nn, layout.size())),
                None => Err(AllocError),
            }
        } else {
            Err(AllocError)
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        assert!(
            layout.align() <= GC_MAX_ALIGN,
            "attempt to deallocate impossible allocation (align > 4)"
        );
        unsafe { gc::dealloc(self.token, ptr.as_ptr()) }
    }

    unsafe fn grow(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { self.realloc(ptr, old_layout, new_layout) }
    }

    unsafe fn shrink(
        &self,
        ptr: NonNull<u8>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> Result<NonNull<[u8]>, AllocError> {
        unsafe { self.realloc(ptr, old_layout, new_layout) }
    }
}
