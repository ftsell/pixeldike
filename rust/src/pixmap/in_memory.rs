use super::*;
use std::sync::atomic::Ordering;

pub struct InMemoryPixmap {
    data: Vec<AtomicU32>,
    width: usize,
    height: usize,
}

///
/// A pixmap implementation based on an in-memory store of AtomicU32
///
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

    fn get_raw_data(&self) -> Vec<Color> {
        return self
            .data
            .iter()
            .map(|v| v.load(Ordering::Relaxed))
            .map(|v| Color::from(v))
            .collect();
    }

    fn put_raw_data(&self, data: &Vec<Color>) {
        for (i, color) in data.iter().enumerate() {
            self.data[i].store(color.into(), Ordering::Relaxed)
        }
    }
}

impl Default for InMemoryPixmap {
    fn default() -> Self {
        Self::new(800, 600).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::super::test as super_test;
    use super::*;
    use quickcheck::TestResult;

    quickcheck! {
        fn test_set_and_get_pixel(width: usize, height: usize, x: usize, y: usize, color: u32) -> TestResult {
            match InMemoryPixmap::new(width, height) {
                Err(_) => TestResult::discard(),
                Ok(pixmap) => super_test::test_set_and_get_pixel(pixmap, x, y, color)
            }
        }
    }

    quickcheck! {
        fn test_put_and_get_raw_data(color: u32) -> TestResult {
            let pixmap = InMemoryPixmap::default();
            super_test::test_put_and_get_raw_data(pixmap, color)
        }
    }
}
