//! A sink implementation for drawing on a linux framebuffer

use crate::pixmap::SharedPixmap;
use crate::DaemonHandle;
use anyhow::Context;
use framebuffer::Framebuffer;
use std::mem;
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::interval;

struct Sampler {
    /// Width of the source buffer from which a pixel is sampled
    src_width: usize,
    /// Height of the source buffer from which a pixel is sampled
    src_height: usize,
    /// Width of the destination buffer onto which a pixel is rendered
    out_width: usize,
    /// Width of the destination buffer onto which a pixel is rendered
    out_height: usize,
}

impl Sampler {
    pub fn new(src_width: usize, src_height: usize, out_width: usize, out_height: usize) -> Self {
        Self {
            src_width,
            src_height,
            out_width,
            out_height,
        }
    }

    #[inline(always)]
    pub fn sample(&self, x: usize, y: usize) -> (usize, usize) {
        let x: f64 = x as f64;
        let y: f64 = y as f64;

        let x = x / self.out_width as f64 * self.src_width as f64;
        let y = y / self.out_height as f64 * self.src_height as f64;

        (x as usize, y as usize)
    }
}

/// Options for configuring a [`FramebufferSink`]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FramebufferSinkOptions {
    /// The path to a framebuffer device
    pub path: PathBuf,
    /// How many frames per second should be rendered
    pub framerate: usize,
}

/// A sink that periodically renders pixmap data onto a framebuffer device
#[derive(Debug)]
pub struct FramebufferSink {
    options: FramebufferSinkOptions,
    pixmap: SharedPixmap,
}

impl FramebufferSink {
    /// Create a new `FramebufferSink`
    pub fn new(options: FramebufferSinkOptions, pixmap: SharedPixmap) -> Self {
        Self { options, pixmap }
    }

    /// Start a background task for rendering onto the framebuffer device
    pub async fn start(self) -> anyhow::Result<DaemonHandle> {
        let fb = self.open_fb_device()?;

        let handle = tokio::spawn(async move { self.render(fb).await });
        Ok(DaemonHandle::new(handle))
    }

    /// Open and configure the framebuffer device for later rendering
    fn open_fb_device(&self) -> anyhow::Result<Framebuffer> {
        {
            let fb = Framebuffer::new(&self.options.path).context(format!(
                "Could not obtain a handle to the framebuffer device {}",
                self.options.path.display()
            ))?;

            // configure framebuffer
            let mut var_screeninfo = fb.var_screen_info.clone();
            var_screeninfo.xres = var_screeninfo.xres_virtual;
            var_screeninfo.yres = var_screeninfo.yres_virtual;
            Framebuffer::put_var_screeninfo(&fb.device, &var_screeninfo)?;
        }

        // re-open framebuffer device to apply configuration
        Ok(Framebuffer::new(&self.options.path)?)
    }

    async fn render(self, mut fb: Framebuffer) -> anyhow::Result<!> {
        let mut interval = interval(Duration::from_secs_f64(1.0 / self.options.framerate as f64));

        let (pixmap_width, pixmap_height) = self.pixmap.get_size();
        let sampler = Sampler::new(
            pixmap_width,
            pixmap_height,
            fb.var_screen_info.xres as usize,
            fb.var_screen_info.yres as usize,
        );

        loop {
            let pixel_data = self.pixmap.get_raw_data();

            let r_encoding = &fb.var_screen_info.red;
            let g_encoding = &fb.var_screen_info.green;
            let b_encoding = &fb.var_screen_info.blue;

            let bits_per_pixel = fb.var_screen_info.bits_per_pixel as usize;
            let bytes_per_pixel = bits_per_pixel / 8;
            let line_length = fb.fix_screen_info.line_length as usize;

            let frame = &mut fb.frame;
            for screen_x in 0..fb.var_screen_info.xres as usize {
                for screen_y in 0..fb.var_screen_info.yres as usize {
                    let (px_x, px_y) = sampler.sample(screen_x, screen_y);
                    let pixmap_pixel = &pixel_data[px_y * pixmap_width + px_x];
                    let screen_i = screen_y * line_length + screen_x * bytes_per_pixel;

                    if bits_per_pixel == 16 {
                        let mut encoded_pixel: u16 = 0;

                        encoded_pixel |= (pixmap_pixel.0 as u16 >> (8u16 - r_encoding.length as u16))
                            << (r_encoding.offset as u16);
                        encoded_pixel |= (pixmap_pixel.1 as u16 >> (8u16 - g_encoding.length as u16))
                            << (g_encoding.offset as u16);
                        encoded_pixel |= (pixmap_pixel.2 as u16 >> (8u16 - b_encoding.length as u16))
                            << (b_encoding.offset as u16);

                        frame[screen_i..screen_i + mem::size_of::<u16>()]
                            .copy_from_slice(&encoded_pixel.to_ne_bytes());
                    } else if bits_per_pixel == 32 {
                        let mut encoded_pixel: u32 = 0;

                        encoded_pixel |=
                            (pixmap_pixel.0 as u32 >> (8u32 - r_encoding.length)) << (r_encoding.offset);
                        encoded_pixel |=
                            (pixmap_pixel.1 as u32 >> (8u32 - g_encoding.length)) << (g_encoding.offset);
                        encoded_pixel |=
                            (pixmap_pixel.2 as u32 >> (8u32 - b_encoding.length)) << (b_encoding.offset);

                        frame[screen_i..screen_i + mem::size_of::<u32>()]
                            .copy_from_slice(&encoded_pixel.to_ne_bytes());
                    } else {
                        panic!(
                            "Framebuffer has unsupported bits_per_pixel {}",
                            fb.var_screen_info.bits_per_pixel
                        )
                    }
                }
            }

            interval.tick().await;
        }
    }
}
