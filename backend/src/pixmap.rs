use crate::color::{Color};

pub struct Pixmap {
    map: Vec<Vec<Color>>,
    pub x_size: usize,
    pub y_size: usize,
}

impl Pixmap {
    pub fn new(x_size: usize, y_size: usize, background_color: Color) -> Pixmap {
        let mut map: Vec<Vec<Color>> = Vec::new();

        // Fill the map with background color
        for x in 0..x_size {
            map.push(Vec::new());
            for y in 0..y_size {
                map[x].push(background_color.clone());
            }
        };

        return Pixmap {
            map,
            x_size,
            y_size,
        };
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Result<&u32, String> {
        match self.map.get(x) {
            None => Err(format!("X value {} is outside of map", x).to_string()),
            Some(column) => {
                match column.get(y) {
                    None => Err(format!("Y value {} is outside of map", y).to_string()),
                    Some(value) => Ok(value)
                }
            }
        }
    }

    pub fn get_pixel_mut(&mut self, x: usize, y: usize) -> Result<&mut u32, String> {
        match self.map.get_mut(x) {
            None => Err(format!("X value {} is outside of map", x).to_string()),
            Some(column) => {
                match column.get_mut(y) {
                    None => Err(format!("Y value {} is outside of map", y).to_string()),
                    Some(value) => Ok(value)
                }
            }
        }
    }
}
