//!
//! Traits for working with pixmaps
//!
//! These traits model different capabilities which a pixmap canvas might implement.
//! All of them implement at least [`PixmapBase`] and should implement most of the others but
//! are not required to.
//!

use super::Color;
use anyhow::Result;

/// The common trait implemented by all pixmaps
pub trait PixmapBase {
    /// Get the size of this pixmap as (width, height) tuple
    fn get_size(&self) -> Result<(usize, usize)>;
}

/// A trait for *reading* single pixel data from a pixmap
pub trait PixmapRead: PixmapBase {
    /// Get the color value of the pixel at position (x,y)
    fn get_pixel(&self, x: usize, y: usize) -> Result<Color>;
}

/// A trait for *writing* single pixel data into a pixmap
pub trait PixmapWrite: PixmapBase {
    /// Set the pixel value at position (x,y) to the specified color
    fn set_pixel(&self, x: usize, y: usize, color: Color) -> Result<()>;
}

/// A trait for *reading* the complete pixmap data
pub trait PixmapRawRead: PixmapBase {
    /// Get all of the contained pixel data
    fn get_raw_data(&self) -> Result<Vec<Color>>;
}

/// A trait for overwriting the complete canvas of a pixmap
pub trait PixmapRawWrite: PixmapBase {
    /// Overwrite all of the contained pixel data.
    ///
    /// If the given *data* is too small, the remaining pixmap colors will be kept as they are.
    ///
    /// If the given *data* is too large, left over data will simply be ignored.
    fn put_raw_data(&self, data: &[Color]) -> Result<()>;
}
