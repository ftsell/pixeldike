mod color;
mod in_memory;

use std::sync::atomic::AtomicU32;
use std::sync::Arc;

pub use color::*;
pub use in_memory::InMemoryPixmap;

pub type SharedPixmap<P> = Arc<P>;

pub trait Pixmap {
    fn get_pixel(&self, x: usize, y: usize) -> Option<Color>;

    fn set_pixel(&self, x: usize, y: usize, color: Color) -> bool;

    fn get_size(&self) -> (usize, usize);

    fn get_raw_data(&self) -> Vec<Color>;

    fn put_raw_data(&self, data: &Vec<Color>);
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
            false => TestResult::discard(),
            true => quickcheck::TestResult::from_bool(pixmap.get_pixel(x, y).unwrap() == color),
        }
    }

    pub(crate) fn test_put_and_get_raw_data(pixmap: impl Pixmap, color: Color) -> TestResult {
        // setup
        let data = vec![color; pixmap.get_size().0 * pixmap.get_size().1];

        // execution
        pixmap.put_raw_data(&data);
        let data_out = pixmap.get_raw_data();

        // verification
        TestResult::from_bool(data == data_out)
    }
}
