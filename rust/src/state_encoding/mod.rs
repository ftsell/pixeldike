//!
//! Encoding of pixmaps with different algorithms
//!
//! A pixelflut server is able to send it's pixmap using different encoding mechanisms to a
//! requesting clients.
//! This module implements the defined encoding algorithms and also provides background threads
//! which periodically re-encode a pixmap.
//!

use std::sync::Arc;
use tokio::sync::Notify;
use tokio::task::JoinHandle;

pub use encodings::*;

use crate::pixmap::traits::{PixmapBase, PixmapRawRead};
use crate::pixmap::SharedPixmap;

mod encodings;
pub mod rgb64;
pub mod rgba64;

/// Start background tasks for all encoding algorithms and return join handles to those tasks
pub fn start_encoders<P>(
    encodings: SharedMultiEncodings,
    pixmap: SharedPixmap<P>,
) -> Vec<(JoinHandle<()>, Arc<Notify>)>
where
    P: PixmapBase + PixmapRawRead + Send + Sync + 'static,
{
    vec![
        rgb64::start_encoder(encodings.clone(), pixmap.clone()),
        rgba64::start_encoder(encodings.clone(), pixmap.clone()),
    ]
}
