extern crate argparse;

use std::sync::{Arc, Mutex};
use argparse::{ArgumentParser, StoreTrue, StoreFalse};

mod udp_server;
mod tcp_server;
mod command_handler;
mod websocket_server;

const COMMAND_PORT: u16 = 1234;
const WEBSOCKET_PORT: u16 = 1235;

const X_SIZE: usize = 800;
const Y_SIZE: usize = 600;
const BACKGROUND_COLOR: &str = "FFFFFFFF";


fn main() {
    let mut tcp = true;
    parse_arguments(&mut tcp);

    let map = generate_map();

    let websocket_handler = websocket_server::start(map.clone(), WEBSOCKET_PORT);

    let command_handler;
    if tcp {
        command_handler = tcp_server::start(map.clone(), COMMAND_PORT);
    } else {
        command_handler = udp_server::start(map.clone(), COMMAND_PORT);
    }

    command_handler.join().unwrap();
    websocket_handler.join().unwrap();
}


fn generate_map() -> Vec<Vec<Arc<Mutex<String>>>> {
    print!("Generating empty canvas...");
    let mut map: Vec<Vec<Arc<Mutex<String>>>> = Vec::new();

    // Fill map with background color
    for x in 0..X_SIZE {
        map.push(Vec::new());
        for _y in 0..Y_SIZE {
            map[x].push(Arc::new(Mutex::new(BACKGROUND_COLOR.to_string())));
        }
    }

    println!("done");
    return map;
}


fn parse_arguments(tcp: &mut bool) {
    let mut parser = ArgumentParser::new();
    parser.set_description("Pixelflut - Pixel drawing game for programmers");

    parser.refer(tcp)
        .add_option(&["--tcp"], StoreTrue, "Use connection based TCP for command input (recommended)")
        .add_option(&["--udp"], StoreFalse, "Use UDP for command input");

    parser.parse_args_or_exit();
}
