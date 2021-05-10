use super::SharedMultiEncodings;
use crate::pixmap::{Color, Pixmap, SharedPixmap};
use anyhow::{Error, Result};
use tokio::time::{interval, Duration};

static LOG_TARGET: &str = "pixelflut.encoder.rgb64";

pub type Encoding = String;

pub async fn run_encoder<P>(encodings: SharedMultiEncodings, pixmap: SharedPixmap<P>)
where
    P: Pixmap,
{
    info!(target: LOG_TARGET, "Starting rgb64 encoder");

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

fn encode<P>(pixmap: &SharedPixmap<P>) -> Encoding
where
    P: Pixmap,
{
    let mut data = Vec::with_capacity(pixmap.get_size().unwrap().0 * pixmap.get_size().unwrap().1 * 3);

    for i in pixmap.get_raw_data().unwrap() {
        let i: u32 = i.into();
        let color = i.to_le_bytes();
        data.push(color[0]);
        data.push(color[1]);
        data.push(color[2]);
    }

    base64::encode(&data)
}

pub fn decode(data: Encoding) -> Result<Vec<Color>> {
    let mut result = Vec::new();

    let mut color = [0u8; 3];
    for (i, i_value) in base64::decode(data)?.iter().enumerate() {
        if i % 3 == 0 {
            color[0] = *i_value;
        } else if i % 3 == 1 {
            color[1] = *i_value;
        } else if i % 3 == 2 {
            color[2] = *i_value;
            result.push(color.into());
        }
    }

    Ok(result)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::pixmap::{Color, InMemoryPixmap};
    use quickcheck::TestResult;

    #[test]
    fn test_encoded_content_has_correct_length() {
        let pixmap = SharedPixmap::<InMemoryPixmap>::default();
        let encoded = encode(&pixmap);
        let encoded_bytes = base64::decode(&encoded).unwrap();
        assert_eq!(
            encoded_bytes.len(),
            pixmap.get_size().unwrap().0 * pixmap.get_size().unwrap().1 * 3
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
            let i  = (y * pixmap.get_size().unwrap().0 + x) * 3;
            let encoded_color = &encoded_bytes[i..i+3];
            let decoded_color = Color(encoded_color[0], encoded_color[1], encoded_color[2]);
            TestResult::from_bool(decoded_color == color)
        }
    }
}
