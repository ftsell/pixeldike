use crate::pixmap::Color;
use std::sync::atomic::{AtomicU32, Ordering};
use thiserror::Error;

/// A fast pixel storage implementation
#[derive(Debug)]
pub struct Pixmap {
    data: Vec<AtomicU32>,
    width: usize,
    height: usize,
}

/// An error which indicates that invalid coordinates could not be accessed
#[derive(Debug, Error, Copy, Clone)]
#[error("Could not access invalid coordinates {}x{} on pixmap of size {}x{}", .target.0, .target.1, .pixmap_size.0, .pixmap_size.1)]
pub struct InvalidCoordinatesError {
    target: (usize, usize),
    pixmap_size: (usize, usize),
}

/// An error which indicates that a pixmap of a given size cannot be constructed
#[derive(Debug, Error, Copy, Clone)]
#[error("Given size {}x{} is not valid for constructing a pixmap: {details}", .size.0, .size.1)]
pub struct InvalidSizeError {
    size: (usize, usize),
    details: &'static str,
}

impl Pixmap {
    /// Create a new Pixmap with the specified dimensions
    pub fn new(width: usize, height: usize) -> Result<Self, InvalidSizeError> {
        Self::new_with_initial_color(width, height, Color::default())
    }

    /// Create a new pixmap with the specified dimensions and initial color
    pub fn new_with_initial_color(
        width: usize,
        height: usize,
        color: Color,
    ) -> Result<Self, InvalidSizeError> {
        if width == 0 || height == 0 {
            return Err(InvalidSizeError {
                size: (width, height),
                details: "Width and Height must both be greater than 0",
            });
        }

        let size = width * height;
        Ok(Self {
            data: (0..size).map(|_| AtomicU32::new(color.into())).collect(),
            width,
            height,
        })
    }

    /// Get the size of this pixmap as `(width, height)` tuple
    pub fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /// Get the color value of the pixel at position (x,y)
    pub fn get_pixel(&self, x: usize, y: usize) -> Result<Color, InvalidCoordinatesError> {
        Ok(self.get_storage(x, y)?.load(Ordering::Relaxed).into())
    }

    /// Set the pixel value at position (x,y) to the specified color
    pub fn set_pixel(&self, x: usize, y: usize, color: Color) -> Result<(), InvalidCoordinatesError> {
        self.get_storage(x, y)?.store(color.into(), Ordering::Relaxed);
        Ok(())
    }

    /// Get the raw data that is contained in the pixmap
    pub fn get_raw_data(&self) -> Vec<Color> {
        self.data
            .iter()
            .map(|d| d.load(Ordering::Relaxed).into())
            .collect()
    }

    /// Get the U32 that stores pixel data for the given coordinates
    fn get_storage(&self, x: usize, y: usize) -> Result<&AtomicU32, InvalidCoordinatesError> {
        let i = y * self.width + x;
        match self.data.get(i) {
            None => Err(InvalidCoordinatesError {
                target: (x, y),
                pixmap_size: self.get_size(),
            }),
            Some(data) => Ok(data),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{quickcheck, TestResult};

    quickcheck! {
        fn test_set_and_get_pixel(x: usize, y: usize, color: Color) -> TestResult {
            let pixmap = Pixmap::new(800, 600).unwrap();
            match pixmap.set_pixel(x, y, color) {
                Err(_) => TestResult::discard(),
                Ok(_) => {
                    let got_color = pixmap.get_pixel(x, y).unwrap();
                    TestResult::from_bool(color == got_color)
                }
            }
        }
    }

    // pub(crate) fn test_put_and_get_raw_data(
    //     pixmap: &(impl PixmapBase + PixmapRawRead + PixmapRawWrite),
    //     color: Color,
    // ) -> TestResult {
    //     // setup
    //     let data = vec![color; pixmap.get_size().unwrap().0 * pixmap.get_size().unwrap().1];
    //
    //     // execution
    //     pixmap.put_raw_data(&data).unwrap();
    //     let data_out = pixmap.get_raw_data().unwrap();
    //
    //     // verification
    //     println!("{:?}", data);
    //     println!("{:?}", data_out);
    //     TestResult::from_bool(data == data_out)
    // }
    //
    // pub(crate) fn test_put_raw_data_with_incorrect_size_data(
    //     pixmap: &(impl PixmapBase + PixmapWrite + PixmapRawRead + PixmapRawWrite),
    // ) {
    //     // setup
    //     let size = pixmap.get_size().unwrap().0 * pixmap.get_size().unwrap().1;
    //
    //     // empty data
    //     pixmap.set_pixel(0, 0, Color(42, 42, 42)).unwrap();
    //     pixmap.set_pixel(1, 0, Color(43, 43, 43)).unwrap();
    //     pixmap.put_raw_data(&Vec::<Color>::new()).unwrap();
    //     let output_data: Vec<_> = pixmap.get_raw_data().unwrap();
    //     assert_eq!(output_data[0], Color(42, 42, 42));
    //     assert_eq!(output_data[1], Color(43, 43, 43));
    //     assert_eq!(output_data[2..], vec![Color(0, 0, 0); size - 2]);
    //
    //     // too small data
    //     let input_data = vec![Color(42, 42, 42); 10];
    //     pixmap.put_raw_data(&input_data).unwrap();
    //     let output_data: Vec<_> = pixmap.get_raw_data().unwrap();
    //     assert_eq!(output_data[0..10], input_data);
    //     assert_eq!(output_data[10..], vec![Color(0, 0, 0); size - 10]);
    //
    //     // too large data
    //     let input_data = vec![Color(42, 42, 42); size + 10];
    //     pixmap.put_raw_data(&input_data).unwrap();
    //     let output_data = pixmap.get_raw_data().unwrap();
    //     assert_eq!(output_data, vec![Color(42, 42, 42); size]);
    // }
}
