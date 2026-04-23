use std::num::{NonZeroU8, NonZeroUsize};

use super::Frame;
use crate::{
    chroma::ChromaSubsampling,
    pixel::Pixel,
    plane::{Plane, PlaneGeometry},
};

/// A builder for constructing [`Frame`] instances with validation.
///
/// `FrameBuilder` uses the builder pattern to construct frames safely, validating
/// that all parameters are compatible (bit depth matches pixel type, dimensions are
/// compatible with chroma subsampling, padding is properly aligned, etc.).
///
/// # Required Parameters
///
/// The following parameters must be provided when creating a new builder:
/// - `width`: Frame width in pixels
/// - `height`: Frame height in pixels
/// - `subsampling`: Chroma subsampling format
/// - `bit_depth`: Bit depth (8 for `u8` pixels, 9-16 for `u16` pixels)
///
/// # Optional Parameters
///
/// Luma padding can be set via setter methods. When padding is set, it is automatically
/// propagated to the chroma planes according to the subsampling ratio.
///
/// # Example
///
/// ```rust
/// use v_frame::frame::FrameBuilder;
/// use v_frame::chroma::ChromaSubsampling;
/// use std::num::{NonZeroU8, NonZeroUsize};
///
/// let frame = FrameBuilder::new(
///     NonZeroUsize::new(1920).unwrap(),
///     NonZeroUsize::new(1080).unwrap(),
///     ChromaSubsampling::Yuv420,
///     NonZeroU8::new(8).unwrap(),
/// )
/// .luma_padding_left(8)
/// .luma_padding_right(8)
/// .build::<u8>().unwrap();
/// ```
pub struct FrameBuilder {
    /// Visible width in pixels.
    width: NonZeroUsize,
    /// Visible height in pixels.
    height: NonZeroUsize,
    /// Chroma subsampling format.
    subsampling: ChromaSubsampling,
    /// Bit depth of the frame's pixels (8-16).
    bit_depth: NonZeroU8,
    /// Number of padding pixels on the left of the luma plane.
    luma_padding_left: usize,
    /// Number of padding pixels on the right of the luma plane.
    luma_padding_right: usize,
    /// Number of padding pixels on the top of the luma plane.
    luma_padding_top: usize,
    /// Number of padding pixels on the bottom of the luma plane.
    luma_padding_bottom: usize,
}

impl FrameBuilder {
    /// Creates a new frame builder, taking the parameters that are required for all frames.
    /// The builder then allows for setting additional, optional parameters.
    #[inline]
    #[must_use]
    pub fn new(
        width: NonZeroUsize,
        height: NonZeroUsize,
        subsampling: ChromaSubsampling,
        bit_depth: NonZeroU8,
    ) -> Self {
        Self {
            width,
            height,
            subsampling,
            bit_depth,
            luma_padding_left: 0,
            luma_padding_right: 0,
            luma_padding_top: 0,
            luma_padding_bottom: 0,
        }
    }

    /// Set the `luma_padding_left` for the frame builder.
    #[inline]
    #[must_use]
    pub fn luma_padding_left(mut self, luma_padding_left: usize) -> Self {
        self.luma_padding_left = luma_padding_left;
        self
    }

    /// Set the `luma_padding_right` for the frame builder.
    #[inline]
    #[must_use]
    pub fn luma_padding_right(mut self, luma_padding_right: usize) -> Self {
        self.luma_padding_right = luma_padding_right;
        self
    }

    /// Set the `luma_padding_top` for the frame builder.
    #[inline]
    #[must_use]
    pub fn luma_padding_top(mut self, luma_padding_top: usize) -> Self {
        self.luma_padding_top = luma_padding_top;
        self
    }

    /// Set the `luma_padding_bottom` for the frame builder.
    #[inline]
    #[must_use]
    pub fn luma_padding_bottom(mut self, luma_padding_bottom: usize) -> Self {
        self.luma_padding_bottom = luma_padding_bottom;
        self
    }

