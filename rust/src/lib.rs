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
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate gettext_macros;
extern crate gettext;

use crate::pixmap::{Pixmap, SharedPixmap};
use gettext::Catalog;
use std::sync::Arc;

mod i18n;
mod net;
mod parser;
mod pixmap;

pub async fn start_server() {
    let pixmap: SharedPixmap = Arc::new(Pixmap::default());
    let pixmap2 = pixmap.clone();
    let handle1 = tokio::spawn(async move {
        net::tcp_server::start(pixmap2).await;
    });
    let pixmap2 = pixmap.clone();
    let handle2 = tokio::spawn(async move {
        net::udp_server::start(pixmap2).await;
    });

    tokio::join!(handle1, handle2);
}
