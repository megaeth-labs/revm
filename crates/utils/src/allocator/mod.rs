//! This module is used to support memory size measurement for Vec and hashbrown:: HashMap.
//! It provides a memory allocator to record the allocated memory size, and also provides
//! a separate Vec type for specifying the memory allocator during creation.
use allocator_api2::alloc::{AllocError, Allocator};
pub use allocator_api2::vec::Vec;
use std::{
    alloc::Layout,
    sync::atomic::{AtomicUsize, Ordering::SeqCst},
};
use std::{
    alloc::{GlobalAlloc, System},
    ptr::NonNull,
    slice,
};

static ALLOC: AtomicUsize = AtomicUsize::new(0);
static DEALLOC: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug, Copy, Clone, Default)]
pub struct TrackingAllocator;

pub fn reset() {
    ALLOC.store(0, SeqCst);
    DEALLOC.store(0, SeqCst);
}

pub fn record_alloc(layout: Layout) {
    ALLOC.fetch_add(layout.size(), SeqCst);
}

pub fn record_dealloc(layout: Layout) {
    DEALLOC.fetch_add(layout.size(), SeqCst);
}

pub struct Stats {
    pub alloc: usize,
    pub dealloc: usize,
    pub diff: isize,
}

pub fn stats() -> Stats {
    let alloc: usize = ALLOC.load(SeqCst);
    let dealloc: usize = DEALLOC.load(SeqCst);
    let diff = (alloc as isize) - (dealloc as isize);

    Stats {
        alloc,
        dealloc,
        diff,
    }
}

unsafe impl Allocator for TrackingAllocator {
    fn allocate(&self, layout: Layout) -> Result<NonNull<[u8]>, AllocError> {
        unsafe {
            let ptr = System.alloc(layout);
            if ptr.is_null() {
                Err(AllocError)
            } else {
                let slice_ptr: *mut [u8] = slice::from_raw_parts_mut(ptr, layout.size());
                let non_null_slice: NonNull<[u8]> = NonNull::new_unchecked(slice_ptr);
                record_alloc(layout);

                Ok(non_null_slice)
            }
        }
    }

    unsafe fn deallocate(&self, ptr: NonNull<u8>, layout: Layout) {
        record_dealloc(layout);
        let raw_ptr: *mut u8 = ptr.as_ptr();
        System.dealloc(raw_ptr, layout);
    }
}
