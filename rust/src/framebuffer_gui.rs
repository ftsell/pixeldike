//! Code for rendering pixmaps on framebuffer devices

use crate::pixmap::traits::PixmapRawRead;
use crate::pixmap::SharedPixmap;
use crate::DaemonHandle;
use framebuffer::Framebuffer;
use std::mem;
use std::path::PathBuf;
use tokio::time::Interval;

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

/// A struct for controlling the rendering of a pixmap on a framebuffer device
#[derive(Debug)]
pub struct FramebufferGui {
    framebuffer: Framebuffer,
    render_interval: Interval,
}

impl FramebufferGui {
    /// Create a `FramebufferGui` that draws on the framebuffer device located at `path`
    pub fn new(path: PathBuf, render_interval: Interval) -> Self {
        // configure framebuffer on the given path
        let framebuffer = Framebuffer::new(&path).unwrap_or_else(|_| {
            panic!(
                "Could not obtain handle to the framebuffer device {}",
                path.to_string_lossy()
            )
        });
        let mut var_screeninfo = framebuffer.var_screen_info.clone();
        var_screeninfo.xres = var_screeninfo.xres_virtual;
        var_screeninfo.yres = var_screeninfo.yres_virtual;

        Framebuffer::put_var_screeninfo(&framebuffer.device, &var_screeninfo).unwrap();

        // re-read framebuffer struct from applied configuration
        Self {
            framebuffer: Framebuffer::new(&path).unwrap_or_else(|_| {
                panic!(
                    "Could not obtain handle to the framebuffer device {}",
                    path.to_string_lossy()
                )
            }),
            render_interval,
        }
    }

    /// Start a background thread that renders this `FramebufferGui`
    pub fn start_render_task<P>(self, pixmap: SharedPixmap<P>) -> DaemonHandle
    where
        P: PixmapRawRead + Send + Sync + 'static,
    {
        let join_handle = tokio::spawn(async move { self.render(pixmap).await });
        DaemonHandle::new(join_handle)
    }

    /// Render the current pixmap state in a loop
    async fn render<P>(mut self, pixmap: SharedPixmap<P>) -> anyhow::Result<!>
    where
        P: PixmapRawRead,
    {
        loop {
            self.render_interval.tick().await;

            let pixel_data = pixmap.get_raw_data().unwrap();
            let (pixmap_width, pixmap_height) = pixmap.get_size().unwrap();

            let r_encoding = &self.framebuffer.var_screen_info.red;
            let g_encoding = &self.framebuffer.var_screen_info.green;
            let b_encoding = &self.framebuffer.var_screen_info.blue;

            let bits_per_pixel = self.framebuffer.var_screen_info.bits_per_pixel as usize;
            let bytes_per_pixel = (bits_per_pixel / 8) as usize;
            let line_length = self.framebuffer.fix_screen_info.line_length as usize;

            let sampler = Sampler::new(
                pixmap_width,
                pixmap_height,
                self.framebuffer.var_screen_info.xres as usize,
                self.framebuffer.var_screen_info.yres as usize,
            );

            let frame = &mut self.framebuffer.frame;
            for screen_x in 0..self.framebuffer.var_screen_info.xres as usize {
                for screen_y in 0..self.framebuffer.var_screen_info.yres as usize {
                    let (px_x, px_y) = sampler.sample(screen_x, screen_y);
                    let pixmap_pixel = &pixel_data[px_y * pixmap_width + px_x];
                    let screen_i = screen_y * line_length + screen_x * bytes_per_pixel;

                    if bits_per_pixel == 16 {
                        let mut encoded_pixel: u16 = 0;

                        encoded_pixel |= (pixmap_pixel.0 as u16 >> (8u16 - r_encoding.length as u16))
                            << (16 - r_encoding.length as u16 - r_encoding.offset as u16);
                        encoded_pixel |= (pixmap_pixel.1 as u16 >> (8u16 - g_encoding.length as u16))
                            << (16 - g_encoding.length as u16 - g_encoding.offset as u16);
                        encoded_pixel |= (pixmap_pixel.2 as u16 >> (8u16 - b_encoding.length as u16))
                            << (16 - b_encoding.length as u16 - b_encoding.offset as u16);

                        frame[screen_i..screen_i + mem::size_of::<u16>()]
                            .copy_from_slice(&encoded_pixel.to_ne_bytes());
                    } else {
                        panic!(
                            "Framebuffer has unsupported bits_per_pixel {}",
                            self.framebuffer.var_screen_info.bits_per_pixel
                        )
                    }
                }
            }
        }
    }
}
