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

#[cfg(test)]
#[macro_use]
extern crate quickcheck;
extern crate nom;

use crate::pixmap::{Pixmap, SharedPixmap};
use std::sync::Arc;

mod net;
mod parser;
mod pixmap;

pub async fn start_server() {
    let pixmap: SharedPixmap = Arc::new(Pixmap::default());
    net::tcp_server::start(&pixmap).await;
}
