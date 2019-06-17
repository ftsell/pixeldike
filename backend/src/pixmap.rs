use crate::color::Color;
use std::sync::Mutex;

pub struct Pixmap {
    map: Vec<Vec<Mutex<Color>>>,
    pub x_size: usize,
    pub y_size: usize,
}

impl Pixmap {
    pub fn new(x_size: usize, y_size: usize, background_color: Color) -> Pixmap {
        let mut map = Vec::new();

        // Fill the map with background color
        for x in 0..x_size {
            map.push(Vec::new());
            for y in 0..y_size {
                map[x].push(Mutex::new(background_color.clone()));
            }
        }

        return Pixmap {
            map,
            x_size,
            y_size,
        };
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Result<Color, String> {
        match self.map.get(x) {
            None => Err(format!("X value {} is outside of map", x).to_string()),
            Some(column) => match column.get(y) {
                None => Err(format!("Y value {} is outside of map", y).to_string()),
                Some(value) => {
                    // Lock mutex for reading
                    let mutex = value.lock().unwrap();
                    return Ok((*mutex).clone());
                },
            },
        }
    }

    pub fn set_pixel(&self, x: usize, y: usize, color: Color) -> Result<(), String> {
        match self.map.get(x) {
            None => Err(format!("X value {} is outside of map", x).to_string()),
            Some(column) => match column.get(y) {
                None => Err(format!("Y value {} is outside of map", y).to_string()),
                Some(value) => {
                    let mut mutex = value.lock().unwrap();
                    (*mutex) = color;
                    Ok(())
                },
            },
        }
    }
}
