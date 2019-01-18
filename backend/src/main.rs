extern crate argparse;
extern crate futures;
extern crate tokio;

use argparse::{ArgumentParser, StoreTrue};
use futures::lazy;

mod pixmap;
mod servers;

use crate::servers::PxServer;

const TCP_PORT: u16 = 1234;
const UDP_PORT: u16 = 1234;
const WEBSOCKET_PORT: u16 = 1235;

const X_SIZE: usize = 800;
const Y_SIZE: usize = 600;
const BACKGROUND_COLOR: &str = "FFFFFFFF";

fn main() {
    let args = parse_arguments();

    tokio::run(lazy(move || {
        let map = pixmap::Pixmap::new(X_SIZE, Y_SIZE, BACKGROUND_COLOR.to_string());

        if args.tcp {
            servers::tcp_server::TcpServer::new(map.clone()).start(TCP_PORT);
        }

        if args.udp {
            servers::udp_server::UdpServer::new(map.clone()).start(UDP_PORT);
        }

        if args.ws {
            servers::websocket_server::WsServer::new(map.clone()).start(WEBSOCKET_PORT);
        }

        if !args.tcp && !args.udp && !args.ws {
            eprintln!("Not starting anything because no server spceified");
            eprintln!("pixelflut --help for more info");
        }

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
