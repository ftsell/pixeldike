#![feature(never_type)]

use bytes::buf::Writer;
use bytes::{BufMut, BytesMut};
use clap::Parser;
use image::imageops::FilterType;
use rand::prelude::*;
use std::net::SocketAddr;
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
use pixeldike::net::clients::{GenClient, TcpClient};
use pixeldike::net::protocol::{Request, Response};
use pixeldike::net::servers::{GenServer, TcpServerOptions};
#[cfg(feature = "udp")]
use pixeldike::net::servers::{UdpServer, UdpServerOptions};
#[cfg(feature = "ws")]
use pixeldike::net::servers::{WsServer, WsServerOptions};
use pixeldike::pixmap::{Color, Pixmap};
use pixeldike::sinks::ffmpeg::{FfmpegOptions, FfmpegSink};
use pixeldike::sinks::framebuffer::{FramebufferSink, FramebufferSinkOptions};
use pixeldike::sinks::pixmap_file::{FileSink, FileSinkOptions};
use pixeldike::DaemonResult;

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
            let pixmap = pixeldike::sinks::pixmap_file::load_pixmap_file(path)
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

    if let Some(bind_addr) = &opts.tcp_bind_addr {
        let pixmap = pixmap.clone();
        let server = pixeldike::net::servers::TcpServer::new(TcpServerOptions {
            bind_addr: bind_addr.to_owned(),
        });
        server
            .start(pixmap, &mut join_set)
            .await
            .expect("Could not start tcp server");
    }

    #[cfg(feature = "udp")]
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

    #[cfg(feature = "ws")]
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

async fn put_rectangle(opts: &cli::PutRectangleData) {
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

    run_gen_client(
        fill_buf,
        &opts.common,
        matches!(opts.color, TargetColor::RandomPerIteration),
    )
    .await
}

async fn put_image(opts: &cli::PutImageData) {
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

    run_gen_client(fill_buf, &opts.common, false).await
}

/// Run a generic client loop that fills its command buffer from the provided function.
///
/// `fill_buf` should be a function that fills the provided buffer with pixelflut commands.
/// It is given `x_min, x_max, y_min, y_max` as additional arguments so that commands can be generated for the right
/// dimensions.
///
/// If `requires_buf_refresh` is true, then the command is filled per iteration of the client loop.
/// Otherwise it is only filled once.
async fn run_gen_client<F>(fill_buf: F, opts: &cli::CommonClientOps, requires_buf_refresh: bool)
where
    F: Fn(&mut Writer<BytesMut>, usize, usize, usize, usize),
{
    // preparation
    let mut px = make_client(opts.server).await;
    let (canvas_width, canvas_height) = get_size(&mut px).await;
    let (x_min, x_max, y_min, y_max) = calc_bounds(canvas_width, canvas_height, &opts);
    let mut buf = BytesMut::new().writer();

    tracing::info!("Preparing command buffer");
    fill_buf(&mut buf, x_min, x_max, y_min, y_max);

    // main loop
    tracing::info!("Running client loop");
    loop {
        // send whole buffer to server
        tracing::debug!("Sending prepared commands to server");
        px.get_writer()
            .write_all(buf.get_ref())
            .await
            .expect("Could not write commands to server");

        // abort loop if only one iteration is requested
        if !opts.do_loop {
            break;
        }

        // refresh buffer content if required
        if requires_buf_refresh {
            buf.get_mut().clear();
            fill_buf(&mut buf, x_min, x_max, y_min, y_max);
        }
    }
}

async fn make_client(addr: SocketAddr) -> TcpClient {
    tracing::info!("Connecting to pixelflut server {}", addr);
    TcpClient::connect(addr)
        .await
        .expect("Could not connect to pixelflut server")
}

async fn get_size(px: &mut TcpClient) -> (usize, usize) {
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
    (width, height)
}

/// Determine effective bounds from cli args as well as remote canvas size
///
/// Returns `(x_min, x_max, y_min, y_max)`
fn calc_bounds(
    canvas_width: usize,
    canvas_height: usize,
    opts: &cli::CommonClientOps,
) -> (usize, usize, usize, usize) {
    let x_min = if opts.x_offset >= canvas_width {
        panic!(
            "given x-offset {} is outside of servers canvas with width {}",
            opts.x_offset, canvas_width
        )
    } else {
        opts.x_offset
    };
    let y_min = if opts.y_offset >= canvas_height {
        panic!(
            "given y-offset {} is outside of servers canvas with height {}",
            opts.y_offset, canvas_height
        )
    } else {
        opts.y_offset
    };
    let x_max = match opts.width {
        TargetDimension::Fill => canvas_width,
        TargetDimension::Specific(width) => {
            if x_min + width >= canvas_width {
                panic!(
                    "given width {} combined with x-offset {} is outside of server canvas with width {}",
                    width, x_min, canvas_width
                );
            } else {
                x_min + width
            }
        }
    };
    let y_max = match opts.height {
        TargetDimension::Fill => canvas_height,
        TargetDimension::Specific(height) => {
            if y_min + height >= canvas_height {
                panic!(
                    "given height {} combined with y-offset {} is outside of server canvas with height {}",
                    height, y_min, canvas_height
                );
            } else {
                y_min + height
            }
        }
    };

    (x_min, x_max, y_min, y_max)
}
