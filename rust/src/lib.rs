#![deny(trivial_numeric_casts, trivial_casts, unsafe_code)]
#![warn(
    missing_crate_level_docs,
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications
)]

//!
//! Pixelflut is a pixel drawing game for programmers inspired by reddits r/place.
//!
//! This library serves as a reference server and client implementation.
//!

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
#[macro_use]
extern crate log;

mod i18n;
pub mod net;
pub mod pixmap;
mod protocol;
pub mod state_encoding;
