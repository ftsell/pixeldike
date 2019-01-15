extern crate argparse;
extern crate futures;
extern crate tokio;

use argparse::{ArgumentParser, StoreTrue, StoreFalse};
use futures::{lazy};
use tokio::runtime::Runtime;

mod pixmap;
mod servers;

use crate::servers::PxServer;


const COMMAND_PORT: u16 = 1234;
const WEBSOCKET_PORT: u16 = 1235;

const X_SIZE: usize = 800;
const Y_SIZE: usize = 600;
const BACKGROUND_COLOR: &str = "FFFFFFFF";


fn main() {
    let mut tcp = true;
    let mut udp = true;
    parse_arguments(&mut tcp, &mut udp);

    tokio::run(lazy(move || {
        let map = pixmap::Pixmap::new(X_SIZE, Y_SIZE, BACKGROUND_COLOR.to_string());

        if tcp {
            servers::tcp_server::TcpServer::new(map.clone()).start(COMMAND_PORT);
        }

        if udp {
            servers::udp_server::UdpServer::new(map.clone()).start(COMMAND_PORT + 1);
        }

        if true {
            servers::websocket_server::WsServer::new(map.clone()).start(COMMAND_PORT + 2);
        }

        Ok(())
    }));
}


fn parse_arguments(tcp: &mut bool, udp: &mut bool) {
    let mut parser = ArgumentParser::new();
    parser.set_description("Pixelflut - Pixel drawing game for programmers");

    parser.refer(tcp)
        .add_option(&["--tcp"], StoreTrue, "Use connection based TCP for command input (recommended)");

    parser.parse_args_or_exit();

    *udp = true; // TODO Enable argparse for udp
}
