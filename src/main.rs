extern crate argparse;

use std::sync::Arc;
use std::sync::Mutex;
use argparse::{ArgumentParser, StoreTrue, StoreFalse};

mod udp_server;
mod tcp_server;
mod command_handler;
mod websocket_server;

const COMMAND_PORT: u16 = 1234;
const WEBSOCKET_PORT: u16 = 1235;

const X_SIZE: usize = 200;
const Y_SIZE: usize = 100;
const BACKGROUND_COLOR: &str = "FFFFFFFF";

fn main() {
    print!("Generating empty canvas...");
    let map: Arc<Mutex<Vec<Vec<String>>>> =
        Arc::new(Mutex::new(vec![
            vec![String::from(BACKGROUND_COLOR); Y_SIZE];
            X_SIZE
        ]));
    println!("done");

    let mut tcp = true;
    parse_arguments(&mut tcp);


    let websocket_handler = websocket_server::start(map.clone(),WEBSOCKET_PORT);
    let command_handler;
    if tcp {
        command_handler = tcp_server::start(map.clone(),COMMAND_PORT);
    }
    else {
        command_handler = udp_server::start(map.clone(), COMMAND_PORT);
    }

    command_handler.join().unwrap();
    websocket_handler.join().unwrap();
}


fn parse_arguments(tcp: &mut bool) {
    let mut parser = ArgumentParser::new();
    parser.set_description("Pixelflut - Pixel drawing game for programmers");

    parser.refer(tcp)
        .add_option(&["--tcp"], StoreTrue, "Use connection based TCP for command input (recommended)")
        .add_option(&["--udp"], StoreFalse, "Use UDP for command input");

    parser.parse_args_or_exit();
}
