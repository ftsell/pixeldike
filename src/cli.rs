use clap::{ArgAction, Args, Parser, Subcommand};
use pixeldike::pixmap::Color;
use std::path::PathBuf;
use std::str::FromStr;
use url::Url;

/// Command-Line arguments as a well formatted struct, parsed using clap.
#[derive(Parser, Debug, Clone)]
#[command(author, version, about)]
pub(crate) struct CliOpts {
    #[command(subcommand)]
    pub command: Command,

    /// Increase program verbosity
    ///
    /// The default verbosity level is INFO.
    #[arg(short = 'v', long = "verbose", action = ArgAction::Count, default_value = "0")]
    pub verbose: u8,

    /// Decrease program verbosity
    ///
    /// The default verbosity level is INFO.
    #[arg(short = 'q', long = "quiet", action = ArgAction::Count, default_value = "0")]
    pub quiet: u8,
}

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum Command {
    /// Start a pixelflut server
    Server(ServerOpts),
    /// Run a pixelflut client to project a colored rectangle onto a servers pixmap
    PutRectangle(PutRectangleData),
    /// Upload an image to a pixelflut server
    PutImage(PutImageData),
}

#[derive(Args, Debug, Clone)]
pub(crate) struct ServerOpts {
    /// Url on which to bind a server
    ///
    /// Valid protocols are "tcp://", "udp://" and "ws://".
    #[arg(long = "listen")]
    pub listen: Vec<Url>,

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

    #[cfg(feature = "windowing")]
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

/// Arguments common to all client commands
#[derive(Args, Debug, Clone)]
pub(crate) struct CommonClientOps {
    /// Address of the pixelflut server
    #[arg(short = 's', long = "server")]
    pub server: Url,
    /// The width of the rectangle that should be drawn
    ///
    /// Possible values: ["fill", <number>]
    #[arg(long = "width", default_value = "fill")]
    pub width: TargetDimension,
    /// The height of the rectangle that should be drawn
    ///
    /// Possible values: ["fill", <number>]
    #[arg(long = "height", default_value = "fill")]
    pub height: TargetDimension,
    /// Offset from the left of the canvas edge to start drawing
    #[arg(short = 'x', default_value = "0")]
    pub x_offset: usize,
    /// Offset from the top of the canvas to start drawing
    #[arg(short = 'y', default_value = "0")]
    pub y_offset: usize,
    /// Only draw the rectangle once
    #[arg(long = "once", action = ArgAction::SetFalse)]
    pub do_loop: bool,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct PutRectangleData {
    #[command(flatten)]
    pub common: CommonClientOps,

    /// The color which the rectangle should have.
    #[arg(long = "color", default_value = "random")]
    pub color: TargetColor,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct PutImageData {
    #[command(flatten)]
    pub common: CommonClientOps,

    /// Path to an image file that should be uploaded
    #[arg(short = 'f', long = "file")]
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub(crate) enum TargetDimension {
    /// Fill all available space
    Fill,
    /// Fill a specific number of bytes
    Specific(usize),
}

impl FromStr for TargetDimension {
    type Err = <usize as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("fill") {
            Ok(TargetDimension::Fill)
        } else {
            let v = usize::from_str(s)?;
            Ok(TargetDimension::Specific(v))
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum TargetColor {
    RandomPerIteration,
    RandomOnce,
    Specific(Color),
}

impl FromStr for TargetColor {
    type Err = <u32 as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq_ignore_ascii_case("random") {
            Ok(TargetColor::RandomOnce)
        } else if s.eq_ignore_ascii_case("random-per-iteration") {
            Ok(TargetColor::RandomPerIteration)
        } else {
            let color = u32::from_str_radix(s, 16)?;
            Ok(TargetColor::Specific(color.into()))
        }
    }
}
