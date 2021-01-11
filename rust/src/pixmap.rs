use crate::parser::command::StateAlgorithm;
use bytes::Bytes;
use nom::lib::std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Color(pub u8, pub u8, pub u8);

impl From<u32> for Color {
    fn from(src: u32) -> Self {
        let b = src.to_le_bytes();
        Color(b[0], b[1], b[2])
    }
}

impl Into<u32> for Color {
    fn into(self) -> u32 {
        0u32 | (self.0 as u32) | (self.1 as u32) << 8 | (self.2 as u32) << 16
    }
}

impl Into<Vec<u8>> for Color {
    fn into(self) -> Vec<u8> {
        vec![self.0, self.1, self.2]
    }
}

impl ToString for Color {
    fn to_string(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.0, self.1, self.2)
    }
}

pub type SharedPixmap = Arc<Pixmap>;

pub struct Pixmap {
    data: Vec<AtomicU32>,
    width: usize,
    height: usize,
}

impl Pixmap {
    /// Creates a new Pixmap with the specified dimensions.
    ///
    /// *Panics* if either num_shards, width or height is zero.
    pub fn new(width: usize, height: usize) -> Result<Self, &'static str> {
        if width == 0 {
            Err("width is 0")
        } else if height == 0 {
            Err("height is 0")
        } else {
            let size = width * height;

            Ok(Pixmap {
                data: (0..size).map(|_| AtomicU32::new(0)).collect(),
                width,
                height,
            })
        }
    }

    /// Calculates the vector index of the specified coordinates
    fn get_pixel_index(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn are_coordinates_inside(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<Color> {
        if !self.are_coordinates_inside(x, y) {
            None
        } else {
            let i = self.get_pixel_index(x, y);
            Some(Color::from(self.data[i].load(Ordering::Relaxed)))
        }
    }

    pub fn set_pixel(&self, x: usize, y: usize, color: Color) -> bool {
        if !self.are_coordinates_inside(x, y) {
            false
        } else {
            let i = self.get_pixel_index(x, y);
            self.data[i].store(color.into(), Ordering::SeqCst);
            true
        }
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    pub(crate) fn get_raw_data(&self) -> &Vec<AtomicU32> {
        &self.data
    }
}

impl Default for Pixmap {
    fn default() -> Self {
        Self::new(800, 600).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::TestResult;

    quickcheck! {
        fn test_set_and_get_pixel(width: usize, height: usize, x: usize, y: usize, color: u32) -> TestResult {
            match Pixmap::new(width, height) {
                Err(_) => TestResult::discard(),
                Ok(pixmap) => {
                    let color = color.into();
                    match pixmap.set_pixel(x, y, color) {
                        false => TestResult::discard(),
                        true => quickcheck::TestResult::from_bool(pixmap.get_pixel(x, y).unwrap() == color)
                    }
                }
            }
        }
    }
}
