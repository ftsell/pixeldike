extern crate argparse;
extern crate futures;

use argparse::{ArgumentParser, StoreTrue, StoreFalse};
use futures::{lazy};

mod pixmap;
mod servers;
mod websocket_server;

use crate::servers::PxServer;


const COMMAND_PORT: u16 = 1234;
const WEBSOCKET_PORT: u16 = 1235;

const X_SIZE: usize = 800;
const Y_SIZE: usize = 600;
const BACKGROUND_COLOR: &str = "FFFFFFFF";


fn main() {
    let mut tcp = true;
    parse_arguments(&mut tcp);

    //let websocket_handler = websocket_server::start(map.clone(), WEBSOCKET_PORT);

    tokio::run(lazy(move || {
        let map = pixmap::Pixmap::new(X_SIZE, Y_SIZE, BACKGROUND_COLOR.to_string());

        if tcp {
            servers::tcp_server::TcpServer::new(map.clone(), COMMAND_PORT).start();
        }

        Ok(())
    }));
}


fn parse_arguments(tcp: &mut bool) {
    let mut parser = ArgumentParser::new();
    parser.set_description("Pixelflut - Pixel drawing game for programmers");

    parser.refer(tcp)
        .add_option(&["--tcp"], StoreTrue, "Use connection based TCP for command input (recommended)")
        .add_option(&["--udp"], StoreFalse, "Use UDP for command input");

    parser.parse_args_or_exit();
}
