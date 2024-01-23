//! Streaming of pixelflut data to an RTSP server
//!
//! This is implemented by launching an ffmpeg subprocess which does the encoding and sending.

use crate::pixmap::traits::PixmapRawRead;
use crate::pixmap::SharedPixmap;
use crate::DaemonHandle;
use anyhow::anyhow;
use std::io::Write;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, Command};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RtspOptions {
    /// Address of the rtmp server to which a video stream should be sent
    pub server_addr: String,
    pub fps: u8,
}

#[derive(Debug)]
pub struct RtspStream<P>
where
    P: PixmapRawRead,
{
    options: RtspOptions,
    pixmap: SharedPixmap<P>,
    ffmpeg: Option<Child>,
}

impl<P> RtspStream<P>
where
    P: PixmapRawRead + Send + Sync + 'static,
{
    pub fn new(options: RtspOptions, pixmap: SharedPixmap<P>) -> Self {
        Self {
            options,
            pixmap,
            ffmpeg: None,
        }
    }

    pub async fn start(mut self) -> anyhow::Result<DaemonHandle> {
        self.start_ffmpeg()?;
        let handle = tokio::spawn(async move { self.run().await });
        Ok(DaemonHandle::new(handle))
    }

    /// Start the ffmpeg subprocess
    fn start_ffmpeg(&mut self) -> anyhow::Result<()> {
        if self.ffmpeg.is_some() {
            return Err(anyhow!("ffmpeg is already running"));
        }

        let (width, height) = self.pixmap.get_size()?;

        let args = [
            // ==== Global Options ====
            "-hide_banner",
            "-loglevel",
            "info",
            // don't buffer input frames but render them immediately
            // "-re",
            // ==== Video Input Options ====
            // the video we give ffmpeg uses the 'rawvideo' format with 8-bit per color
            "-f",
            "rawvideo",
            "-pix_fmt",
            "rgb24",
            // the canvas size of the input video
            "-video_size",
            &format!("{}x{}", width, height),
            "-framerate",
            &format!("{}", self.options.fps),
            // get input from stdin
            "-i",
            "/dev/stdin",
            // ==== Audio Input Options ====
            // synthesise empty audio
            "-f",
            "lavfi",
            "-i",
            "anullsrc=channel_layout=stereo:sample_rate=44100",
            // ==== Output Options ====
            "-vcodec",
            "libx264",
            "-acodec",
            "aac",
            "-preset",
            "veryfast",
            // disable b-frames since webrtc does not support it and not all streaming servers can convert properly
            "-bf",
            "0",
            // set pixel-format
            "-pix_fmt",
            "yuv420p",
            // set bit-rates for video and audio
            "-b:v",
            "6000k",
            "-b:a",
            "128k",
            // set output frame rate
            "-framerate",
            &self.options.fps.to_string(),
            "-g",
            &(self.options.fps * 6).to_string(),
            // force output format to be rtsp
            "-f",
            "rtsp",
            &self.options.server_addr,
        ];
        tracing::info!(
            "Starting ffmpeg subprocess for rtsp streaming: {}",
            args.join(" ")
        );
        self.ffmpeg = Some(Command::new("ffmpeg").args(args).stdin(Stdio::piped()).spawn()?);
        Ok(())
    }

    async fn run(self) -> anyhow::Result<!> {
        let mut ffmpeg = self.ffmpeg.ok_or(anyhow!("ffmpeg is not running"))?;
        let Some(channel) = &mut ffmpeg.stdin else {
            tokio::time::sleep(Duration::from_secs(10)).await;
            return Err(anyhow!("intentional failure"));
        };
        let mut interval = tokio::time::interval(Duration::from_secs_f64(1.0 / self.options.fps as f64));

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
            // ffmpeg
            //     .communicate_bytes(Some(&raw_data))
            //     .expect("Could not communicate with ffmpeg");

            interval.tick().await;
        }
    }
}
