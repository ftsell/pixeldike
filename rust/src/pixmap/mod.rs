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
