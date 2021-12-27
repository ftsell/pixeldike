//!
//! Each pixel is encoded into 4 bytes for the color channels red, green, blue and alpha whereby alpha is always 255.
// These bytes are then simply appended to each other in row-major order.
// At the end everything is base64 encoded.
//!

use tokio::time::{interval, Duration};

use crate::pixmap::{Pixmap, SharedPixmap};

use super::SharedMultiEncodings;

static LOG_TARGET: &str = "pixelflut.encoder.rgba64";

/// *RGBA64* encoded pixmap canvas data
pub type Encoding = String;

/// Run the *RGBA64* encoding algorithm in a loop.
///
/// Effectively, this periodically re-encodes the provided *pixmap*'s data into the given
/// *encodings* storage.
pub async fn run_encoder<P>(encodings: SharedMultiEncodings, pixmap: SharedPixmap<P>)
where
    P: Pixmap,
{
    info!(target: LOG_TARGET, "Starting rgba64 encoder");

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

fn encode<P>(pixmap: &SharedPixmap<P>) -> Encoding
where
    P: Pixmap,
{
    // TODO Improve this by mapping the AtomicU32 types to byte slices and then use those directly
    let mut data = Vec::with_capacity(pixmap.get_size().unwrap().0 * pixmap.get_size().unwrap().1 * 4);

    for i in pixmap.get_raw_data().unwrap() {
        let i: u32 = i.into();
        let color = i.to_le_bytes();
        data.push(color[0]);
        data.push(color[1]);
        data.push(color[2]);
        data.push(255);
    }

    base64::encode(&data)
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;

    use crate::pixmap::{Color, InMemoryPixmap};

    use super::*;

    #[test]
    fn test_encoded_content_has_correct_length() {
        let pixmap = SharedPixmap::<InMemoryPixmap>::default();
        let encoded = encode(&pixmap);
        let encoded_bytes = base64::decode(&encoded).unwrap();
        assert_eq!(
            encoded_bytes.len(),
            pixmap.get_size().unwrap().0 * pixmap.get_size().unwrap().1 * 4
        )
    }

    quickcheck! {
        fn test_encoded_color_is_correctly_decodable(x: usize, y: usize, color: u32) -> TestResult {
            // prepare
            let pixmap = SharedPixmap::<InMemoryPixmap>::default();
            let color = color.into();
            if pixmap.set_pixel(x, y, color).is_err() {
                return TestResult::discard()
            }

            // execute
            let encoded = encode(&pixmap);
            let encoded_bytes = base64::decode(&encoded).unwrap();

            // verify
            let i  = (y * pixmap.get_size().unwrap().0 + x) * 4;
            let encoded_color = &encoded_bytes[i..i+3];
            let decoded_color = Color(encoded_color[0], encoded_color[1], encoded_color[2]);
            TestResult::from_bool(decoded_color == color)
        }
    }
}
