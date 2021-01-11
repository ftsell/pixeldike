use crate::pixmap::SharedPixmap;

mod encodings;
pub mod rgb64;
pub mod rgba64;

pub use encodings::*;
use std::future::Future;
use tokio::task::JoinHandle;

pub fn start_encoders(
    encodings: SharedMultiEncodings,
    pixmap: SharedPixmap,
) -> Vec<JoinHandle<()>> {
    vec![
        start_encoder(encodings.clone(), pixmap.clone(), rgb64::run_encoder),
        start_encoder(encodings, pixmap, rgba64::run_encoder),
    ]
}

fn start_encoder<
    F: FnOnce(SharedMultiEncodings, SharedPixmap) -> G + Send + 'static,
    G: Future<Output = ()> + Send,
>(
    encodings: SharedMultiEncodings,
    pixmap: SharedPixmap,
    encoder_function: F,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        encoder_function(encodings, pixmap).await;
    })
}
