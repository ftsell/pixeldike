#![feature(never_type)]

use bytes::buf::Writer;
use bytes::{BufMut, BytesMut};
use clap::Parser;
use image::imageops::FilterType;
use rand::prelude::*;
use std::net::ToSocketAddrs;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::task::{JoinSet, LocalSet};
use tokio::time::interval;
use tracing::metadata::LevelFilter;
use tracing_subscriber::filter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

use crate::cli::{CliOpts, TargetColor, TargetDimension};
use image::io::Reader as ImageReader;
use itertools::Itertools;
use pixeldike::net::clients::{TcpClient, UdpClient, UnixSocketClient};
use pixeldike::net::protocol::{Request, Response};
use pixeldike::net::servers::{GenServer, TcpServer, TcpServerOptions, UnixSocketOptions, UnixSocketServer};
#[cfg(feature = "udp")]
use pixeldike::net::servers::{UdpServer, UdpServerOptions};
#[cfg(feature = "ws")]
use pixeldike::net::servers::{WsServer, WsServerOptions};
use pixeldike::pixmap::{Color, Pixmap};
use pixeldike::sinks::ffmpeg::{FfmpegOptions, FfmpegSink};
use pixeldike::sinks::framebuffer::{FramebufferSink, FramebufferSinkOptions};
use pixeldike::sinks::pixmap_file::{FileSink, FileSinkOptions};
use pixeldike::DaemonResult;
use url::Url;

mod cli;
mod main_utils;

