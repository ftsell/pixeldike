use clap::Parser;
use std::net::ToSocketAddrs;
use std::path::Path;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

use pixelflut::net::clients::GenClient;
use pixelflut::net::framing::MsgWriter;
use pixelflut::net::protocol::Request;
use pixelflut::net::servers::{GenServer, TcpServerOptions, UdpServer, UdpServerOptions};
use pixelflut::pixmap::Color;
use pixelflut::DaemonHandle;

mod cli;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(console_subscriber::spawn())
        .with(
            tracing_subscriber::fmt::layer()
                .event_format(tracing_subscriber::fmt::format().compact())
                .with_filter(
                    EnvFilter::builder()
                        .with_default_directive(Directive::from_str("debug").unwrap())
                        .with_env_var(EnvFilter::DEFAULT_ENV)
                        .from_env()
                        .unwrap(),
                ),
        )
        .init();

    let args = cli::CliOpts::parse();
    match args.command {
        cli::Command::Server(opts) => start_server(&opts).await,
        cli::Command::Client(opts) => start_client(&opts).await,
    };
}

async fn start_server(opts: &cli::ServerOpts) {
    // create pixmap instances
    let pixmap = Arc::new(
        pixelflut::pixmap::InMemoryPixmap::new(opts.width, opts.height)
            .expect("could not create in memory pixmap"),
    );

    // create final pixmap instance which automatically saves data into file
    // let pixmap = Arc::new(
    //     pixelflut::pixmap::ReplicatingPixmap::new(primary_pixmap, vec![Box::new(file_pixmap)], 0.2).unwrap(),
    // );
    let encodings = pixelflut::state_encoding::SharedMultiEncodings::default();
    let mut daemon_tasks: Vec<DaemonHandle> = Vec::new();

    #[cfg(feature = "framebuffer_gui")]
    if let Some(framebuffer_dev) = &opts.framebuffer {
        let framebuffer = pixelflut::framebuffer_gui::FramebufferGui::new(
            framebuffer_dev.to_owned(),
            interval(Duration::from_millis(1_000 / 40)),
        );
        let handle = framebuffer.start_render_task(pixmap.clone());
        daemon_tasks.push(handle)
    }

    #[cfg(feature = "gui")]
    if opts.show_gui {
        daemon_tasks.push(pixelflut::gui::start_gui(pixmap.clone()))
    }

    #[cfg(feature = "tcp_server")]
    if let Some(bind_addr) = &opts.tcp_bind_addr {
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        let server = pixelflut::net::servers::TcpServer::new(TcpServerOptions {
            bind_addr: bind_addr.to_owned(),
        });
        daemon_tasks.push(
            server
                .start(pixmap, encodings)
                .await
                .expect("Could not start tcp server"),
        );
    }

    #[cfg(feature = "udp_server")]
    if let Some(udp_bind_addr) = &opts.udp_bind_addr {
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        let server = UdpServer::new(UdpServerOptions {
            bind_addr: udp_bind_addr.to_owned(),
        });
        daemon_tasks.push(
            server
                .start(pixmap, encodings)
                .await
                .expect("Could not start udp server"),
        );
    }

    // #[cfg(feature = "ws_server")]
    // if let Some(ws_port) = &opts.ws_port {
    //     let pixmap = pixmap.clone();
    //     let encodings = encodings.clone();
    //     let (handle, _) = pixelflut::net::ws_server::start_listener(
    //         pixmap.clone(),
    //         encodings.clone(),
    //         pixelflut::net::ws_server::WsOptions {
    //             listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", ws_port))
    //                 .expect("could not build SocketAddr"),
    //         },
    //     );
    //     background_task_handles.push(handle);
    // }

    if daemon_tasks.is_empty() {
        panic!("No listeners are supposed to be started which makes no sense");
    }

    //let encoder_handles = pixelflut::state_encoding::start_encoders(encodings, pixmap);

    for handle in daemon_tasks {
        if let Err(e) = handle.join().await {
            tracing::error!("Error in background task: {:?}", e)
        }
    }
}

async fn start_client(opts: &cli::ClientOpts) {
    match (&opts.image, &opts.message) {
        (Some(image_path), None) => draw_image(opts).await,
        (None, Some(message)) => todo!(),
        _ => {
            tracing::error!("Either an image path or a message (but not both) must be passed as pixel source")
        }
    }
}

async fn draw_image(opts: &cli::ClientOpts) {
    let mut client = pixelflut::net::clients::UdpClient::connect(pixelflut::net::clients::UdpClientOptions {
        server_addr: format!("{}:{}", opts.host, opts.port)
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap(),
    })
    .await
    .expect("Could not connect pixelflut client to server");

    for x in 100usize..400 {
        for y in 100usize..400 {
            client
                .get_msg_writer()
                .write_request(&Request::SetPixel {
                    x,
                    y,
                    color: Color(0xFF, 0x00, 0x00),
                })
                .await
                .expect("Could not write pixel data")
        }
    }
}
