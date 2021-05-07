mod color;
mod file_backed;
mod in_memory;

use anyhow::Result;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use thiserror::Error;

pub use color::*;
pub use file_backed::FileBackedPixmap;
pub use in_memory::InMemoryPixmap;

pub(crate) type SharedPixmap<P> = Arc<P>;

// TODO Improve error handling

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

pub trait Pixmap {
    fn get_pixel(&self, x: usize, y: usize) -> Result<Color>;

    fn set_pixel(&self, x: usize, y: usize, color: Color) -> Result<()>;

    fn get_size(&self) -> Result<(usize, usize)>;

    fn get_raw_data(&self) -> Result<Vec<Color>>;

    fn put_raw_data(&self, data: &Vec<Color>) -> Result<()>;
}

/// Calculates the vector index of the specified coordinates
fn get_pixel_index(pixmap: &impl Pixmap, x: usize, y: usize) -> Result<usize> {
    Ok(y * pixmap.get_size()?.0 + x)
}

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
