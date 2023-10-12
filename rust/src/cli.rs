use clap::{Args, Parser, Subcommand};
use std::net::SocketAddr;
use std::path::PathBuf;

/// Command-Line arguments as a well formatted struct, parsed using clap.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub(crate) struct CliOpts {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Command {
    /// Start a pixelflut server
    Server(ServerOpts),
    /// Run a pixelflut client to project an image onto a servers pixmap
    Client(ClientOpts),
}

#[derive(Args, Debug, Clone)]
pub(crate) struct ServerOpts {
    /// bind address on which a tcp server should be started
    #[arg(long = "tcp")]
    #[cfg(feature = "tcp_server")]
    pub tcp_bind_addr: Option<SocketAddr>,

    /// bind address on which a tcp server should be started
    #[arg(long = "udp")]
    #[cfg(feature = "udp_server")]
    pub udp_bind_addr: Option<SocketAddr>,

    /// port on which to start a websocket listener
    #[arg(long = "ws")]
    #[cfg(feature = "ws_server")]
    pub ws_port: Option<u16>,

    /// width of the pixmap
    #[arg(short = 'x', long = "width", default_value = "800")]
    pub width: usize,

    /// height of the pixmap
    #[arg(short = 'y', long = "height", default_value = "600")]
    pub height: usize,

    /// file path into which the pixmap is persisted and from which it is read on startup
    //#[arg(short = 'f', long = "file")]
    //pub path: PathBuf,

    /// whether a gui should be shown
    #[arg(long = "gui")]
    #[cfg(feature = "gui")]
    pub show_gui: bool,

    #[arg(
        long = "framebuffer",
        help = "path to the framebuffer device on which the pixmap is live-rendered"
    )]
    #[cfg(feature = "framebuffer_gui")]
    pub framebuffer: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct ClientOpts {
    /// Address of the server to connect to
    #[arg(long = "addr")]
    pub addr: SocketAddr,

    /// Width of the area to draw
    #[arg(long = "width")]
    pub width: usize,

    /// Height of the area to draw
    #[arg(long = "height")]
    pub height: usize,

    /// Offset in the x dimension to apply before drawing
    #[arg(short = 'x')]
    pub x_offset: usize,

    /// Offset in the y dimension to apply before drawing
    #[arg(short = 'y')]
    pub y_offset: usize,

    /// Path to an image that should be drawn
    #[arg(long = "image")]
    pub image: Option<PathBuf>,

    /// A text message that should be drawn
    #[arg(long = "message")]
    pub message: Option<String>,
}
