use clap::value_t_or_exit;
use pixelflut;
use pretty_env_logger;
use std::net::SocketAddr;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;

mod cli;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let matches = cli::get_app().get_matches();

    match matches.subcommand() {
        ("server", Some(sub_matches)) => {
            start_server(
                value_t_or_exit!(sub_matches, "width", usize),
                value_t_or_exit!(sub_matches, "height", usize),
                sub_matches
                    .value_of("path")
                    .expect("path is required but not in matches"),
                value_t_or_exit_opt!(sub_matches, "tcp_port", usize),
                value_t_or_exit_opt!(sub_matches, "udp_port", usize),
                value_t_or_exit_opt!(sub_matches, "ws_port", usize),
            )
            .await;
        }
        _ => {}
    }
}

async fn start_server(
    width: usize,
    height: usize,
    path: &str,
    tcp_port: Option<usize>,
    udp_port: Option<usize>,
    ws_port: Option<usize>,
) {
    let pixmap = Arc::new(
        pixelflut::pixmap::FileBackedPixmap::new(&Path::new(path), width, height, false)
            .expect("could not create pixmap"),
    );
    let encodings = pixelflut::state_encoding::SharedMultiEncodings::default();
    let mut handles = Vec::new();

    if let Some(tcp_port) = tcp_port {
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        handles.push(tokio::spawn(async move {
            pixelflut::net::tcp_server::listen(
                pixmap,
                encodings,
                pixelflut::net::tcp_server::TcpOptions {
                    listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", tcp_port))
                        .expect("could not build SocketAddr"),
                },
            )
            .await;
        }));
    }

    if let Some(udp_port) = udp_port {
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        handles.push(tokio::spawn(async move {
            pixelflut::net::udp_server::listen(
                pixmap,
                encodings,
                pixelflut::net::udp_server::UdpOptions {
                    listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", udp_port))
                        .expect("could not build SocketAddr"),
                },
            )
            .await;
        }))
    }

    if let Some(ws_port) = ws_port {
        handles.push(tokio::spawn(async move {
            pixelflut::net::ws_server::listen(
                pixmap.clone(),
                encodings.clone(),
                pixelflut::net::ws_server::WsOptions {
                    listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", ws_port))
                        .expect("could not build SocketAddr"),
                },
            )
            .await;
        }))
    }

    for handle in handles {
        let _ = tokio::join!(handle);
    }
}
