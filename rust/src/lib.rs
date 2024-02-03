#![feature(never_type)]
#![feature(cursor_remaining)]
#![feature(sync_unsafe_cell)]
#![feature(int_roundings)]
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

mod daemon_task;
mod i18n;
pub mod net;
pub mod pixmap;
pub mod sinks;

//pub mod gui;

pub use daemon_task::DaemonHandle;
