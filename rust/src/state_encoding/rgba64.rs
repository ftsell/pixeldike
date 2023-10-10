//!
//! Each pixel is encoded into 4 bytes for the color channels red, green, blue and alpha whereby alpha is always 255.
// These bytes are then simply appended to each other in row-major order.
// At the end everything is base64 encoded.
//!

use std::sync::Arc;
use tokio::select;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

use crate::pixmap::traits::{PixmapBase, PixmapRawRead};
use crate::pixmap::SharedPixmap;

use super::SharedMultiEncodings;

static LOG_TARGET: &str = "pixelflut.encoder.rgba64";

/// *RGBA64* encoded pixmap canvas data
pub type Encoding = String;

/// Start the *RGBA64* encoding algorithm on a new task.
///
/// Effectively, this periodically re-encodes the provided *pixmap*'s data into the given
/// *encodings* storage in the background.
pub fn start_encoder<P>(
    encodings: SharedMultiEncodings,
    pixmap: SharedPixmap<P>,
) -> (JoinHandle<()>, Arc<Notify>)
where
    P: PixmapBase + PixmapRawRead + Send + Sync + 'static,
{
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();
    let handle = tokio::spawn(async move { run_encoder(encodings, pixmap, notify2).await });

    (handle, notify)
}

/// Run the *RGBA64* encoding algorithm in a loop.
///
/// Effectively, this periodically re-encodes the provided *pixmap*'s data into the given
/// *encodings* storage.
pub async fn run_encoder<P>(
    encodings: SharedMultiEncodings,
    pixmap: SharedPixmap<P>,
    notify_stop: Arc<Notify>,
) where
    P: PixmapBase + PixmapRawRead,
{
    tracing::info!(target: LOG_TARGET, "Starting rgba64 encoder");

    let mut timer = interval(Duration::from_millis(100));
    loop {
        select! {
            _ = timer.tick() => {
                let encoding = encode(&pixmap);
                {
                    let mut lock = encodings.rgba64.lock().unwrap();
                    (*lock) = encoding;
                }
            },
            _ = notify_stop.notified() => {
                tracing::info!("Stopping rgba64 encoder");
                break
            }
        }
    }
}

fn encode<P>(pixmap: &SharedPixmap<P>) -> Encoding
where
    P: PixmapBase + PixmapRawRead,
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

    // TODO Re-Enable
    // quickcheck! {
    //     fn test_encoded_color_is_correctly_decodable(x: usize, y: usize, color: u32) -> TestResult {
    //         // prepare
    //         let pixmap = SharedPixmap::<InMemoryPixmap>::default();
    //         let color = color.into();
    //         if pixmap.set_pixel(x, y, color).is_err() {
    //             return TestResult::discard()
    //         }
    //
    //         // execute
    //         let encoded = encode(&pixmap);
    //         let encoded_bytes = base64::decode(&encoded).unwrap();
    //
    //         // verify
    //         let i  = (y * pixmap.get_size().unwrap().0 + x) * 4;
    //         let encoded_color = &encoded_bytes[i..i+3];
    //         let decoded_color = Color(encoded_color[0], encoded_color[1], encoded_color[2]);
    //         TestResult::from_bool(decoded_color == color)
    //     }
    // }
}
