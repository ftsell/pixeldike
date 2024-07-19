use crate::pixmap::Color;
use std::cell::SyncUnsafeCell;
use thiserror::Error;

/// A fast pixel storage implementation
#[derive(Debug)]
pub struct Pixmap {
    data: SyncUnsafeCell<Vec<Color>>,
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
        if width == 0 || height == 0 {
            return Err(InvalidSizeError {
                size: (width, height),
                details: "Width and Height must both be greater than 0",
            });
        }

        Ok(Self {
            data: SyncUnsafeCell::new(vec![Color::default(); width * height]),
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
        let i = y.saturating_mul(self.width).saturating_add(x);
        match unsafe { self.get_color_data() }.get(i) {
            None => Err(InvalidCoordinatesError {
                target: (x, y),
                pixmap_size: self.get_size(),
            }),
            Some(color) => Ok(*color),
        }
    }

    /// Set the pixel value at position (x,y) to the specified color
    pub fn set_pixel(&self, x: usize, y: usize, color: Color) -> Result<(), InvalidCoordinatesError> {
        let i = y.saturating_mul(self.width).saturating_add(x);
        match unsafe { self.get_color_data() }.get_mut(i) {
            None => Err(InvalidCoordinatesError {
                target: (x, y),
                pixmap_size: self.get_size(),
            }),
            Some(stored_color) => {
                *stored_color = color;
                Ok(())
            }
        }
    }

    /// Get a (usable) handle to the raw data that is contained in the pixmap
    ///
    /// # Safety
    /// No memory safety rules are ensured for this data.
    /// The handed out mutable reference is not checked to be the only one and the underlying data may change at any time.
    /// It is completely unsafe and undefined behavior to work with the returned data.
    ///
    /// # Reasoning
    /// While it is undefined behavior to use the returned handle, it works as expected and it is _very_ fast to store
    /// and work with pixel data in this way.
    /// Since pixelflut is designed to be primarily fast and does not intend to offer a consistent view of the pixel
    /// data this has show to be fine.
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn get_color_data(&self) -> &mut [Color] {
        &mut *self.data.get()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{quickcheck, TestResult};

    quickcheck! {
        fn test_set_and_get_pixel(x: usize, y: usize) -> TestResult {
            let color = Color::from((0xAB, 0xAB, 0xAB));
            let pixmap = Pixmap::new(80, 60).unwrap();
            match pixmap.set_pixel(x, y, color) {
                Err(_) => TestResult::discard(),
                Ok(_) => {
                    let got_color = pixmap.get_pixel(x, y).unwrap();
                    TestResult::from_bool(color == got_color)
                }
            }
        }
    }
}
