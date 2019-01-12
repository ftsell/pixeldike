use std::sync::{Mutex, Arc};


pub struct Pixmap {
    map: Vec<Vec<Arc<Mutex<String>>>>,
    pub x_size: usize,
    pub y_size: usize,
}


impl Pixmap {
    pub fn new(x_size: usize, y_size: usize, color: String) -> Pixmap {
        let mut map: Vec<Vec<Arc<Mutex<String>>>> = Vec::new();

        // Fill map with background color
        for x in 0..x_size {
            map.push(Vec::new());
            for _y in 0..y_size {
                map[x].push(Arc::new(Mutex::new(color.clone())));
            }
        }

        return Pixmap {
            map,
            x_size,
            y_size,
        };
    }

    fn check_coordinates_in_map(&self, x: usize, y: usize) -> Result<(), String> {
        if x >= self.x_size || y >= self.y_size {
            return Err(format!("Coordinates {},{} not inside grid: 0-{},0-{}",
                               x,
                               y,
                               self.x_size,
                               self.y_size
            ).to_string());
        }

        Ok(())
    }

    pub fn set_pixel(&self, x: usize, y: usize, color: String) -> Result<(), String> {
        // Make sure that coordinates are valid
        self.check_coordinates_in_map(x, y).and_then(|()| {

            // Retrieve entry from map
            let mutex: &Arc<Mutex<String>> = self.map.get(x).unwrap().get(y).unwrap();

            // Lock mutex for modification
            {
                let mut entry = mutex.lock().unwrap();
                // Overwrite the contained value of this element
                *entry = color;
            }

            Ok(())
        })
    }

    pub fn get_pixel(&self, x: usize, y: usize) -> Result<String, String> {
        // Make sure that coordinates are valid
        self.check_coordinates_in_map(x, y).and_then(|()| {

            let color ;
            // Retrieve entry from map
            let mutex: &Arc<Mutex<String>> = self.map.get(x).unwrap().get(y).unwrap();

            // Lock mutex for reading
            {
                let mut entry = mutex.lock().unwrap();
                // Overwrite the contained value of this element
                color = (*entry).clone();
            }

            Ok(color)

        })
    }

    pub fn get_size(&self) -> String {
        format!("{} {}", self.x_size, self.y_size)
    }
}


impl Clone for Pixmap {
    fn clone(&self) -> Self {
        Pixmap {
            map: self.map.clone(),
            x_size: self.x_size,
            y_size: self.y_size,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.map = source.map.clone();
        self.x_size = source.x_size;
        self.y_size = source.y_size;
    }
}
