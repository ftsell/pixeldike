use clap::{Args, Parser, Subcommand};
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
}

#[derive(Args, Debug, Clone)]
pub(crate) struct ServerOpts {
    /// port on which to start a tcp listener
    #[arg(long = "tcp")]
    #[cfg(feature = "tcp_server")]
    pub tcp_port: Option<u16>,

    /// port on which to start a udp listener
    #[arg(long = "udp")]
    #[cfg(feature = "udp_server")]
    pub udp_port: Option<u16>,

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
    #[arg(short = 'f', long = "file")]
    pub path: PathBuf,

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
