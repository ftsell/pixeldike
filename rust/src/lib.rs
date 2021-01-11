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
#![feature(async_closure)]

//!
//! Pixelflut is a pixel drawing game for programmers inspired by reddits r/place.
//!
//! This library serves as a reference server and client implementation.
//!

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
extern crate nom;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate gettext_macros;
extern crate gettext;
#[macro_use]
extern crate log;

use crate::net::start_listeners;
use crate::pixmap::{Pixmap, SharedPixmap};
use crate::state_encoding::{start_encoders, SharedMultiEncodings};
use tokio::task::JoinHandle;

mod i18n;
mod net;
mod parser;
mod pixmap;
mod state_encoding;

///
/// Start a pixelflut server
///
/// # Panics:
/// - When no tokio runtime is running
///
pub async fn run_server() {
    info!(target: "pixelflut", "Starting server");

    let pixmap: SharedPixmap = SharedPixmap::default();
    let encodings: SharedMultiEncodings = SharedMultiEncodings::default();

    let mut handles = start_encoders(encodings, pixmap.clone());
    handles.append(&mut start_listeners(pixmap));

    for handle in handles {
        tokio::join!(handle);
    }
}
