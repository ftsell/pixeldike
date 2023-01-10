use std::cmp::min;
use std::sync::atomic::Ordering;

use anyhow::Result;

use super::traits::*;
use super::GenericError as Error;
use super::*;

///
/// A pixmap implementation based on an in-memory store of AtomicU32
///
#[derive(Debug)]
pub struct InMemoryPixmap {
    data: Vec<AtomicU32>,
    width: usize,
    height: usize,
}

impl InMemoryPixmap {
    /// Creates a new Pixmap with the specified dimensions.
    pub fn new(width: usize, height: usize) -> Result<Self> {
        if width == 0 || height == 0 {
            Err(Error::InvalidSize(width, height).into())
        } else {
            let size = width * height;

            Ok(InMemoryPixmap {
                data: (0..size).map(|_| AtomicU32::new(0)).collect(),
                width,
                height,
            })
        }
    }
}

impl PixmapBase for InMemoryPixmap {
    fn get_size(&self) -> Result<(usize, usize)> {
        Ok((self.width, self.height))
    }
}

impl PixmapRead for InMemoryPixmap {
    fn get_pixel(&self, x: usize, y: usize) -> Result<Color> {
        verify_coordinates_are_inside(self, x, y)?;

        let i = pixel_coordinates_2_index(self, x, y)?;
        Ok(Color::from(self.data[i].load(Ordering::Relaxed)))
    }
}

impl PixmapWrite for InMemoryPixmap {
    fn set_pixel(&self, x: usize, y: usize, color: Color) -> Result<()> {
        verify_coordinates_are_inside(self, x, y)?;

        let i = pixel_coordinates_2_index(self, x, y)?;
        self.data[i].store(color.into(), Ordering::SeqCst);
        Ok(())
    }
}

impl PixmapRawRead for InMemoryPixmap {
    fn get_raw_data(&self) -> Result<Vec<Color>> {
        Ok(self
            .data
            .iter()
            .map(|v| v.load(Ordering::Relaxed))
            .map(|v| Color::from(v))
            .collect())
    }
}

impl PixmapRawWrite for InMemoryPixmap {
    fn put_raw_data(&self, data: &[Color]) -> Result<()> {
        for (i, color) in data[..min(data.len(), self.width * self.height)]
            .iter()
            .enumerate()
        {
            let color: Color = color.clone().into();
            self.data[i].store(color.into(), Ordering::Relaxed);
        }

        Ok(())
    }
}

impl Default for InMemoryPixmap {
    fn default() -> Self {
        Self::new(800, 600).unwrap()
    }
}

#[cfg(test)]
mod test {
    use quickcheck::TestResult;

    use super::super::test;
    use super::*;

    quickcheck! {
        fn test_set_and_get_pixel(width: usize, height: usize, x: usize, y: usize, color: Color) -> TestResult {
            match InMemoryPixmap::new(width, height) {
                Err(_) => TestResult::discard(),
                Ok(pixmap) => test::test_set_and_get_pixel(pixmap, x, y, color)
            }
        }
    }

    quickcheck! {
        fn test_put_and_get_raw_data(color: Color) -> TestResult {
            let pixmap = InMemoryPixmap::default();
            test::test_put_and_get_raw_data(&pixmap, color)
        }
    }

    #[test]
    fn test_put_raw_data_with_incorrect_size_data() {
        let pixmap = InMemoryPixmap::default();
        test::test_put_raw_data_with_incorrect_size_data(&pixmap);
    }
}
