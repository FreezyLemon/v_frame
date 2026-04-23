// Copyright (c) 2018-2025, The rav1e contributors. All rights reserved
//
// This source code is subject to the terms of the BSD 2 Clause License and
// the Alliance for Open Media Patent License 1.0. If the BSD 2 Clause License
// was not distributed with this source code in the LICENSE file, you can
// obtain it at www.aomedia.org/license/software. If the Alliance for Open
// Media Patent License 1.0 was not distributed with this source code in the
// PATENTS file, you can obtain it at www.aomedia.org/license/patent.

//! YUV video frame structures and builders.
//!
//! This module provides the [`Frame`] type, which represents a complete YUV video frame
//! consisting of one luma (Y) plane and optionally two chroma (U and V) planes. Frames
//! are constructed using the [`FrameBuilder`] pattern to ensure type safety and correct
//! configuration.
//!
//! # Frame Structure
//!
//! A YUV frame contains:
//! - **Y plane**: Luma (brightness) information, always present
//! - **U plane**: First chroma component (Cb), present unless monochrome
//! - **V plane**: Second chroma component (Cr), present unless monochrome
//!
//! The relative dimensions of the chroma planes are determined by the
//! [`ChromaSubsampling`](crate::chroma::ChromaSubsampling) format.
//!
//! # Type Safety
//!
//! Frames are generic over the pixel type `T: Pixel`:
//! - Use `Frame<u8>` for 8-bit video
//! - Use `Frame<u16>` for high bit-depth (9-16 bit) video
//!
//! The builder validates that the pixel type matches the specified bit depth,
//! returning [`Error::DataTypeMismatch`](crate::error::Error::DataTypeMismatch) if they
//! don't align.
//!
//! # Padding
//!
//! Frames support optional padding around the luma plane, which is automatically
//! propagated to the chroma planes according to the subsampling ratio. Padding is
//! useful for video codec algorithms that need to access pixels beyond the visible
//! frame boundaries.
//!
//! # Example
//!
//! ```rust
//! use v_frame::frame::FrameBuilder;
//! use v_frame::chroma::ChromaSubsampling;
//! use std::num::{NonZeroU8, NonZeroUsize};
//!
//! // Create a 1920x1080 YUV420 8-bit frame
//! let width = NonZeroUsize::new(1920).unwrap();
//! let height = NonZeroUsize::new(1080).unwrap();
//! let bit_depth = NonZeroU8::new(8).unwrap();
//!
//! let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
//!     .build::<u8>()
//!     .unwrap();
//!
//! // Access the planes
//! assert_eq!(frame.y_plane.width().get(), 1920);
//! assert_eq!(frame.y_plane.height().get(), 1080);
//!
//! // Chroma planes are half size for YUV420
//! let u_plane = frame.u_plane.as_ref().unwrap();
//! assert_eq!(u_plane.width().get(), 960);
//! assert_eq!(u_plane.height().get(), 540);
//! ```
//!
//! # Creating Frames with Padding
//!
//! ```rust
//! use v_frame::frame::FrameBuilder;
//! use v_frame::chroma::ChromaSubsampling;
//! use std::num::{NonZeroU8, NonZeroUsize};
//!
//! let width = NonZeroUsize::new(1920).unwrap();
//! let height = NonZeroUsize::new(1080).unwrap();
//! let bit_depth = NonZeroU8::new(10).unwrap();
//!
//! let frame = FrameBuilder::new(width, height, ChromaSubsampling::Yuv420, bit_depth)
//! .luma_padding_left(16)
//! .luma_padding_right(16)
//! .luma_padding_top(16)
//! .luma_padding_bottom(16)
//! .build::<u16>().unwrap();
//! ```

mod builder;
pub use builder::{BuildError, FrameBuilder};

#[cfg(test)]
mod tests;

use std::num::NonZeroU8;

use crate::{chroma::ChromaSubsampling, pixel::Pixel, plane::Plane};

/// Contains the data representing one YUV video frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame<T: Pixel> {
    /// The luma plane for this frame
    pub y_plane: Plane<T>,
    /// The first chroma plane for this frame, or `None` if this is a grayscale frame
    pub u_plane: Option<Plane<T>>,
    /// The second chroma plane for this frame, or `None` if this is a grayscale frame
    pub v_plane: Option<Plane<T>>,
    /// The chroma subsampling for this frame
    pub subsampling: ChromaSubsampling,
    /// The number of bits per pixel in this frame
    pub bit_depth: NonZeroU8,
}

impl<T: Pixel> Frame<T> {
    /// Returns a reference to the plane at the given 0-based index,
    /// if it exists in the frame. Otherwise, returns `None`.
    #[inline]
    #[must_use]
    pub fn plane(&self, index: usize) -> Option<&Plane<T>> {
        match index {
            0 => Some(&self.y_plane),
            1 => self.u_plane.as_ref(),
            2 => self.v_plane.as_ref(),
            _ => None,
        }
    }

    /// Returns a mutable reference to the plane at the given 0-based index,
    /// if it exists in the frame. Otherwise, returns `None`.
    #[inline]
    #[must_use]
    pub fn plane_mut(&mut self, index: usize) -> Option<&mut Plane<T>> {
        match index {
            0 => Some(&mut self.y_plane),
            1 => self.u_plane.as_mut(),
            2 => self.v_plane.as_mut(),
            _ => None,
        }
    }
}
