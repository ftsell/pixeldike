use clap::Parser;
use nom::Finish;
use std::net::ToSocketAddrs;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use pixelflut::net::clients::GenClient;
use pixelflut::net::framing::MsgWriter;
use pixelflut::net::protocol::{parse_response, Request};
use pixelflut::net::servers::{GenServer, TcpServerOptions, UdpServer, UdpServerOptions};
use pixelflut::pixmap::Color;
use pixelflut::DaemonHandle;

mod cli;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            EnvFilter::builder()
                .parse("debug,wgpu_core=warn,wgpu_hal=warn,naga=warn")
                .unwrap(),
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
        daemon_tasks.extend(
            server
                .start_many(pixmap, encodings, 32)
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
    let server_addr = format!("{}:{}", opts.host, opts.port)
        .to_socket_addrs()
        .unwrap()
        .next()
        .unwrap();
    let mut tcp_client =
        pixelflut::net::clients::TcpClient::<512>::connect(pixelflut::net::clients::TcpClientOptions {
            server_addr,
        })
        .await
        .expect("Could not connect pixelflut client (tcp) to server");

    // fetch config from server
    tcp_client
        .get_msg_writer()
        .write_request(&Request::GetConfig)
        .await
        .unwrap();
    let server_config = tcp_client.get_msg_reader().read_msg().await.unwrap();
    let (_, server_config) = parse_response(server_config).finish().unwrap();
    let server_config = server_config.to_owned();

    // fetch size from server
    tcp_client
        .get_msg_writer()
        .write_request(&Request::GetSize)
        .await
        .unwrap();
    let size = tcp_client.get_msg_reader().read_msg().await.unwrap();
    let (_, size) = parse_response(size).finish().unwrap();
    let size = size.to_owned();

    tracing::info!(
        "Connected to server with canvas {:?} and {:?}",
        size,
        server_config
    );

    loop {
        for x in 0..opts.width {
            for y in 0..opts.height {
                let msg = Request::SetPixel {
                    x: x + opts.x_offset,
                    y: y + opts.y_offset,
                    color: Color(0xFF, 0x00, 0x00),
                };
                tcp_client.get_msg_writer().write_request(&msg).await.unwrap();
            }
        }
    }
}
