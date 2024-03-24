//!
//! Support for saving pixelflut canvases into various sinks
//!

pub mod ffmpeg;
pub mod framebuffer;
pub mod pixmap_file;
#[cfg(feature = "windowing")]
pub mod window;
