use std::sync::Arc;
use std::sync::Mutex;

mod udp_server;
mod websocket_server;

const UDP_PORT: u16 = 1234;
const WEBSOCKET_PORT: u16 = 1235;

const X_SIZE: usize = 800;
const Y_SIZE: usize = 600;
const BACKGROUND_COLOR: &str = "FFFFFFFF";

fn main() {
    print!("Generating empty canvas...");
    let map: Arc<Mutex<Vec<Vec<String>>>> =
        Arc::new(Mutex::new(vec![
            vec![String::from(BACKGROUND_COLOR); Y_SIZE];
            X_SIZE
        ]));
    println!("done");

    let websocket_handler = websocket_server::start(WEBSOCKET_PORT);
    let udp_handler = udp_server::start(map.clone(), UDP_PORT);

    udp_handler.join().unwrap();
    websocket_handler.join().unwrap();
}
