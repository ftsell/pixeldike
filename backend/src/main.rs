extern crate argparse;
extern crate futures;
extern crate hex;
extern crate tokio;

use argparse::{ArgumentParser, StoreTrue};
use futures::lazy;

mod color;
mod network;
mod pixmap;
//mod servers;

use crate::color::{color_from_rgba, Color};
use crate::network::protocol::Command;
use crate::network::px_server::PxServer;
use crate::network::tcp_server::TcpServer;
use std::sync::Arc;

const TCP_PORT: u16 = 1234;
const UDP_PORT: u16 = 1234;
const WEBSOCKET_PORT: u16 = 1235;

const X_SIZE: usize = 800;
const Y_SIZE: usize = 600;
const BACKGROUND_COLOR: Color = 0x000000FF_u32;      // Black with no transparency

fn main() {
    let args = parse_arguments();

    tokio::run(lazy(move || {
        let mut map = Arc::new(pixmap::Pixmap::new(
            X_SIZE,
            Y_SIZE,
            BACKGROUND_COLOR,
        ));

        let mut tcp_server = TcpServer::new(map.clone());
        tcp_server.start(&"127.0.0.1".to_string(), 1234);

        Ok(())
    }));
}

struct Args {
    tcp: bool,
    udp: bool,
    ws: bool,
}

fn parse_arguments() -> Args {
    let mut args = Args {
        tcp: false,
        udp: false,
        ws: false,
    };

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Pixelflut - Pixel drawing game for programmers");

        parser
            .refer(&mut args.tcp)
            .add_option(&["--tcp"], StoreTrue, "Enable TCP PX server");

        parser
            .refer(&mut args.udp)
            .add_option(&["--udp"], StoreTrue, "Enable UDP PX server");

        parser
            .refer(&mut args.ws)
            .add_option(&["--ws"], StoreTrue, "Enable Websocket PX server");

        parser.parse_args_or_exit();
    }

    return args;
}
