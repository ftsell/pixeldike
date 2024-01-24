//!
//! Data structures to store pixel data, also called *Pixmaps*
//!

use std::sync::Arc;

pub use color::*;

mod color;
// mod file_backed_pixmap;
mod storage;

pub use storage::{InvalidCoordinatesError, Pixmap};

/// A [`Pixmap`] which can be used throughout multiple threads
///
/// This is simply an [`Arc`] around a pixmap because pixmaps are already implementing
/// interior mutability and thus are already [`Send`] and [`Sync`]. The Arc then allows actual
/// sharing between multiple contexts because it provides a [`Clone`] implementation that refers
/// to the same data.
pub type SharedPixmap = Arc<Pixmap>;