    /// Constructs a `Frame` from the current builder.
    ///
    /// # Errors
    /// - Returns `BuildError::UnsupportedBitDepth` if the input bit depth is unsupported
    ///   (currently 8-16 bit inputs are supported)
    /// - Returns `BuildError::DataTypeMismatch` if the size of `T` does not match the input bit depth
    /// - Returns `BuildError::UnsupportedResolution` if the resolution or padding dimensions
    ///   do not support the requested subsampling
    #[inline]
    pub fn build<T: Pixel>(self) -> Result<Frame<T>, BuildError> {
        if self.bit_depth.get() < 8 || self.bit_depth.get() > 16 {
            return Err(BuildError::UnsupportedBitDepth {
                found: self.bit_depth.get(),
            });
        }

        let byte_width = size_of::<T>();
        assert!(
            byte_width <= 2,
            "unsupported pixel byte width: {byte_width}"
        );
        if (byte_width == 1 && self.bit_depth.get() != 8)
            || (byte_width == 2 && self.bit_depth.get() <= 8)
        {
            return Err(BuildError::DataTypeMismatch);
        }

        let luma_stride = self
            .width
            .saturating_add(self.luma_padding_left)
            .saturating_add(self.luma_padding_right);
        let luma_geometry = PlaneGeometry {
            width: self.width,
            height: self.height,
            stride: luma_stride,
            pad_left: self.luma_padding_left,
            pad_right: self.luma_padding_right,
            pad_top: self.luma_padding_top,
            pad_bottom: self.luma_padding_bottom,
            subsampling_x: NonZeroU8::new(1).expect("non-zero constant"),
            subsampling_y: NonZeroU8::new(1).expect("non-zero constant"),
        };
        if !self.subsampling.has_chroma() {
            return Ok(Frame {
                y_plane: Plane::new(luma_geometry),
                u_plane: None,
                v_plane: None,
                subsampling: self.subsampling,
                bit_depth: self.bit_depth,
            });
        }

        let Some((chroma_width, chroma_height)) = self
            .subsampling
            .chroma_dimensions(self.width.get(), self.height.get())
        else {
            return Err(BuildError::UnsupportedResolution);
        };

        let (ss_x, ss_y) = self.subsampling.subsample_ratio().expect("not monochrome");
        if self.luma_padding_left % ss_x.get() as usize > 0
            || self.luma_padding_right % ss_x.get() as usize > 0
            || self.luma_padding_top % ss_y.get() as usize > 0
            || self.luma_padding_bottom % ss_y.get() as usize > 0
        {
            return Err(BuildError::UnsupportedResolution);
        }
        let chroma_padding_left = self.luma_padding_left / ss_x.get() as usize;
        let chroma_padding_right = self.luma_padding_right / ss_x.get() as usize;
        let chroma_padding_top = self.luma_padding_top / ss_y.get() as usize;
        let chroma_padding_bottom = self.luma_padding_bottom / ss_y.get() as usize;
        let chroma_stride = chroma_width
            .saturating_add(chroma_padding_left)
            .saturating_add(chroma_padding_right);

        let chroma_geometry = PlaneGeometry {
            width: NonZeroUsize::new(chroma_width).expect("cannot be zero"),
            height: NonZeroUsize::new(chroma_height).expect("cannot be zero"),
            stride: NonZeroUsize::new(chroma_stride).expect("cannot be zero"),
            pad_left: chroma_padding_left,
            pad_right: chroma_padding_right,
            pad_top: chroma_padding_top,
            pad_bottom: chroma_padding_bottom,
            subsampling_x: ss_x,
            subsampling_y: ss_y,
        };
        Ok(Frame {
            y_plane: Plane::new(luma_geometry),
            u_plane: Some(Plane::new(chroma_geometry)),
            v_plane: Some(Plane::new(chroma_geometry)),
            subsampling: self.subsampling,
            bit_depth: self.bit_depth,
        })
    }
}

use std::fmt;

/// This enum represents possible error conditions that can occur when
/// creating a new [`Frame`][crate::frame::Frame].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuildError {
    /// The supplied bit depth is unsupported.
    ///
    /// The library only supports bit depths from 8 to 16 bits inclusive.
    UnsupportedBitDepth {
        /// The requested bit depth which triggered the error
        found: u8,
    },

    /// The pixel data type does not match the specified bit depth.
    ///
    /// 8-bit frames must use `u8`, while 9-16 bit frames must use `u16`.
    DataTypeMismatch,

    /// Frame dimensions are incompatible with the supplied chroma subsampling format.
    ///
    /// For example, YUV420 requires even width and height, while YUV422 requires even width.
    UnsupportedResolution,
}

impl fmt::Display for BuildError {
    #[expect(clippy::missing_inline_in_public_items)]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::UnsupportedBitDepth { found } => write!(
                f,
                "only 8-16 bit frame data is supported, tried to create {found} bit frame"
            ),
            BuildError::DataTypeMismatch => {
                write!(f, "bit depth did not match requested data type")
            }
            BuildError::UnsupportedResolution => write!(
                f,
                "selected chroma subsampling does not support odd resolutions"
            ),
        }
    }
}

impl std::error::Error for BuildError {}
