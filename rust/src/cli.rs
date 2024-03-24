use clap::builder::{PossibleValue, RangedU64ValueParser, TypedValueParser, ValueParserFactory};
use clap::error::ErrorKind;
use clap::{Arg, ArgAction, Args, Error, Parser, Subcommand};
use pixelflut::pixmap::Color;
use std::ffi::OsStr;
use std::net::SocketAddr;
use std::path::PathBuf;

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

/// Arguments common to all client commands
#[derive(Args, Debug, Clone)]
pub(crate) struct CommonClientOps {
    /// Address of the pixelflut server
    #[arg(long = "server")]
    pub server: SocketAddr,
    /// The width of the rectangle that should be drawn
    #[arg(long = "width", default_value = "fill")]
    pub width: TargetDimension,
    /// The height of the rectangle that should be drawn
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

impl ValueParserFactory for TargetDimension {
    type Parser = TargetDimensionParser;

    fn value_parser() -> Self::Parser {
        TargetDimensionParser
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct TargetDimensionParser;

impl TypedValueParser for TargetDimensionParser {
    type Value = TargetDimension;

    fn parse_ref(&self, cmd: &clap::Command, arg: Option<&Arg>, value: &OsStr) -> Result<Self::Value, Error> {
        let str_value = value.to_str().ok_or(cmd.clone().error(
            ErrorKind::InvalidValue,
            format!(
                "{} argument is neither 'fill' nor a number",
                arg.unwrap().get_id()
            ),
        ))?;

        if str_value.eq_ignore_ascii_case("fill") {
            Ok(TargetDimension::Fill)
        } else {
            RangedU64ValueParser::new()
                .parse_ref(cmd, arg, value)
                .map(|int_value: usize| TargetDimension::Specific(int_value))
        }
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        Some(Box::new(
            [PossibleValue::new("fill"), PossibleValue::new("<number>")].into_iter(),
        ))
    }
}

#[derive(Debug, Clone)]
pub(crate) enum TargetColor {
    RandomPerIteration,
    RandomOnce,
    Specific(Color),
}

impl ValueParserFactory for TargetColor {
    type Parser = TargetColorParser;

    fn value_parser() -> Self::Parser {
        TargetColorParser
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) struct TargetColorParser;

impl TypedValueParser for TargetColorParser {
    type Value = TargetColor;

    fn parse_ref(&self, cmd: &clap::Command, arg: Option<&Arg>, value: &OsStr) -> Result<Self::Value, Error> {
        let make_error = || {
            cmd.clone().error(
                ErrorKind::InvalidValue,
                format!(
                    "{} argument is neither 'random', 'random-per-iteration' nor a valid hex color",
                    arg.unwrap().get_id()
                ),
            )
        };

        let str_value = value.to_str().ok_or(make_error())?;

        if str_value.eq_ignore_ascii_case("random") {
            Ok(TargetColor::RandomOnce)
        } else if str_value.eq_ignore_ascii_case("random-per-iteration") {
            Ok(TargetColor::RandomPerIteration)
        } else {
            let color = u32::from_str_radix(str_value, 16).map_err(|_| make_error())?;
            Ok(TargetColor::Specific(color.into()))
        }
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        Some(Box::new(
            [
                PossibleValue::new("random"),
                PossibleValue::new("random-per-iteration"),
                PossibleValue::new("<hex-color>"),
            ]
            .into_iter(),
        ))
    }
}
