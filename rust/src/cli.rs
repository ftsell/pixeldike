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
    pub ws_bind_addr: Option<SocketAddr>,

    /// width of the pixmap
    #[arg(short = 'x', long = "width", default_value = "800")]
    pub width: usize,

    /// height of the pixmap
    #[arg(short = 'y', long = "height", default_value = "600")]
    pub height: usize,

    #[command(flatten)]
    pub sink_opts: SinkOpts,
}

/// Specific options for sinking the pixmap data into something else (e.g. streaming it somewhere)
#[derive(Args, Debug, Clone)]
pub(crate) struct SinkOpts {
    /// An RTMP url to which pixmap data should be streamed
    ///
    /// Must be in a form understood by ffmpeg i.e. `rtmp://[username:password@]server[:port][/app][/instance][/playpath]`
    #[arg(long = "rtmp-stream")]
    pub rtmp_dst_addr: Option<String>,

    /// An RTSP url to which pixmap data should be streamed
    ///
    /// Must be in a form understood by ffmpeg i.e. `rtsp://hostname[:port]/path`
    #[arg(long = "rtsp-stream")]
    pub rtsp_dst_addr: Option<String>,

    /// The target framerate with which the pixmap stream should be emitted
    #[arg(long = "stream-framerate", default_value = "30")]
    pub framerate: usize,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct ClientOpts {
    /// Hostname of the server to connect to
    #[arg(long = "host")]
    pub host: String,

    /// Port to connect to on the server
    #[arg(long = "port", default_value = "1234")]
    pub port: u16,

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
