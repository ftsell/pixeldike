use clap::Parser;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use pixelflut::net::servers::{
    GenServer, TcpServerOptions, UdpServer, UdpServerOptions, WsServer, WsServerOptions,
};
use pixelflut::pixmap::Pixmap;
use pixelflut::sinks::ffmpeg::{FfmpegOptions, FfmpegSink};
use pixelflut::sinks::pixmap_file::{FileSink, FileSinkOptions};
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
        //cli::Command::Client(opts) => start_client(&opts).await,
        _ => unimplemented!(),
    };
}

async fn start_server(opts: &cli::ServerOpts) {
    // create a pixmap or load an existing snapshot
    let pixmap = match &opts.file_opts.load_snapshot {
        None => Arc::new(Pixmap::new(opts.width, opts.height).unwrap()),
        Some(path) => {
            let pixmap = pixelflut::sinks::pixmap_file::load_pixmap_file(path)
                .await
                .unwrap();
            let (width, height) = pixmap.get_size();
            if width != opts.width || height != opts.height {
                tracing::warn!(
                    "Stored snapshot has different dimensions than {}x{}, creating an empty pixmap instead",
                    opts.width,
                    opts.height
                );
                Arc::new(Pixmap::new(opts.width, opts.height).unwrap())
            } else {
                Arc::new(pixmap)
            }
        }
    };

    let mut daemon_tasks: Vec<DaemonHandle> = Vec::new();

    // configure snapshotting
    if let Some(path) = &opts.file_opts.snapshot_file {
        let pixmap = pixmap.clone();
        let sink = FileSink::new(
            FileSinkOptions {
                path: path.to_owned(),
                interval: interval(Duration::from_secs(opts.file_opts.snapshot_interval_secs as u64)),
            },
            pixmap,
        );
        daemon_tasks.push(sink.start().await.expect("Could not start snapshot task"))
    }

    // configure streaming
    if opts.sink_opts.rtmp_dst_addr.is_some() || opts.sink_opts.rtsp_dst_addr.is_some() {
        // construct output spec depending on cli options
        let mut output_spec = Vec::new();
        if let Some(rtsp_dst_addr) = &opts.sink_opts.rtsp_dst_addr {
            output_spec.append(&mut FfmpegOptions::make_rtsp_out_spec(
                rtsp_dst_addr,
                opts.sink_opts.framerate,
            ));
        }
        if let Some(rtmp_dst_addr) = &opts.sink_opts.rtmp_dst_addr {
            output_spec.append(&mut FfmpegOptions::make_rtmp_out_spec(
                rtmp_dst_addr,
                opts.sink_opts.framerate,
            ));
        }

        // start the ffmpeg subprocess
        let pixmap = pixmap.clone();
        let ffmpeg = FfmpegSink::new(
            FfmpegOptions {
                framerate: opts.sink_opts.framerate,
                synthesize_audio: true,
                log_level: "warning".to_string(),
                output_spec,
            },
            pixmap,
        );
        daemon_tasks.push(ffmpeg.start().await.expect("Could not start ffmpeg sink"));
    }

    #[cfg(feature = "tcp_server")]
    if let Some(bind_addr) = &opts.tcp_bind_addr {
        let pixmap = pixmap.clone();
        let server = pixelflut::net::servers::TcpServer::new(TcpServerOptions {
            bind_addr: bind_addr.to_owned(),
        });
        daemon_tasks.push(server.start(pixmap).await.expect("Could not start tcp server"));
    }

    #[cfg(feature = "udp_server")]
    if let Some(udp_bind_addr) = &opts.udp_bind_addr {
        let pixmap = pixmap.clone();
        let server = UdpServer::new(UdpServerOptions {
            bind_addr: udp_bind_addr.to_owned(),
        });
        daemon_tasks.extend(
            server
                .start_many(pixmap, 16)
                .await
                .expect("Could not start udp server"),
        );
    }

    #[cfg(feature = "ws_server")]
    if let Some(ws_bind_addr) = &opts.ws_bind_addr {
        let pixmap = pixmap.clone();
        let server = WsServer::new(WsServerOptions {
            bind_addr: ws_bind_addr.to_owned(),
        });
        daemon_tasks.push(
            server
                .start(pixmap)
                .await
                .expect("Could not start websocket server"),
        );
    }

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

// async fn start_client(opts: &cli::ClientOpts) {
//     match (&opts.image, &opts.message) {
//         (Some(image_path), None) => draw_image(opts).await,
//         (None, Some(message)) => todo!(),
//         _ => {
//             tracing::error!("Either an image path or a message (but not both) must be passed as pixel source")
//         }
//     }
// }
//
// async fn draw_image(opts: &cli::ClientOpts) {
//     let server_addr = format!("{}:{}", opts.host, opts.port)
//         .to_socket_addrs()
//         .unwrap()
//         .next()
//         .unwrap();
//     let mut tcp_client =
//         pixelflut::net::clients::TcpClient::<512>::connect(pixelflut::net::clients::TcpClientOptions {
//             server_addr,
//         })
//         .await
//         .expect("Could not connect pixelflut client (tcp) to server");
//
//     // fetch config from server
//     tcp_client
//         .get_msg_writer()
//         .write_request(&Request::GetConfig)
//         .await
//         .unwrap();
//     let server_config = tcp_client.get_msg_reader().read_msg().await.unwrap();
//     let (_, server_config) = parse_response(server_config).finish().unwrap();
//     let server_config = server_config.to_owned();
//
//     // fetch size from server
//     tcp_client
//         .get_msg_writer()
//         .write_request(&Request::GetSize)
//         .await
//         .unwrap();
//     let size = tcp_client.get_msg_reader().read_msg().await.unwrap();
//     let (_, size) = parse_response(size).finish().unwrap();
//     let size = size.to_owned();
//
//     tracing::info!(
//         "Connected to server with canvas {:?} and {:?}",
//         size,
//         server_config
//     );
//
//     loop {
//         for x in 0..opts.width {
//             for y in 0..opts.height {
//                 let msg = Request::SetPixel {
//                     x: x + opts.x_offset,
//                     y: y + opts.y_offset,
//                     color: Color(0xFF, 0x00, 0x00),
//                 };
//                 tcp_client.get_msg_writer().write_request(&msg).await.unwrap();
//             }
//         }
//     }
// }
