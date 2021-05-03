mod color;
mod in_memory;

use std::sync::atomic::AtomicU32;
use std::sync::Arc;

pub use color::*;
pub use in_memory::InMemoryPixmap;

pub type SharedPixmap = Arc<Box<InMemoryPixmap>>;

pub trait Pixmap {
    fn get_pixel(&self, x: usize, y: usize) -> Option<Color>;

    fn set_pixel(&self, x: usize, y: usize, color: Color) -> bool;

    fn get_size(&self) -> (usize, usize);
}
