use super::SharedMultiEncodings;
use crate::pixmap::SharedPixmap;
use bytes::Bytes;
use std::sync::atomic::Ordering;
use tokio::prelude::*;
use tokio::time::{interval, Duration};

pub type Encoding = String;

pub async fn run_encoder(encodings: SharedMultiEncodings, pixmap: SharedPixmap) {
    let mut int = interval(Duration::from_millis(100));
    loop {
        int.tick().await;
        let encoding = encode(&pixmap);

        {
            let mut lock = encodings.rgba64.lock().unwrap();
            (*lock) = encoding;
        }
    }
}

fn encode(pixmap: &SharedPixmap) -> Encoding {
    // TODO Improve this by mapping the AtomicU32 types to byte slices and then use those directly
    let mut data = Vec::with_capacity(pixmap.get_size().0 * pixmap.get_size().1 * 4);

    for i in pixmap.get_raw_data() {
        let color = i.load(Ordering::Relaxed).to_le_bytes();
        data.push(color[0]);
        data.push(color[1]);
        data.push(color[2]);
        data.push(0);
    }

    base64::encode(&data)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::pixmap::Color;
    use quickcheck::TestResult;

    #[test]
    fn test_encoded_content_has_correct_length() {
        let pixmap = SharedPixmap::default();
        let encoded = encode(&pixmap);
        let encoded_bytes = base64::decode(&encoded).unwrap();
        assert_eq!(
            encoded_bytes.len(),
            pixmap.get_size().0 * pixmap.get_size().1 * 4
        )
    }

    quickcheck! {
        fn test_encoded_color_is_correctly_decodable(x: usize, y: usize, color: u32) -> TestResult {
            // prepare
            let mut pixmap = SharedPixmap::default();
            let color = color.into();
            if !pixmap.set_pixel(x, y, color) {
                return TestResult::discard()
            }

            // execute
            let encoded = encode(&pixmap);
            let encoded_bytes = base64::decode(&encoded).unwrap();

            // verify
            let i  = (y * pixmap.get_size().0 + x) * 4;
            let encoded_color = &encoded_bytes[i..i+3];
            let decoded_color = Color(encoded_color[0], encoded_color[1], encoded_color[2]);
            TestResult::from_bool(decoded_color == color)
        }
    }
}
