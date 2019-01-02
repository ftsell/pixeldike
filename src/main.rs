use std::sync::Mutex;
use std::sync::Arc;

mod websocket_server;
mod udp_server;

const UDP_PORT: u16 = 1234;
const WEBSOCKET_PORT: u16 = 1235;

const X_SIZE: usize = 100;
const Y_SIZE: usize = 100;
const BACKGROUND_COLOR: &str = "FFFFFFFF";


fn main() {
    let map: Arc<Mutex<Vec<Vec<String>>>>
        = Arc::new(Mutex::new(vec![vec![String::from(BACKGROUND_COLOR); Y_SIZE]; X_SIZE]));

    let websocket_handler = websocket_server::start(WEBSOCKET_PORT);
    let udp_handler = udp_server::start(map.clone(), UDP_PORT);

    udp_handler.join().unwrap();
}

