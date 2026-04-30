// Copyright (c) 2017-2025, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

//! Pixel data type abstractions.
//!
//! This module defines the [`Pixel`] trait, which abstracts over the pixel data types
//! used throughout the library. This allows the same code to work with both 8-bit
//! (`u8`) and high bit-depth (`u16`) pixel data.
//!
//! # Supported Pixel Types
//!
//! - `u8`: For 8-bit pixel data
//! - `u16`: For 9-16 bit pixel data (high bit-depth)
//!
//! The type used must match the bit depth specified when creating frames:
//! - 8-bit frames must use `u8`
//! - 9-16 bit frames must use `u16`

use std::fmt::Debug;

mod private {
    pub trait Sealed {}

    impl Sealed for u8 {}
    impl Sealed for u16 {}
}

/// A trait for types that can be used as pixel data.
///
/// This trait abstracts over the pixel data types supported by the library,
/// currently `u8` for 8-bit data and `u16` for high bit-depth (9-16 bit) data.
///
/// All frame and plane types are generic over `T: Pixel`, allowing the same
/// data structures and algorithms to work with both standard and high bit-depth
/// video content.
///
/// # Type Safety
///
/// The library enforces correct type usage through validation:
/// - Frames with 8-bit depth can only be created with `T = u8`
/// - Frames with 9-16 bit depth can only be created with `T = u16`
///
/// Attempting to create a frame with a mismatched type will result in
/// [`FrameError::DataTypeMismatch`][crate::frame::FrameError::DataTypeMismatch].
///
/// # Safety
///
/// All implementing types must be valid if represented by an all-zero byte-pattern,
/// i.e. using [`std::mem::zeroed`] must __not__ cause undefined behavior for
/// implementing types.
pub unsafe trait Pixel:
    Debug + Copy + Clone + Default + Send + Sync + 'static + private::Sealed
{
    /// Lossless and cheap conversion from a `u8` into `Self`.
    fn from_u8(x: u8) -> Self;

    /// Conversion from a `u16` into `Self`.
    ///
    /// Returns `None` if the implementing type cannot losslessly
    /// represent the value of `x`.
    fn try_from_u16(x: u16) -> Option<Self>;

    /// Conversion from any integer type into `Self`.
    ///
    /// Returns `None` if the implementing type cannot losslessly
    /// represent the value of `x`.
    #[inline]
    fn try_from_int<T: TryInto<u16>>(x: T) -> Option<Self> {
        x.try_into().ok().and_then(Self::try_from_u16)
    }

    /// Lossless and cheap conversion into a `u16`.
    fn as_u16(self) -> u16;

    /// Lossless and cheap conversion into a `u32`.
    #[inline]
    fn as_u32(self) -> u32 {
        u32::from(self.as_u16())
    }

    /// Lossless and cheap conversion into a `u64`.
    #[inline]
    fn as_u64(self) -> u64 {
        u64::from(self.as_u16())
    }

    /// Lossless and cheap conversion into a `u128`.
    #[inline]
    fn as_u128(self) -> u128 {
        u128::from(self.as_u16())
    }

    /// Lossless and cheap conversion into a `i32`.
    #[inline]
    fn as_i32(self) -> i32 {
        i32::from(self.as_u16())
    }

    /// Lossless and cheap conversion into a `i64`.
    #[inline]
    fn as_i64(self) -> i64 {
        i64::from(self.as_u16())
    }

    /// Lossless and cheap conversion into a `i128`.
    #[inline]
    fn as_i128(self) -> i128 {
        i128::from(self.as_u16())
    }
}

/// Pixel implementation for 8-bit video data.
// SAFETY: u8 is valid if represented by a zeroed byte.
unsafe impl Pixel for u8 {
    #[inline]
    fn from_u8(x: u8) -> Self {
        x
    }

    #[inline]
    fn try_from_u16(x: u16) -> Option<Self> {
        u8::try_from(x).ok().map(Self::from_u8)
    }

    #[inline]
    fn as_u16(self) -> u16 {
        u16::from(self)
    }
}

/// Pixel implementation for high bit-depth (9-16 bit) video data.
// SAFETY: u16 is valid if represented by zeroed bytes.
unsafe impl Pixel for u16 {
    #[inline]
    fn from_u8(x: u8) -> Self {
        Self::from(x)
    }

    #[inline]
    fn try_from_u16(x: u16) -> Option<Self> {
        Some(x)
    }

    #[inline]
    fn as_u16(self) -> u16 {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u8() {
        for v in 0..=u8::MAX {
            assert_eq!(v, u8::try_from(v.as_u16()).expect("u16 fits into u8"));
            assert_eq!(v, u8::try_from(v.as_u32()).expect("u32 fits into u8"));
            assert_eq!(v, u8::try_from(v.as_u64()).expect("u64 fits into u8"));
            assert_eq!(v, u8::try_from(v.as_u64()).expect("u64 fits into u8"));
            assert_eq!(v, u8::try_from(v.as_u128()).expect("u128 fits into u8"));

            assert_eq!(v, u8::try_from(v.as_i32()).expect("i32 fits into u8"));
            assert_eq!(v, u8::try_from(v.as_i64()).expect("i64 fits into u8"));
            assert_eq!(v, u8::try_from(v.as_i64()).expect("i64 fits into u8"));
            assert_eq!(v, u8::try_from(v.as_i128()).expect("i128 fits into u8"));
        }
    }

    #[test]
    fn u16() {
        for v in 0..=u16::MAX {
            assert_eq!(v, v.as_u16());
            assert_eq!(v, u16::try_from(v.as_u32()).expect("u32 fits into u16"));
            assert_eq!(v, u16::try_from(v.as_u64()).expect("u64 fits into u16"));
            assert_eq!(v, u16::try_from(v.as_u64()).expect("u64 fits into u16"));
            assert_eq!(v, u16::try_from(v.as_u128()).expect("u128 fits into u16"));

            assert_eq!(v, u16::try_from(v.as_i32()).expect("i32 fits into u16"));
            assert_eq!(v, u16::try_from(v.as_i64()).expect("i64 fits into u16"));
            assert_eq!(v, u16::try_from(v.as_i64()).expect("i64 fits into u16"));
            assert_eq!(v, u16::try_from(v.as_i128()).expect("i128 fits into u16"));
        }
    }
}
