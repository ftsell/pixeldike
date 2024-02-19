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
    PutImage(PutImageOpts),
}

#[derive(Args, Debug, Clone)]
pub(crate) struct ServerOpts {
    /// bind address on which a tcp server should be started
    #[arg(long = "tcp")]
    pub tcp_bind_addr: Option<SocketAddr>,

    /// bind address on which a tcp server should be started
    #[arg(long = "udp")]
    pub udp_bind_addr: Option<SocketAddr>,

    /// port on which to start a websocket listener
    #[arg(long = "ws")]
    pub ws_bind_addr: Option<SocketAddr>,

    /// width of the pixmap
    #[arg(short = 'x', long = "width", default_value = "800")]
    pub width: usize,

    /// height of the pixmap
    #[arg(short = 'y', long = "height", default_value = "600")]
    pub height: usize,

    #[command(flatten)]
    pub stream_opts: StreamOpts,

    #[command(flatten)]
    pub file_opts: FileOpts,

    #[command(flatten)]
    pub fb_opts: FramebufferOpts,

    #[arg(long = "open-window")]
    pub open_window: bool,
}

/// Specific options for sinking the pixmap data into something else (e.g. streaming it somewhere)
#[derive(Args, Debug, Clone)]
pub(crate) struct StreamOpts {
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

/// Specific options regarding snapshot files
#[derive(Args, Debug, Clone)]
pub(crate) struct FileOpts {
    /// A snapshot file from which the initial canvas content is loaded
    ///
    /// If the stored snapshot has different dimensions than the ones given via --width and --height, the snapshot is
    /// not loaded and an empty canvas is created instead.
    #[arg(long = "load-snapshot")]
    pub load_snapshot: Option<PathBuf>,

    /// A path into which snapshots are stored
    #[arg(long = "snapshot")]
    pub snapshot_file: Option<PathBuf>,

    /// The interval in seconds with which snapshots are written to disk
    #[arg(long = "snapshot-interval", default_value = "5")]
    pub snapshot_interval_secs: usize,
}

/// Specific options for rendering onto a framebuffer
#[derive(Args, Debug, Clone)]
pub(crate) struct FramebufferOpts {
    /// A framebuffer device onto which pixmap data should be rendered
    #[arg(long = "fb-device")]
    pub fb_device: Option<PathBuf>,

    /// The target framerate which the framebuffer rendering should target
    #[arg(long = "fb-framerate", default_value = "30")]
    pub fb_framerate: usize,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct PutImageOpts {
    /// Address of the pixelflut server
    #[arg(long = "server")]
    pub server: SocketAddr,
    // /// Path to an image that should be drawn
    // #[arg(long = "image")]
    // pub image: PathBuf,
}
