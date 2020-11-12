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

impl ToString for Color {
    fn to_string(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.0, self.1, self.2)
    }
}

pub type SharedPixmap = Arc<Pixmap>;

pub struct Pixmap {
    data: Vec<Mutex<Vec<Color>>>,
    width: usize,
    height: usize,
}

impl Pixmap {
    /// Creates a new Pixmap with the specified dimensions.
    ///
    /// *Panics* if either num_shards, width or height is zero.
    pub fn new(width: usize, height: usize, num_shards: usize) -> Result<Self, &'static str> {
        if width == 0 {
            Err("width is 0")
        } else if height == 0 {
            Err("height is 0")
        } else if num_shards == 0 {
            Err("num_shards i 0")
        } else if num_shards >= width * height {
            Err("num_shards requests more shards than there is data (num_shards >= width * height)")
        } else if (width * height) % num_shards != 0 {
            Err("num_shards would result in unequal shard sizes (width * height % num_shards != 0)")
        } else {
            let size = width * height;
            let shard_size = size / num_shards;

            Ok(Pixmap {
                data: (0..num_shards)
                    .map(|_| Mutex::new(vec![0u32.into(); shard_size]))
                    .collect(),
                width,
                height,
            })
        }
    }

    /// Calculates the tuple `(shard_number, index_in_shard)` of the specified pixel
    fn get_pixel_index(&self, x: usize, y: usize) -> (usize, usize) {
        let global_i = y * self.width + x;
        let shard_size = self.width * self.height / self.data.len();
        let local_i = global_i % shard_size;
        let shard_num = (global_i - local_i) / shard_size;

        (shard_num, local_i)
    }

    fn are_coordinates_inside(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Option<Color> {
        if !self.are_coordinates_inside(x, y) {
            None
        } else {
            let (shard_index, i) = self.get_pixel_index(x, y);
            let shard = self.data.get(shard_index).unwrap();
            {
                let lock = shard.lock().unwrap();
                return Some(lock.get(i).unwrap().clone());
            }
        }
    }

    pub fn set_pixel(&self, x: usize, y: usize, color: Color) -> bool {
        if !self.are_coordinates_inside(x, y) {
            false
        } else {
            let (shard_index, i) = self.get_pixel_index(x, y);
            let shard = self.data.get(shard_index).unwrap();
            {
                let mut lock = shard.lock().unwrap();
                let shard_data = &mut (*lock);
                shard_data[i] = color;
            }
            true
        }
    }

    pub fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    /*
    pub(crate) fn get_raw_data(&self) -> Vec<Color> {
        let shard_size = self.width * self.height / self.data.len();
        let mut result: Vec<Color> = Vec::with_capacity(self.width * self.height);

        let mut shard_data;
        for shard in &self.data {
            {
                let lock = shard.lock().unwrap();
                shard_data = lock.clone();
            }
            result.append(&mut shard_data);
        }

        result
    }
     */
}

impl Default for Pixmap {
    fn default() -> Self {
        Self::new(800, 600, 10).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::TestResult;

    quickcheck! {
        fn test_set_and_get_pixel(width: usize, height: usize, x: usize, y: usize, color: u32) -> TestResult {
            match Pixmap::new(width, height, 1) {
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

    quickcheck! {
        fn test_set_and_get_pixel_sharded(width: usize, height: usize, num_shards: usize, x: usize, y: usize) -> TestResult {
            match Pixmap::new(width, height, num_shards) {
                Err(_) => TestResult::discard(),
                Ok(pixmap) => {
                    let color = Color(42, 43, 44);
                    match pixmap.set_pixel(x, y, color) {
                        false => TestResult::discard(),
                        true => TestResult::from_bool(pixmap.get_pixel(x, y).unwrap() == color)
                    }
                }
             }
        }
    }
}
