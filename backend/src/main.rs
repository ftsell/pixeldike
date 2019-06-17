extern crate argparse;
extern crate futures;
extern crate hex;
extern crate tokio;

use argparse::{ArgumentParser, StoreTrue, Store};
use futures::lazy;

mod color;
mod network;
mod pixmap;

use crate::color::{color_from_rgba, Color};
use crate::network::protocol::Command;
use crate::network::px_server::PxServer;
use crate::network::tcp_server::TcpServer;
use std::sync::Arc;

const BACKGROUND_COLOR: Color = 0x000000FF_u32;      // Black with no transparency

fn main() {
    let args = parse_arguments();

    tokio::run(lazy(move || {
        println!("Creating empty canvas of size {}x{}", args.x_size, args.y_size);
        let mut map = Arc::new(pixmap::Pixmap::new(
            args.x_size,
            args.y_size,
            BACKGROUND_COLOR,
        ));

        if args.tcp != 0 {
            let mut tcp_server = TcpServer::new(map.clone());
            tcp_server.start(&"0.0.0.0".to_string(), args.tcp);
        }

        if args.tcp == 0 && args.udp == 0 && args.ws == 0 {
            println!("Not starting anything because no ports were specified.\n\
            Add --help for more info.")
        }

        Ok(())
    }));
}

struct Args {
    tcp: u16,
    udp: u16,
    ws: u16,
    x_size: usize,
    y_size: usize
}

fn parse_arguments() -> Args {
    let mut args = Args {
        tcp: 0,
        udp: 0,
        ws: 0,
        x_size: 800,
        y_size: 600
    };

    {
        let mut parser = ArgumentParser::new();
        parser.set_description("Pixelflut - Pixel drawing game for programmers");

        parser
            .refer(&mut args.tcp)
            .add_option(&["--tcp"], Store, "Enable TCP PX server on port");

        parser
            .refer(&mut args.udp)
            .add_option(&["--udp"], Store, "Enable UDP PX server");

        parser
            .refer(&mut args.ws)
            .add_option(&["--ws"], Store, "Enable Websocket PX server");

        parser
            .refer(&mut args.x_size)
            .add_option(&["-x"], Store, "Size of the canvas in X dimension");

        parser
            .refer(&mut args.y_size)
            .add_option(&["-y"], Store, "Size of the canvas in Y dimension");

        parser.parse_args_or_exit();
    }

    return args;
}
