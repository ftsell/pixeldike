#![feature(never_type)]

use bytes::{BufMut, BytesMut};
use clap::Parser;
use itertools::Itertools;
use rand::prelude::*;
use rand::{random, thread_rng};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::task::{JoinSet, LocalSet};
use tokio::time::interval;
use tracing::metadata::LevelFilter;
use tracing_subscriber::filter;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::cli::CliOpts;
use pixelflut::net::clients::{GenClient, TcpClient};
use pixelflut::net::protocol::{Request, Response};
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
    let args = cli::CliOpts::parse();
    init_logger(&args);

    // prepare async environment and run the specified program action
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

#[inline]
fn init_logger(args: &CliOpts) {
    // determine combined log level from cli arguments
    const DEFAULT_LEVEL: u8 = 3;
    let log_level = match DEFAULT_LEVEL
        .saturating_add(args.verbose)
        .saturating_sub(args.quiet)
    {
        0 => LevelFilter::OFF,
        1 => LevelFilter::ERROR,
        2 => LevelFilter::WARN,
        3 => LevelFilter::INFO,
        4 => LevelFilter::DEBUG,
        _ => LevelFilter::TRACE,
    };

    // configure appropriate level filter
    // tokio is very spammy on higher log levels which is usually not interesting so we filter it out
    let filter = filter::Targets::new()
        .with_default(log_level)
        .with_target("tokio", Ord::min(LevelFilter::WARN, log_level))
        .with_target("runtime", Ord::min(LevelFilter::WARN, log_level));
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(filter)
        .with(log_level)
        .with(
            EnvFilter::builder()
                .parse("trace,tokio=warn,runtime=warn")
                .unwrap(),
        )
        .init();
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
    tracing::info!("Connecting to pixelflut server {}", opts.server);
    let mut px = TcpClient::connect(opts.server)
        .await
        .expect("Could not connect to pixelflut server");

    // retrieve size metadata
    let Response::Size { width, height } = px
        .exchange(Request::GetSize)
        .await
        .expect("Could not retrieve size from pixelflut server")
    else {
        panic!("Server sent invalid response to size request")
    };
    tracing::info!(
        "Successfully exchanged metadata with pixelflut server (width={}, height={})",
        width,
        height
    );

    let mut buf = BytesMut::new().writer();
    loop {
        // send random color and fill screen with it
        let color = Color::from((random(), random(), random()));
        tracing::info!("Drawing {color:X} onto the server…");

        // accumulate color commands into one large buffer buffer
        let mut coords = (0..width).cartesian_product(0..height).collect::<Vec<_>>();
        coords.shuffle(&mut thread_rng());
        for (x, y) in coords {
            Request::SetPixel { x, y, color }.write(&mut buf).unwrap();
        }

        // send whole buffer to server
        px.get_writer()
            .write_all_buf(buf.get_mut())
            .await
            .expect("Could not write commands to server");
    }
}