#[tokio::main]
async fn main() {
    let args = cli::CliOpts::parse();
    init_logger(&args);

    // prepare async environment and run the specified program action
    let local_set = LocalSet::new();
    local_set
        .run_until(async move {
            match &args.command {
                cli::Command::Server(opts) => start_server(opts).await,
                cli::Command::PutRectangle(opts) => put_rectangle(opts).await,
                cli::Command::PutImage(opts) => put_image(opts).await,
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
        .init();
}

async fn start_server(opts: &cli::ServerOpts) {
    // create a pixmap or load an existing snapshot
    let pixmap = match &opts.file_opts.load_snapshot {
        None => Arc::new(Pixmap::new(opts.width, opts.height).unwrap()),
        Some(path) => {
            let loaded_pixmap = pixeldike::sinks::pixmap_file::load_pixmap_file(path).await;
            match loaded_pixmap {
                Err(e) => {
                    tracing::error!(
                        "Could not load snapshot from {}, using empty pixmap instead: {}",
                        path.display(),
                        e
                    );
                    Arc::new(Pixmap::new(opts.width, opts.height).unwrap())
                }
                Ok(loaded_pixmap) => {
                    let (width, height) = loaded_pixmap.get_size();
                    if width != opts.width || height != opts.height {
                        tracing::warn!(
                    "Stored snapshot has different dimensions than {}x{}, creating an empty pixmap instead",
                    opts.width,
                    opts.height
                );
                        Arc::new(Pixmap::new(opts.width, opts.height).unwrap())
                    } else {
                        Arc::new(loaded_pixmap)
                    }
                }
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
    #[cfg(feature = "windowing")]
    if opts.open_window {
        let pixmap = pixmap.clone();
        pixeldike::sinks::window::start(&mut join_set, pixmap)
            .expect("Could not open window for live rendering");
    }

    // configure streaming sink
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

    // configure framebuffer sink
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

    // configure and start all servers
    for url in &opts.listen {
        match url.scheme() {
            #[cfg(feature = "tcp")]
            "tcp" => {
                if !url.username().is_empty() {
                    tracing::warn!(
                        "{} listen directive specifies credentials which is not supported by the TCP server",
                        url
                    )
                }
                if !url.path().is_empty() {
                    tracing::warn!(
                        "{} listen directive specifies a path which is not supported by the TCP server",
                        url
                    );
                }
                for bind_addr in (url.host_str().unwrap(), url.port().unwrap_or(1234))
                    .to_socket_addrs()
                    .expect("Could not resolve socket addr from listener url")
                {
                    TcpServer::new(TcpServerOptions { bind_addr })
                        .start(pixmap.clone(), &mut join_set)
                        .await
                        .expect(&format!("Could not start tcp server on {}", url));
                }
            }
            "unix" => {
                let path = PathBuf::from_str(url.path()).expect("Could not turn url path into system path");
                UnixSocketServer::new(UnixSocketOptions { path })
                    .start(pixmap.clone(), &mut join_set)
                    .await
                    .expect(&format!("Could not start unix socket listener on {}", url));
            }
            #[cfg(feature = "udp")]
            "udp" => {
                if !url.username().is_empty() {
                    tracing::info!("{}", url.authority());
                    tracing::warn!(
                        "{} listen directive specifies credentials which is not supported by the UDP server",
                        url
                    )
                }
                if !url.path().is_empty() {
                    tracing::warn!(
                        "{} listen directive specifies a path which is not supported by the UDP server",
                        url
                    );
                }
                for bind_addr in (url.host_str().unwrap(), url.port().unwrap_or(1234))
                    .to_socket_addrs()
                    .expect("Could not resolve socket addr from listener url")
                {
                    UdpServer::new(UdpServerOptions { bind_addr })
                        .start(pixmap.clone(), &mut join_set)
                        .await
                        .expect(&format!("Could not start tcp server on {}", url));
                }
            }
            #[cfg(feature = "ws")]
            "ws" => {
                if !url.username().is_empty() {
                    tracing::info!("{}", url.authority());
                    tracing::warn!(
                        "{} listen directive specifies credentials which is not supported by the WebSocket server",
                        url
                    )
                }
                if url.path() != "/" {
                    tracing::warn!(
                        "{} listen directive specifies a path which is not supported by the WebSocket server. The WebSocket is instead available on all paths.",
                        url
                    );
                }

                for bind_addr in (url.host_str().unwrap(), url.port().unwrap_or(1235))
                    .to_socket_addrs()
                    .expect("Could not resolve socket addr from listener url")
                {
                    WsServer::new(WsServerOptions { bind_addr })
                        .start(pixmap.clone(), &mut join_set)
                        .await
                        .expect(&format!("Could not start tcp server on {}", url));
                }
            }
            proto => {
                panic!("Unsupported server protocol {}", proto);
            }
        }
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

async fn put_rectangle(opts: &cli::PutRectangleData) {
    // define how a request buffer is filled
    let fill_buf = |buf: &mut Writer<BytesMut>, x_min: usize, x_max: usize, y_min: usize, y_max: usize| {
        // select a color
        let color = match opts.color {
            TargetColor::RandomPerIteration | TargetColor::RandomOnce => {
                Color::from((random(), random(), random()))
            }
            TargetColor::Specific(c) => c,
        };

        // accumulate color commands into one large buffer buffer
        tracing::debug!("Filling command-buffer to draw #{color:X} from {x_min},{y_min} to {x_max},{y_max}");
        let mut coords = (x_min..x_max).cartesian_product(y_min..y_max).collect::<Vec<_>>();
        coords.shuffle(&mut thread_rng());
        for (x, y) in coords {
            Request::SetPixel { x, y, color }.write(buf).unwrap();
        }
    };

    // run main client loop
    main_utils::DynClient::connect(&opts.common.server)
        .await
        .expect("Could not connect to pixelflut server")
        .run_loop(
            fill_buf,
            &opts.common,
            matches!(opts.color, TargetColor::RandomPerIteration),
        )
        .await;
}

async fn put_image(opts: &cli::PutImageData) {
    // define how a request buffer is filled
    let fill_buf = |buf: &mut Writer<BytesMut>, x_min: usize, x_max: usize, y_min: usize, y_max: usize| {
        tracing::debug!("Opening image at {}", &opts.path.display());
        let img = ImageReader::open(&opts.path)
            .expect("Could not open image file")
            .decode()
            .expect("Could not decode image")
            .to_rgb8();

        tracing::debug!("Resizing image to dimensions {}x{}", x_max - x_min, y_max - y_min);
        let img = image::imageops::resize(
            &img,
            (x_max - x_min) as u32,
            (y_max - y_min) as u32,
            FilterType::Triangle,
        );

        // accumulate color commands into one large buffer buffer
        tracing::debug!("Converting image to pixelflut commands");
        let mut coords = (x_min..x_max).cartesian_product(y_min..y_max).collect::<Vec<_>>();
        coords.shuffle(&mut thread_rng());
        for (x, y) in coords {
            let color = img.get_pixel(x as u32, y as u32);
            Request::SetPixel {
                x,
                y,
                color: color.0.into(),
            }
            .write(buf)
            .unwrap();
        }
    };

    // run main client loop
    main_utils::DynClient::connect(&opts.common.server)
        .await
        .expect("Could not connect to pixelflut server")
        .run_loop(fill_buf, &opts.common, false)
        .await;
}
