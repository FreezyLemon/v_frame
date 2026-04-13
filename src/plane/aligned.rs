use std::alloc::{Layout, alloc, alloc_zeroed, dealloc, handle_alloc_error};
use std::marker::PhantomData;
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

use crate::pixel::Pixel;

/// Alignment for plane data on WASM platforms (8 bytes).
#[cfg(target_arch = "wasm32")]
const DATA_ALIGNMENT: usize = 1 << 3;

/// Alignment for plane data on non-WASM platforms (64 bytes for SIMD optimization).
#[cfg(not(target_arch = "wasm32"))]
const DATA_ALIGNMENT: usize = 1 << 6;

pub struct AlignedData<T> {
    ptr: NonNull<T>,
    // len == capacity, no resizing allowed.
    // if len == 0 we don't allocate (and don't deallocate)
    len: usize,
    _marker: PhantomData<T>,
}

impl<T> AlignedData<T> {
    // if None: don't alloc/dealloc
    const fn layout(len: usize) -> Option<Layout> {
        const {
            assert!(DATA_ALIGNMENT.is_power_of_two());
            assert!(size_of::<T>() > 0);
        }

        if len == 0 {
            return None;
        }

        let layout_size = size_of::<T>() * len;
        if let Ok(l) = Layout::from_size_align(layout_size, DATA_ALIGNMENT) {
            Some(l)
        } else {
            panic!("invalid layout")
        }
    }

    pub fn new_uninit(len: usize) -> AlignedData<MaybeUninit<T>> {
        let Some(layout) = Self::layout(len) else {
            return AlignedData {
                ptr: NonNull::dangling(),
                len: 0,
                _marker: PhantomData,
            };
        };

        // SAFETY: `Self::layout` guarantees that the layout is valid and has nonzero size.
        let ptr = unsafe { alloc(layout) as *mut MaybeUninit<T> };
        let Some(ptr) = NonNull::new(ptr) else {
            handle_alloc_error(layout);
        };

        AlignedData {
            ptr,
            len,
            _marker: PhantomData,
        }
    }
}

impl<T> AlignedData<MaybeUninit<T>> {
    /// Converts to [`AlignedData<T>`].
    ///
    /// # Safety
    /// It is up to the caller to ensure that all contained values are
    /// initialized properly (see [`MaybeUninit::assume_init`]).
    pub unsafe fn assume_init(self) -> AlignedData<T> {
        // The underlying memory would usually be deallocated when `Drop`
        // is run at the end of this scope. It needs to stay valid, so
        // inhibit the destructor here.
        let this = ManuallyDrop::new(self);

        AlignedData {
            ptr: this.ptr.cast(),
            len: this.len,
            _marker: PhantomData,
        }
    }
}

impl<T: Pixel> AlignedData<T> {
    /// Zeroed.
    pub fn new(len: usize) -> Self {
        let Some(layout) = Self::layout(len) else {
            return AlignedData {
                ptr: NonNull::dangling(),
                len: 0,
                _marker: PhantomData,
            };
        };

        // SAFETY: `Self::layout` guarantees that the layout is valid and has nonzero size.
        // SAFETY: The Pixel trait guarantees that zeroed memory is a valid T.
        let ptr = unsafe { alloc_zeroed(layout) as *mut T };
        let Some(ptr) = NonNull::new(ptr) else {
            handle_alloc_error(layout);
        };

        Self {
            ptr,
            len,
            _marker: PhantomData,
        }
    }
}

impl<T> Deref for AlignedData<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        // SAFETY:
        // - `self.ptr` is non-null and valid for `len` reads of `T`
        // - `self.ptr` + `len` describe a single allocation
        // - `self.ptr` is properly aligned (allocated with a valid Layout)
        // - all values of T are properly initialized, either via zeroing
        //   or manually before calling `Self::assume_init`
        // - immutable borrow is upheld by `&self`
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }
}

impl<T> DerefMut for AlignedData<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: See `deref` above. Additionally:
        // - `self.ptr` is valid for `len` writes of `T`
        // - mutable borrow is upheld by `&mut self`
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl<T: Clone> Clone for AlignedData<T> {
    fn clone(&self) -> Self {
        let mut new = Self::new_uninit(self.len);

        assert_eq!(
            self.len, new.len,
            "data length must be equal to clone safely"
        );

        for (new, old) in new.iter_mut().zip(self.iter()) {
            new.write(old.clone());
        }

        // SAFETY:
        // All values are properly initialized in the loop above.
        unsafe { new.assume_init() }
    }
}

impl<T> Drop for AlignedData<T> {
    fn drop(&mut self) {
        if let Some(layout) = Self::layout(self.len) {
            // SAFETY:
            // - `ptr` was allocated via this (global) allocator
            // - `layout` is equal to the one used for allocation (returned from
            //   Self::layout for the same parameter `len`)
            unsafe {
                dealloc(self.ptr.as_ptr() as _, layout);
            }
        }
    }
}
