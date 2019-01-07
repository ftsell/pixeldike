use crate::X_SIZE;
use crate::Y_SIZE;
use std::sync::Arc;
use std::sync::Mutex;


pub enum Command {
    SIZE,
    PX(usize, usize, String),
}


pub fn parse_message(msg: String) -> Result<Command, String> {
    if msg.eq(&String::from("SIZE")) {
        return Ok(Command::SIZE);
    } else if msg.starts_with("PX") {
        // Define iterator over all fields in command and ignore PX part at the beginning
        let mut msg_iterator = msg.split_whitespace();
        msg_iterator.next();
        // Extract values from command
        let x = msg_iterator.next();
        let y = msg_iterator.next();
        let color = msg_iterator.next();

        // Check that every data point could be extracted
        if !(x.is_some() && y.is_some() && color.is_some()) {
            return Err(String::from("Could not extract data from PX command"));
        }

        let x = x.unwrap().parse::<usize>();
        let y = y.unwrap().parse::<usize>();
        let color = {
            if color.unwrap().len() == 6 {
                (color.unwrap().to_string() + "FF")
            } else {
                color.unwrap().to_string()
            }
        };

        if x.is_err() || y.is_err() {
            return Err(String::from("Could not parse xy position"));
        }

        return Ok(Command::PX(x.unwrap(), y.unwrap(), color));
    }

    return Err(String::from(
        "Could not parse message. It is neither a SIZE nor PX command",
    ));
}

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