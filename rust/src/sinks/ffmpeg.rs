//! A sink which pipes the canvas into ffmpeg for video encoding or streaming

use crate::pixmap::traits::PixmapRawRead;
use crate::pixmap::SharedPixmap;
use crate::DaemonHandle;
use anyhow::anyhow;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

/// Configuration options of the ffmpeg sink
///
/// Some ffmpeg options are included in this struct as direct parameters but since output selection and encoding
/// configuration is very complex, it is done via the `output_spec` field which holds arguments that are passed directly
/// to ffmpeg.
/// To simplify the construction of common output specs, helper functions are available.
///
/// ## Examples
///
/// To stream to an rtsp server with 10fps:
///
/// ```rust
/// # use pixelflut::sinks::ffmpeg::FfmpegOptions;
///
/// const FPS: usize = 10;
/// let options = FfmpegOptions {
///     framerate: FPS,
///     synthesize_audio: true,
///     log_level: "warning".to_string(),
///     output_spec: FfmpegOptions::make_rtsp_out_spec("rtsp://localhost:8554/pixelflut", FPS)
/// };
/// ```
///
/// Stream to an RTSP and RTMP server simultaneously:
///
/// ```rust
/// # use pixelflut::sinks::ffmpeg::FfmpegOptions;
///
/// const FPS: usize = 10;
/// let options = FfmpegOptions {
///     framerate: FPS,
///     synthesize_audio: true,
///     log_level: "warning".to_string(),
///     output_spec: [
///         FfmpegOptions::make_rtsp_out_spec("rtsp://localhost:8554/pixelflut", FPS),
///         FfmpegOptions::make_rtmp_out_spec("rtmp://localhost:1935/pixelflut2", FPS),
///     ]
///     .into_iter()
///     .flatten()
///     .collect()
/// };
/// ```
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FfmpegOptions {
    /// The level on which ffmpeg should emit logs.
    ///
    /// Valid values are 'quiet', 'panic', 'fatal', 'error', 'warning', 'info', 'verbose', 'debug', 'trace'
    pub log_level: String,

    /// How many frames per second should be emitted.
    pub framerate: usize,

    /// Whether an empty audio track should be synthesized.
    ///
    /// **Note:** While strictly speaking an audio track is not required since pixelflut only consists of image data,
    /// some viewers won't display the video data if there is no audio component present.
    pub synthesize_audio: bool,

    /// Additional ffmpeg arguments that should be placed in the output part of the generated command.
    pub output_spec: Vec<String>,
}

