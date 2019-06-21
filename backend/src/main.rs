extern crate argparse;
extern crate futures;
extern crate hex;
extern crate tokio;
extern crate base64;

mod color;
mod network;
mod pixmap;

use argparse::{ArgumentParser, Store};
use futures::lazy;
use crate::color::{Color};
use crate::network::px_server::PxServer;
use crate::network::tcp_server::TcpServer;
use std::sync::Arc;
use std::{time};
use futures::stream::Stream;
use crate::pixmap::Pixmap;

const BACKGROUND_COLOR: Color = 0x000000_u32;      // Black

fn main() {
    let args = parse_arguments();

    tokio::run(lazy(move || {
        println!("Creating empty canvas of size {}x{}", args.x_size, args.y_size);
        let map = Arc::new(pixmap::Pixmap::new(
            args.x_size,
            args.y_size,
            BACKGROUND_COLOR,
        ));

        if args.tcp != 0 {
            let tcp_server = TcpServer::new(map.clone());
            tcp_server.start(&"0.0.0.0".to_string(), args.tcp);
        }

        if args.tcp == 0 && args.udp == 0 && args.ws == 0 {
            println!("Not starting anything because no ports were specified.\n\
            Add --help for more info.")
        } else {
            schedule_pixmap_snapshots(3, map.clone());
        }

        Ok(())
    }));
}

fn schedule_pixmap_snapshots(frequency: u16, map: Arc<Pixmap>) {
    let sleep_duration = time::Duration::from_millis((1000 / frequency) as u64);

    let background_task = tokio::timer::Interval::new_interval(sleep_duration)
        .map_err(|e| eprintln!("{}", e.to_string()))
        .for_each(move |_| {
            map.create_snapshot();
            Ok(())
        });

    tokio::spawn(background_task);
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
