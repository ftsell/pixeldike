//!
//! Pixmap implementation on top of [GdkPixbuf](Pixbuf)
//!

use crate::pixmap::{
    pixel_index_2_coordinates, verify_coordinates_are_inside, Color, GenericError as Error, Pixmap,
};
use gtk::gdk_pixbuf::{Colorspace, Pixbuf};
use std::cmp::min;

/// Create a new [GdkPixbuf](Pixbuf) with the given dimensions
pub fn new_gdk_pixbuf_pixmap(width: i32, height: i32) -> anyhow::Result<Pixbuf> {
    if width == 0 || height == 0 {
        Err(Error::InvalidSize(width as usize, height as usize).into())
    } else {
        let pixbuf = Pixbuf::new(Colorspace::Rgb, false, 8, width, height).unwrap();
        //pixbuf.put_raw_data(&vec![Color(0, 0, 0); (width * height) as usize])?;
        Ok(pixbuf)
    }
}

/// Return a new [GdkPixbuf](Pixbuf) with default pixmap settings
pub fn default_gdk_pixbuf_pixmap() -> Pixbuf {
    new_gdk_pixbuf_pixmap(800, 600).expect("Could not construct default gdk pixbuf pixmap")
}

impl Pixmap for Pixbuf {
    fn get_pixel(&self, x: usize, y: usize) -> anyhow::Result<Color> {
        verify_coordinates_are_inside(self, x, y)?;

        let n_channels = self.n_channels() as usize;
        assert!(n_channels >= 3);
        let row_stride = self.rowstride() as usize;
        let pixels = self
            .read_pixel_bytes()
            .expect("Could not get pixels from GDK Pixbuf");

        Ok(Color(
            pixels[y * row_stride + x * n_channels],
            pixels[y * row_stride + x * n_channels + 1],
            pixels[y * row_stride + x * n_channels + 2],
        ))
    }

    fn set_pixel(&self, x: usize, y: usize, color: Color) -> anyhow::Result<()> {
        verify_coordinates_are_inside(self, x, y)?;
        self.put_pixel(x as u32, y as u32, color.0, color.1, color.2, 0);
        Ok(())
    }

    fn get_size(&self) -> anyhow::Result<(usize, usize)> {
        Ok((self.width() as usize, self.height() as usize))
    }

    fn get_raw_data(&self) -> anyhow::Result<Vec<Color>> {
        let n_channels = self.n_channels() as usize;
        assert!(n_channels >= 3);

        let (width, height) = self.get_size()?;
        let row_stride = self.rowstride() as usize;
        let pixels = self
            .read_pixel_bytes()
            .expect("Could not get pixels from GdkPixbuf");

        let mut result = Vec::with_capacity(width * height);
        for iy in 0..height {
            for ix in 0..width {
                result.push(Color(
                    pixels[iy * row_stride + ix * n_channels],
                    pixels[iy * row_stride + ix * n_channels + 1],
                    pixels[iy * row_stride + ix * n_channels + 2],
                ))
            }
        }

        Ok(result)
    }

    fn put_raw_data(&self, data: &Vec<Color>) -> anyhow::Result<()> {
        for (i, color) in data[..min(data.len(), self.width() as usize * self.height() as usize)]
            .iter()
            .enumerate()
        {
            let (x, y) = pixel_index_2_coordinates(self, i)?;
            self.put_pixel(x as u32, y as u32, color.0, color.1, color.2, 0);
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use quickcheck::{quickcheck, TestResult};

    use super::*;
    use crate::pixmap::test;

    quickcheck! {
        fn test_set_and_get_pixel(x: usize, y: usize, color: Color) -> TestResult {
            let pixbuf = default_gdk_pixbuf_pixmap();
            test::test_set_and_get_pixel(pixbuf, x, y, color)
        }
    }

    quickcheck! {
        fn test_put_and_get_raw_data(color: Color) -> TestResult {
            let pixbuf = default_gdk_pixbuf_pixmap();
            test::test_put_and_get_raw_data(&pixbuf, color)
        }
    }

    #[test]
    fn test_put_raw_data_with_incorrect_size_data() {
        let pixbuf = default_gdk_pixbuf_pixmap();
        test::test_put_raw_data_with_incorrect_size_data(&pixbuf);
    }
}
