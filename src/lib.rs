#![feature(never_type)]
#![feature(sync_unsafe_cell)]
#![feature(int_roundings)]
#![feature(test)]
#![deny(trivial_casts)]
#![warn(
    rustdoc::missing_crate_level_docs,
    rustdoc::broken_intra_doc_links,
    rustdoc::private_intra_doc_links,
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
#[cfg(test)]
extern crate test;

pub mod net;
pub mod pixmap;
pub mod sinks;
mod texts;

/// The result type which all background tasks return
pub type DaemonResult = anyhow::Result<!>;
