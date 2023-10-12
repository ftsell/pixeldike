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
use pixelflut::net::servers::{GenServer, TcpServerOptions, UdpServer, UdpServerOptions};
use pixelflut::pixmap::traits::*;
use pixelflut::pixmap::Color;
use pixelflut::{net, DaemonHandle};

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
    let mut background_task_handles: Vec<DaemonHandle> = Vec::new();

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
        let server = pixelflut::net::servers::TcpServer::new(TcpServerOptions {
            bind_addr: SocketAddr::from_str(&format!("0.0.0.0:{}", tcp_port))
                .expect("Could not build SOcketAddr for TcpServer binding"),
        });
        background_task_handles.push(
            server
                .start(pixmap, encodings)
                .await
                .expect("Could not start tcp server"),
        );
    }

    #[cfg(feature = "udp_server")]
    if let Some(udp_port) = &opts.udp_port {
        let pixmap = pixmap.clone();
        let encodings = encodings.clone();
        let server = UdpServer::new(UdpServerOptions {
            bind_addr: SocketAddr::from_str(&format!("0.0.0.0:{}", udp_port))
                .expect("Could not build SocketAddr for UdpServer binding"),
        });
        background_task_handles.push(
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

    for handle in background_task_handles {
        if let Err(e) = handle.join().await {
            tracing::error!("Error in background task: {:?}", e)
        }
    }
}

async fn start_client(opts: &cli::ClientOpts) {
    todo!("implement client")
}
