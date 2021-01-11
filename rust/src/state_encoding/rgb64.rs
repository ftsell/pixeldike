use super::SharedMultiEncodings;
use crate::pixmap::SharedPixmap;
use bytes::Bytes;
use tokio::prelude::*;
use tokio::time::{interval, Duration};

pub type Encoding = String;

pub async fn run_encoder(encodings: SharedMultiEncodings, pixmap: SharedPixmap) {
    let mut int = interval(Duration::from_millis(100));
    loop {
        int.tick().await;
        let encoding = encode(&pixmap);

        {
            let mut lock = encodings.rgb64.lock().unwrap();
            (*lock) = encoding;
        }
    }
}

fn encode(pixmap: &SharedPixmap) -> Encoding {
    let data = pixmap.get_raw_data();
    let tmp_data = Vec::with_capacity()
    base64::encode(&data)
}
