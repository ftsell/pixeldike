#![feature(never_type)]

use clap::Parser;
use nom::Finish;
use rand::random;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::{JoinSet, LocalSet};
use tokio::time::interval;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use pixelflut::net::clients::{GenClient, TcpClient, TcpClientOptions};
use pixelflut::net::framing::MsgWriter;
use pixelflut::net::protocol::{parse_response, Request, Response};
use pixelflut::net::servers::{
    GenServer, TcpServerOptions, UdpServer, UdpServerOptions, WsServer, WsServerOptions,
};
use pixelflut::pixmap::{Color, Pixmap};
use pixelflut::sinks::ffmpeg::{FfmpegOptions, FfmpegSink};
use pixelflut::sinks::framebuffer::{FramebufferSink, FramebufferSinkOptions};
use pixelflut::sinks::pixmap_file::{FileSink, FileSinkOptions};
use pixelflut::DaemonResult;

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

    let local_set = LocalSet::new();
    local_set
        .run_until(async move {
            match &args.command {
                cli::Command::Server(opts) => start_server(&opts).await,
                cli::Command::PutImage(opts) => put_image(&opts).await,
            };
        })
        .await;
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

    let mut join_set: JoinSet<DaemonResult> = JoinSet::new();

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
        sink.start(&mut join_set)
            .await
            .expect("Could not start persistence task");
    }

    // configure gui window
    if opts.open_window {
        let pixmap = pixmap.clone();
        pixelflut::sinks::window::start(&mut join_set, pixmap)
            .expect("Could not open window for live rendering");
    }

    // configure streaming
    if opts.stream_opts.rtmp_dst_addr.is_some() || opts.stream_opts.rtsp_dst_addr.is_some() {
        // construct output spec depending on cli options
        let mut output_spec = Vec::new();
        if let Some(rtsp_dst_addr) = &opts.stream_opts.rtsp_dst_addr {
            output_spec.append(&mut FfmpegOptions::make_rtsp_out_spec(
                rtsp_dst_addr,
                opts.stream_opts.framerate,
            ));
        }
        if let Some(rtmp_dst_addr) = &opts.stream_opts.rtmp_dst_addr {
            output_spec.append(&mut FfmpegOptions::make_rtmp_out_spec(
                rtmp_dst_addr,
                opts.stream_opts.framerate,
            ));
        }

        // start the ffmpeg subprocess
        let pixmap = pixmap.clone();
        let ffmpeg = FfmpegSink::new(
            FfmpegOptions {
                framerate: opts.stream_opts.framerate,
                synthesize_audio: true,
                log_level: "warning".to_string(),
                output_spec,
            },
            pixmap,
        );
        ffmpeg
            .start(&mut join_set)
            .await
            .expect("Could not start ffmpeg sink");
    }

    // configure framebuffer rendering
    if let Some(fb_device) = &opts.fb_opts.fb_device {
        let pixmap = pixmap.clone();
        let sink = FramebufferSink::new(
            FramebufferSinkOptions {
                path: fb_device.to_owned(),
                framerate: opts.fb_opts.fb_framerate,
            },
            pixmap,
        );
        sink.start(&mut join_set)
            .await
            .expect("Coult not start task for framebuffer rendering");
    }

    if let Some(bind_addr) = &opts.tcp_bind_addr {
        let pixmap = pixmap.clone();
        let server = pixelflut::net::servers::TcpServer::new(TcpServerOptions {
            bind_addr: bind_addr.to_owned(),
        });
        server
            .start(pixmap, &mut join_set)
            .await
            .expect("Could not start tcp server");
    }

    if let Some(udp_bind_addr) = &opts.udp_bind_addr {
        let pixmap = pixmap.clone();
        let server = UdpServer::new(UdpServerOptions {
            bind_addr: udp_bind_addr.to_owned(),
        });
        server
            .start_many(pixmap, 16, &mut join_set)
            .await
            .expect("Could not start udp server");
    }

    if let Some(ws_bind_addr) = &opts.ws_bind_addr {
        let pixmap = pixmap.clone();
        let server = WsServer::new(WsServerOptions {
            bind_addr: ws_bind_addr.to_owned(),
        });
        server
            .start(pixmap, &mut join_set)
            .await
            .expect("Could not start websocket server");
    }

    // wait until one tasks exits
    let result = join_set
        .join_next()
        .await
        .expect("Nothing is supposed to be started which makes no sense. Review commandline flags.")
        .expect("Could not join background task")
        .unwrap_err();
    tracing::error!("A background task exited unexpectedly: {}", result);

    // cancel all other tasks
    join_set.shutdown().await;
}

async fn put_image(opts: &cli::PutImageOpts) {
    tracing::info!("Connecting to pixelflut server for metadata retrieval");
    let mut px = TcpClient::<128>::connect(TcpClientOptions {
        server_addr: opts.server,
    })
    .await
    .expect("Could not connect to pixelflut server over tcp");

    // get size from server
    px.get_msg_writer()
        .write_request(&Request::GetSize)
        .await
        .unwrap();
    px.get_msg_writer().flush().await.unwrap();
    let response = px.get_msg_reader().read_msg().await.unwrap();
    let response = parse_response(response).unwrap();
    let Response::Size { width, height } = response else {
        panic!("Server responded with invalid response: {response:?} to size request")
    };
    tracing::info!("Pixelflut server has canvas size {width}x{height}");

    loop {
        let color = Color::from((random(), random(), random()));
        tracing::info!("Drawing {color:X} onto the server…");

        for x in 0..width {
            for y in 0..height {
                px.get_msg_writer()
                    .write_request(&Request::SetPixel { x, y, color })
                    .await
                    .unwrap();
            }
        }
    }
}
