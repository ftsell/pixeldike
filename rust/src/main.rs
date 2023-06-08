use clap::Parser;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::Arc;

use pretty_env_logger;

use pixelflut;
use pixelflut::pixmap::traits::*;

mod cli;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let args = cli::CliOpts::parse();
    match args.command {
        cli::Command::Server(opts) => start_server(&opts).await,
    };
}

async fn start_server(opts: &cli::ServerOpts) {
    // create pixmap instances
    let primary_pixmap = pixelflut::pixmap::InMemoryPixmap::new(opts.width, opts.height)
        .expect("could not create in memory pixmap");
    let file_pixmap =
        pixelflut::pixmap::FileBackedPixmap::new(&opts.path, opts.width, opts.height, false).expect(
            &format!("could not create pixmap backed by file {}", opts.path.display()),
        );

    // copy data from file into memory
    primary_pixmap
        .put_raw_data(
            &file_pixmap
                .get_raw_data()
                .expect("could not load pixel data from file"),
        )
        .expect("could not put pixel data into memory");

    // create final pixmap instance which automatically saves data into file
    let pixmap = Arc::new(
        pixelflut::pixmap::ReplicatingPixmap::new(primary_pixmap, vec![Box::new(file_pixmap)], 0.2).unwrap(),
    );
    let encodings = pixelflut::state_encoding::SharedMultiEncodings::default();
    let mut server_handles = Vec::new();

    let gui_handle = if opts.show_gui {
        let (handle, proxy) = pixelflut::gui::GuiThread::start(pixmap.clone());
        Some(handle)
    } else {
        None
    };

    if let Some(tcp_port) = &opts.tcp_port {
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        let (handle, _) = pixelflut::net::tcp_server::start_listener(
            pixmap,
            encodings,
            pixelflut::net::tcp_server::TcpOptions {
                listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", tcp_port))
                    .expect("could not build SocketAddr"),
            },
        );
        server_handles.push(handle);
    }

    if let Some(udp_port) = &opts.udp_port {
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        let (handle, _) = pixelflut::net::udp_server::start_listener(
            pixmap,
            encodings,
            pixelflut::net::udp_server::UdpOptions {
                listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", udp_port))
                    .expect("could not build SocketAddr"),
            },
        );
        server_handles.push(handle);
    }

    if let Some(ws_port) = &opts.ws_port {
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        let (handle, _) = pixelflut::net::ws_server::start_listener(
            pixmap.clone(),
            encodings.clone(),
            pixelflut::net::ws_server::WsOptions {
                listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", ws_port))
                    .expect("could not build SocketAddr"),
            },
        );
        server_handles.push(handle);
    }

    if server_handles.len() == 0 {
        panic!("No listeners are supposed to be started which makes no sense");
    }

    let encoder_handles = pixelflut::state_encoding::start_encoders(encodings, pixmap);

    if let Some(handle) = gui_handle {
        handle.thread.await;
    }
    for handle in server_handles {
        let _ = tokio::join!(handle);
    }
    for handle in encoder_handles {
        let _ = tokio::join!(handle.0);
    }
}
