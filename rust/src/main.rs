use clap::Parser;
use embedded_graphics::mono_font::{MonoTextStyle, MonoTextStyleBuilder};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::Rectangle;
use image::io::Reader as ImageReader;
use image::Rgb;
use std::fmt::Debug;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tracing_subscriber;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Layer};

use pixelflut;

#[cfg(feature = "framebuffer_gui")]
use pixelflut::framebuffer_gui::FramebufferGui;
use pixelflut::net;
use pixelflut::net::tcp_client::TcpClient;
use pixelflut::net::MsgWriter;
use pixelflut::pixmap::traits::*;
use pixelflut::pixmap::Color;

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
    let mut background_task_handles: Vec<(JoinHandle<anyhow::Result<()>>, Arc<Notify>)> = Vec::new();

    #[cfg(feature = "framebuffer_gui")]
    let render_handle = match &opts.framebuffer {
        None => None,
        Some(fb_path) => Some(pixelflut::framebuffer_gui::start_gui_task(
            FramebufferGui::new(fb_path.to_owned()),
            pixmap.clone(),
        )),
    };

    // #[feature(gui)]
    // {
    //     let gui_handle = if opts.show_gui {
    //         let (handle, _proxy) = pixelflut::gui::GuiThread::start(pixmap.clone());
    //         Some(handle)
    //     } else {
    //         None
    //     };
    // }

    #[cfg(feature = "tcp_server")]
    if let Some(tcp_port) = &opts.tcp_port {
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        let (handle, stop) = pixelflut::net::tcp_server::start_listener(
            pixmap,
            encodings,
            pixelflut::net::tcp_server::TcpOptions {
                listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", tcp_port))
                    .expect("could not build SocketAddr"),
            },
        );
        background_task_handles.push((handle, stop));
    }

    #[cfg(feature = "udp_server")]
    if let Some(udp_port) = &opts.udp_port {
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        background_task_handles.extend(
            pixelflut::net::udp_server::start_listeners(
                pixmap,
                encodings,
                pixelflut::net::udp_server::UdpOptions {
                    listen_address: SocketAddr::from_str(&format!("0.0.0.0:{}", udp_port))
                        .expect("could not build SocketAddr"),
                    ..Default::default()
                },
            )
            .await,
        );
    }

    #[cfg(feature = "ws_server")]
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
        background_task_handles.push(handle);
    }

    if background_task_handles.len() == 0 {
        panic!("No listeners are supposed to be started which makes no sense");
    }

    //let encoder_handles = pixelflut::state_encoding::start_encoders(encodings, pixmap);

    //#[feature(gui)]
    //if let Some(handle) = gui_handle {
    //    handle.thread.await.expect("GUI seems to have crashed");
    //}

    //if let Some

    #[cfg(feature = "framebuffer_gui")]
    if let Some((handle, _)) = render_handle {
        let _ = tokio::join!(handle);
    }

    for (handle, _stop) in background_task_handles {
        let _ = tokio::join!(handle);
    }
    //for handle in encoder_handles {
    //    let _ = tokio::join!(handle.0);
    //}
}

async fn start_client(opts: &cli::ClientOpts) {
    match (&opts.image, &opts.message) {
        (Some(_), None) => draw_image(opts).await,
        (None, Some(message)) => draw_message(opts).await,
        (_, _) => {
            tracing::error!("Either a message or an image path (but not both) must be specified so that something can be drawn")
        }
    }
}

async fn draw_image(opts: &cli::ClientOpts) {
    println!("Reading image");
    let image = ImageReader::open(&opts.image.clone().unwrap())
        .expect("Could not open image")
        .decode()
        .expect("Could not decode image");
    println!("Scaling image to {}*{}", opts.width, opts.height);
    let image = image::imageops::resize(
        &image,
        opts.width as u32,
        opts.height as u32,
        image::imageops::FilterType::Gaussian,
    );

    println!("Connecting to server");
    let mut px_client = net::tcp_client::TcpClient::connect(&opts.addr)
        .await
        .expect("Could not connect to pixelflut server");

    println!("Writing image to server");
    for (x, y, color) in image.enumerate_pixels() {
        px_client
            .writer()
            .write_request(&pixelflut::net_protocol::Request::SetPixel {
                x: x as usize + opts.x_offset,
                y: y as usize + opts.y_offset,
                color: Color(color.0[0], color.0[1], color.0[1]),
            })
            .await
            .expect("Could not write pixel to server");
    }
    //px_client
    //    .flush()
    //    .await
    //    .expect("Could not flush the message stream to the server");
    println!("Done");
}

async fn draw_message(opts: &cli::ClientOpts) {
    println!("Connecting to server");
    let mut px_client = net::tcp_client::TcpClient::connect(&opts.addr)
        .await
        .expect("Could not connect to pixelflut server");

    let text = embedded_graphics::text::Text::new(
        &opts.message.clone().unwrap(),
        Point::new(opts.x_offset as i32, opts.y_offset as i32),
        MonoTextStyle::new(
            &embedded_graphics::mono_font::ascii::FONT_6X13_BOLD,
            Rgb565::BLACK,
        ),
    );

    todo!()
}
