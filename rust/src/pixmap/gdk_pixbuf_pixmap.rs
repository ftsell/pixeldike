use crate::pixmap::{verify_coordinates_are_inside, Color, GenericError as Error, Pixmap};
use gtk::gdk_pixbuf::Pixbuf;
use itertools::Itertools;
use std::cmp::min;
use std::sync::RwLock;

/// A [`Pixmap`] implementation which uses a [`gdk-pixbuf`](Pixbuf) as underlying storage
#[derive(Debug)]
pub struct GdkPixbufPixmap {
    pixbuf: RwLock<Pixbuf>,
}

impl GdkPixbufPixmap {
    pub fn new(width: usize, height: usize) -> anyhow::Result<Self> {
        if width == 0 || height == 0 {
            Err(Error::InvalidSize(width, height).into())
        } else {
            Ok(Self {
                pixbuf: RwLock::new(
                    Pixbuf::new(
                        gtk::gdk_pixbuf::Colorspace::Rgb,
                        false,
                        8,
                        width as i32,
                        height as i32,
                    )
                    .expect("Could not construct GDK Pixbuf"),
                ),
            })
        }
    }
}

impl Default for GdkPixbufPixmap {
    fn default() -> Self {
        GdkPixbufPixmap::new(800, 600).expect("Could not construct GDK Pixbuf pixmap with default size")
    }
}

impl Pixmap for GdkPixbufPixmap {
    fn get_pixel(&self, x: usize, y: usize) -> anyhow::Result<Color> {
        verify_coordinates_are_inside(self, x, y)?;

        let pixbuf = self
            .pixbuf
            .read()
            .expect("Could not acquire read lock PixbufMutex");
        let n_channels = pixbuf.n_channels();
        let row_stride = pixbuf.rowstride();
        let pixels = pixbuf
            .pixel_bytes()
            .expect("GDK Pixbuf has no underlying buffer anymore");

        Ok(Color(
            pixels[y * row_stride as usize + x * n_channels as usize],
            pixels[(y * row_stride as usize + x * n_channels as usize) + 1],
            pixels[(y * row_stride as usize + x * n_channels as usize) + 2],
        ))
    }

    #[allow(unsafe_code)]
    fn set_pixel(&self, x: usize, y: usize, color: Color) -> anyhow::Result<()> {
        verify_coordinates_are_inside(self, x, y)?;

        let pixbuf = self
            .pixbuf
            .write()
            .expect("Could not acquire write lock PixbufMutex");

        let n_channels = pixbuf.n_channels() as usize;
        let row_stride = pixbuf.rowstride() as usize;

        // retrieve bytes in a safe way first and then modify it in an unsafe way
        {
            pixbuf
                .pixel_bytes()
                .expect("GDK Pixbuf has no underlying buffer anymore");
        }
        // this needs to be unsafe because gdk pixbuf normally does not allow mutation
        unsafe {
            let pixels = pixbuf.pixels();
            pixels[y * row_stride + x * n_channels] = color.0;
            pixels[(y * row_stride + x * n_channels) + 1] = color.1;
            pixels[(y * row_stride + x * n_channels) + 2] = color.2;
        }

        Ok(())
    }

    fn get_size(&self) -> anyhow::Result<(usize, usize)> {
        let pixbuf = self.pixbuf.read().expect("Could not acquire lock PixbufMutex");
        Ok((pixbuf.width() as usize, pixbuf.height() as usize))
    }

    fn get_raw_data(&self) -> anyhow::Result<Vec<Color>> {
        let pixbuf = self
            .pixbuf
            .read()
            .expect("Could not acquire read lock on GdkPixbuf");

        Ok(pixbuf
            .pixel_bytes()
            .expect("Could not get bytes from GdkPixbuf")
            .iter()
            .chunks(3)
            .into_iter()
            .map(|chunk| {
                let chunks: Vec<_> = chunk.collect();
                Color(chunks[0].clone(), chunks[1].clone(), chunks[2].clone())
            })
            .collect())
    }

    #[allow(unsafe_code)]
    fn put_raw_data(&self, data: &Vec<Color>) -> anyhow::Result<()> {
        let size = self.get_size().expect("Could not get size of own Pixmap");
        let pixbuf = self
            .pixbuf
            .write()
            .expect("Could not acquire write lock on GdkPixbuf");

        let n_channels = pixbuf.n_channels() as usize;

        // try to get bytes safely and afterwards in an unsafe way
        pixbuf.pixel_bytes().expect("Could not get bytes from GdkPixbuf");
        // this needs to be unsafe because gdk pixbuf normally does not implement mutation
        unsafe {
            let pixels = pixbuf.pixels();
            for (i, color) in data[..min(data.len(), size.0 * size.1)].iter().enumerate() {
                pixels[i * n_channels] = color.0;
                pixels[i * n_channels + 1] = color.1;
                pixels[i * n_channels + 2] = color.2;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;

    use super::*;
    use crate::pixmap::test;

    quickcheck! {
        fn test_set_and_get_pixel(width: usize, height: usize, x: usize, y: usize, color: Color) -> TestResult {
            match GdkPixbufPixmap::new(width, height) {
                Err(_) => TestResult::discard(),
                Ok(pixmap) => test::test_set_and_get_pixel(pixmap, x, y, color)
            }
        }
    }

    quickcheck! {
        fn test_put_and_get_raw_data(color: Color) -> TestResult {
            let pixmap = GdkPixbufPixmap::default();
            test::test_put_and_get_raw_data(&pixmap, color)
        }
    }

    #[test]
    fn test_put_raw_data_with_incorrect_size_data() {
        let pixmap = GdkPixbufPixmap::default();
        test::test_put_raw_data_with_incorrect_size_data(&pixmap);
    }
}
