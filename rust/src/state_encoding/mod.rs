use crate::pixmap::SharedPixmap;

mod encodings;
pub mod rgb64;
pub mod rgba64;

pub use encodings::*;
use tokio::macros::support::Future;

pub fn start_encoders(encodings: SharedMultiEncodings, pixmap: SharedPixmap) {
    start_encoder(encodings.clone(), pixmap.clone(), rgb64::run_encoder);
    start_encoder(encodings, pixmap, rgba64::run_encoder);
}

fn start_encoder<
    F: FnOnce(SharedMultiEncodings, SharedPixmap) -> G + Send + 'static,
    G: Future<Output = ()> + Send,
>(
    encodings: SharedMultiEncodings,
    pixmap: SharedPixmap,
    encoder_function: F,
) {
    tokio::spawn(async move {
        encoder_function(encodings, pixmap).await;
    });
}
