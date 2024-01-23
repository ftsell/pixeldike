//! A sink which pipes the canvas into ffmpeg for video encoding or streaming

use crate::pixmap::traits::PixmapRawRead;
use crate::pixmap::SharedPixmap;
use crate::DaemonHandle;
use anyhow::anyhow;
use std::io::Write;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

/// Configuration options of the ffmpeg subprocess
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FfmpegOptions {
    /// The level on which ffmpeg should emit logs.
    ///
    /// Valid values are 'quiet', 'panic', 'fatal', 'error', 'warning', 'info', 'verbose', 'debug', 'trace'
    pub log_level: String,

    /// How many frames per second should be emitted and which ffmpeg should target.
    pub framerate: usize,

    /// Whether an empty audio track should be synthesized.
    pub synthesize_audio: bool,

    /// Additional ffmpeg arguments that should be placed in the output part of the generated command.
    pub output_spec: Vec<String>,
}

impl FfmpegOptions {
    pub fn make_rtsp_out_spec(server_addr: String, framerate: usize) -> Vec<String> {
        vec![
            // set encoding to commonly supported variant
            "-vcodec".to_string(),
            "libx264".to_string(),
            "-acodec".to_string(),
            "aac".to_string(),
            // encode as quickly as possible
            "-preset".to_string(),
            "veryfast".to_string(),
            // disable b-frames since some players don't support them
            "-bf".to_string(),
            "0".to_string(),
            // set pixel format to a commonly supported one
            "-pix_fmt".to_string(),
            "yuv420p".to_string(),
            // set output frame rate
            "-framerate".to_string(),
            framerate.to_string(),
            // force output format to be rtsp
            "-f".to_string(),
            "rtsp".to_string(),
            // set output url to the given rtsp server
            server_addr,
        ]
    }
}

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
    pub fn new(options: FfmpegOptions, pixmap: SharedPixmap<P>) -> Self {
        Self {
            options,
            pixmap,
            ffmpeg_proc: None,
        }
    }

    pub async fn start(mut self) -> anyhow::Result<DaemonHandle> {
        self.start_ffmpeg()?;
        let handle = tokio::spawn(async move { self.run().await });
        Ok(DaemonHandle::new(handle))
    }

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
