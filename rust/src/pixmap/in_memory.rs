use super::*;
use std::sync::atomic::Ordering;

pub struct InMemoryPixmap {
    data: Vec<AtomicU32>,
    width: usize,
    height: usize,
}

impl InMemoryPixmap {
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

            Ok(InMemoryPixmap {
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

    pub(crate) fn get_raw_data(&self) -> &Vec<AtomicU32> {
        &self.data
    }
}

impl Pixmap for InMemoryPixmap {
    fn get_pixel(&self, x: usize, y: usize) -> Option<Color> {
        if !self.are_coordinates_inside(x, y) {
            None
        } else {
            let i = self.get_pixel_index(x, y);
            Some(Color::from(self.data[i].load(Ordering::Relaxed)))
        }
    }

    fn set_pixel(&self, x: usize, y: usize, color: Color) -> bool {
        if !self.are_coordinates_inside(x, y) {
            false
        } else {
            let i = self.get_pixel_index(x, y);
            self.data[i].store(color.into(), Ordering::SeqCst);
            true
        }
    }

    fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

impl Default for InMemoryPixmap {
    fn default() -> Self {
        Self::new(800, 600).unwrap()
    }
}

/*
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
 */