impl FfmpegOptions {
    /// Create vector of ffmpeg arguments that are suitable for streaming to an [RTSP](https://en.wikipedia.org/wiki/Real-Time_Streaming_Protocol) server.
    ///
    /// The `server_addr` is required to be in `rtsp://hostname[:port]/path` format while `framerate` sets the targeted
    /// framerate of the stream.
    pub fn make_rtsp_out_spec(server_addr: &str, framerate: usize) -> Vec<String> {
        [
            // set encoding to commonly supported variant
            "-vcodec",
            "libx264",
            "-acodec",
            "aac",
            // encode as quickly as possible
            "-preset",
            "veryfast",
            // disable b-frames since some players don't support them
            "-bf",
            "0",
            // set pixel format to a commonly supported one
            "-pix_fmt",
            "yuv420p",
            // set output frame rate
            "-framerate",
            &framerate.to_string(),
            // force output format to be rtsp
            "-f",
            "rtsp",
            // set output url to the given rtsp server
            server_addr,
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }

    /// Create a vector of ffmpeg arguments that are suitable for streaming to an [RTMP](https://en.wikipedia.org/wiki/Real-Time_Messaging_Protocol) server.
    pub fn make_rtmp_out_spec(server_addr: &str, framerate: usize) -> Vec<String> {
        [
            // set encoding to commonly supported variant
            "-vcodec",
            "libx264",
            "-acodec",
            "aac",
            // encode as quickly as possible
            "-preset",
            "veryfast",
            // disable b-frames since some players don't support them
            "-bf",
            "0",
            // set pixel format to a commonly supported one
            "-pix_fmt",
            "yuv420p",
            // set output frame rate
            "-framerate",
            &framerate.to_string(),
            // force output format to be flv which is commonly used over rtmp
            "-f",
            "flv",
            server_addr,
        ]
        .into_iter()
        .map(String::from)
        .collect()
    }
}

/// A sink that puts pixmap data into an ffmpeg subprocess
#[derive(Debug)]
pub struct FfmpegSink<P: PixmapRawRead> {
    options: FfmpegOptions,
    pixmap: SharedPixmap<P>,
    ffmpeg_proc: Option<Child>,
}

impl<P> FfmpegSink<P>
where
    P: PixmapRawRead + Send + Sync + 'static,
{
    /// Create a new ffmpeg sink which sinks data from the given pixmap into an ffmpeg child process
    pub fn new(options: FfmpegOptions, pixmap: SharedPixmap<P>) -> Self {
        Self {
            options,
            pixmap,
            ffmpeg_proc: None,
        }
    }

    /// Span the ffmpeg child process and start sinking data into it
    pub async fn start(mut self) -> anyhow::Result<DaemonHandle> {
        self.start_ffmpeg()?;
        let handle = tokio::spawn(async move { self.run().await });
        Ok(DaemonHandle::new(handle))
    }

    /// Start the ffmpeg child process
    fn start_ffmpeg(&mut self) -> anyhow::Result<()> {
        if self.ffmpeg_proc.is_some() {
            return Err(anyhow!("ffmpeg is already running"));
        }

        let (width, height) = self.pixmap.get_size()?;

        let mut cmd = Command::new("ffmpeg");
        cmd.stdin(Stdio::piped()).kill_on_drop(true).env_clear();

        // Global Options
        cmd.arg("-hide_banner")
            .arg("-loglevel")
            .arg(&self.options.log_level);

        // Video Input Options
        cmd
            // treat input framerate as fixed and don't buffer it
            .arg("-re")
            // specify input encoding as raw image data in rgb encoding
            .arg("-f")
            .arg("rawvideo")
            .arg("-pix_fmt")
            .arg("rgb24")
            // provide metadata since it is not included in the rawvideo format
            .arg("-video_size")
            .arg(&format!("{}x{}", width, height))
            .arg("-framerate")
            .arg(&self.options.framerate.to_string())
            // tell ffmpeg that it should read input from stdin
            .arg("-i")
            .arg("/dev/stdin");

        // Audio Input Options
        if self.options.synthesize_audio {
            cmd.arg("-f")
                .arg("lavfi")
                .arg("-i")
                .arg("anullsrc=channel_layout=stereo:sample_rate=44100");
        }

        // add output args
        cmd.args(&self.options.output_spec);

        tracing::info!("starting ffmpeg sink with args {:?}", cmd.as_std().get_args());

        self.ffmpeg_proc = Some(cmd.spawn()?);
        Ok(())
    }

    /// Execute the main loop which periodically sinks data into ffmpeg
    async fn run(self) -> anyhow::Result<!> {
        let mut ffmpeg = self.ffmpeg_proc.ok_or(anyhow!("ffmpeg is not running"))?;
        let Some(channel) = &mut ffmpeg.stdin else {
            return Err(anyhow!("ffmpegs stdin is not attached"));
        };

        let mut interval =
            tokio::time::interval(Duration::from_secs_f64(1.0 / self.options.framerate as f64));

        loop {
            let data = self.pixmap.get_raw_data()?;
            let raw_data = data
                .iter()
                .flat_map(|c| Into::<[u8; 3]>::into(*c))
                .collect::<Vec<_>>();
            channel
                .write_all(&raw_data)
                .await
                .expect("Could not write to ffmpeg");

            interval.tick().await;
        }
    }
}
