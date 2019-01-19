extern crate clap;
extern crate dns_lookup;

mod pixmap;
mod px_client;

use self::pixmap::Pixmap;
use self::clap::{App, Arg};
use self::dns_lookup::lookup_host;
use std::net::SocketAddr;

fn main() {
    let addr = get_socket_address();

    let pixmap = Pixmap::new(10, 10, "FF0000".to_string());

    let client = px_client::start(pixmap.clone(), addr);

    display_window();
}

fn get_socket_address() -> SocketAddr {
    let matches = App::new("Pixelflut client")
        .version("1.0")
        .author("Finn-Thorben Sell")
        .about("Client for displaying a remote pixelflut canvas")
        .long_about("The remote canvas needs to support the STATE command \
        which the implementation by me does. \n\
        See https://github.com/ftsell/pixelflut where you can find a compatible server and \
        this client.")
        .arg(
            Arg::with_name("remote")
                .short("r")
                .long("remote")
                .number_of_values(1)
                .help("Remote server to use like <server>:<port>")
        )
        .get_matches();

    let arg_remote = matches.value_of("remote")
        .expect("Remote server needs to be specified");

    let addr = arg_remote.parse()
        .or_else(|_| -> Result<SocketAddr, ()> {
            let split: Vec<&str> = arg_remote.split(":").collect();
            let ip = lookup_host("finn-thorben.me")
                .expect("Could not resolve host address");

            Ok(SocketAddr::new(ip[0], split[1].parse().unwrap()))
        }).unwrap();

    return addr;
}

fn display_window() {}
