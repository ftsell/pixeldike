use crate::X_SIZE;
use crate::Y_SIZE;
use std::sync::Arc;
use std::sync::Mutex;
use core::ops::RangeTo;


pub fn cmd_size() -> String {
    format!("SIZE {} {}", X_SIZE, Y_SIZE)
}

pub fn cmd_px(map: &Vec<Vec<Arc<Mutex<String>>>>, x: usize, y: usize, color: String) -> String {
    let answer = format!("PX {} {} {}", x, y, &color);

    // Check that coordinates are inside the grid
    if x >= X_SIZE || y >= Y_SIZE {
        return format!(
            "Coordinates {}:{} not inside grid (0-{}:0-{})",
            x,
            y,
            X_SIZE - 1,
            Y_SIZE - 1
        );
    }

    // Retrieve entry from map
    let mutex: &Arc<Mutex<String>> = map.get(x).unwrap().get(y).unwrap();

    // Lock mutex for modification
    {
        let mut entry = mutex.lock().unwrap();
        // Overwrite the contained value of this element
        *entry = color;
    }

    answer
}
