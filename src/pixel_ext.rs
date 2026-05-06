//! Extensions for the [`Pixel`] trait.
//! 
//! Currently only the [`PixelExt`] trait is provided, but more might be
//! added in the future.

use crate::pixel::Pixel;

/// Implements additional traits on `Pixel` types for performance-sensitive applications.
pub trait PixelExt {
    /// Assume the reference is a reference to a `u8`.
    /// 
    /// # Safety
    /// 
    /// Callers must ensure that the implementing type is a `u8`, for example by
    /// checking the size of the type (via `size_of::<T>() == 1`).
    unsafe fn assume_u8(&self) -> &u8;

    /// Assume the reference is a reference to a `u16`.
    /// 
    /// # Safety
    /// 
    /// Callers must ensure that the implementing type is a `u16`, for example by
    /// checking the size of the type (via `size_of::<T>() == 2`).
    unsafe fn assume_u16(&self) -> &u16;
}

impl<T: Pixel> PixelExt for T {
    unsafe fn assume_u8(&self) -> &u8 {
        unsafe { std::ptr::from_ref(self).cast::<u8>().as_ref_unchecked() }
    }

    unsafe fn assume_u16(&self) -> &u16 {
        unsafe { std::ptr::from_ref(self).cast::<u16>().as_ref_unchecked() }
    }
}
