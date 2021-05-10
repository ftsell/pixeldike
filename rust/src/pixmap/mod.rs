//!
//! Data structures to store pixel data, also called *Pixmaps*
//!

mod color;
mod file_backed_pixmap;
mod in_memory_pixmap;
mod remote_pixmap;

use anyhow::Result;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use thiserror::Error;

pub use color::*;
pub use file_backed_pixmap::FileBackedPixmap;
pub use in_memory_pixmap::InMemoryPixmap;
pub use remote_pixmap::RemotePixmap;

/// Type used for sharing `[Pixmap]`s between multiple places
pub(crate) type SharedPixmap<P> = Arc<P>;

#[derive(Debug, Error)]
enum GenericError {
    #[error("could not access coordinates {},{} on pixmap of size {}*{}", .target.0, .target.1, .size.0, .size.1)]
    InvalidCoordinates {
        target: (usize, usize),
        size: (usize, usize),
    },
    #[error("cannot create pixmap with invalid size {0}*{1}")]
    InvalidSize(usize, usize),
}

///
/// Generic trait for accessing pixel data in a unified way
///
pub trait Pixmap {
    /// Get the color value of the pixel at position (x,y)
    fn get_pixel(&self, x: usize, y: usize) -> Result<Color>;

    /// Set the pixel value at position (x,y) to the specified color
    fn set_pixel(&self, x: usize, y: usize, color: Color) -> Result<()>;

    /// Get the size of this pixmap as (width, height) tuple
    fn get_size(&self) -> Result<(usize, usize)>;

    /// Get all of the contained pixel data
    fn get_raw_data(&self) -> Result<Vec<Color>>;

    /// Overwrite all of the contained pixel data
    fn put_raw_data(&self, data: &Vec<Color>) -> Result<()>;
}

/// Calculates the index of the specified coordinates when pixels are stored in a Vector in row-major order
fn get_pixel_index(pixmap: &impl Pixmap, x: usize, y: usize) -> Result<usize> {
    Ok(y * pixmap.get_size()?.0 + x)
}

/// Calculate whether the specified coordinates are inside the pixmap
fn are_coordinates_inside(pixmap: &impl Pixmap, x: usize, y: usize) -> Result<bool> {
    let size = pixmap.get_size()?;
    Ok(x < size.0 && y < size.1)
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::TestResult;

    pub(crate) fn test_set_and_get_pixel(
        pixmap: impl Pixmap,
        x: usize,
        y: usize,
        color: Color,
    ) -> TestResult {
        match pixmap.set_pixel(x, y, color) {
            Err(_) => TestResult::discard(),
            Ok(_) => quickcheck::TestResult::from_bool(pixmap.get_pixel(x, y).unwrap() == color),
        }
    }

    pub(crate) fn test_put_and_get_raw_data(pixmap: &impl Pixmap, color: Color) -> TestResult {
        // setup
        let data = vec![color; pixmap.get_size().unwrap().0 * pixmap.get_size().unwrap().1];

        // execution
        pixmap.put_raw_data(&data).unwrap();
        let data_out = pixmap.get_raw_data().unwrap();

        // verification
        TestResult::from_bool(data == data_out)
    }
}
