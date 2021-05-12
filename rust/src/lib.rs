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
extern crate nom;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate gettext_macros;
extern crate gettext;
#[macro_use]
extern crate log;
extern crate byteorder;
extern crate thiserror;

use crate::net::start_listeners;
use crate::pixmap::Pixmap;
use crate::state_encoding::{start_encoders, SharedMultiEncodings};
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

mod i18n;
mod net;
pub mod pixmap;
mod protocol;
mod state_encoding;

///
/// Start a pixelflut server
///
/// # Panics:
/// - When no tokio runtime is running
///
pub async fn run_server<P>(pixmap: P)
where
    P: Pixmap + Send + Sync + 'static,
{
    info!(target: "pixelflut", "Starting server");

    let pixmap = Arc::new(pixmap);
    let encodings: SharedMultiEncodings = SharedMultiEncodings::default();

    let mut handles = Vec::new();
    handles.append(&mut start_encoders(encodings.clone(), pixmap.clone()));
    handles.append(&mut start_listeners(
        pixmap,
        encodings,
        net::NetOptions {
            tcp: Some(net::tcp_server::TcpOptions {
                listen_address: SocketAddr::from_str("0.0.0.0:1234").unwrap(),
            }),
            udp: Some(net::udp_server::UdpOptions {
                listen_address: SocketAddr::from_str("0.0.0.0:1234").unwrap(),
            }),
            ws: Some(net::ws_server::WsOptions {
                listen_address: SocketAddr::from_str("0.0.0.0:1235").unwrap(),
            }),
        },
    ));

    for handle in handles {
        let _ = tokio::join!(handle);
    }
}
