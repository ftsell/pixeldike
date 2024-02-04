//! A sink implementation for drawing on a linux framebuffer

use crate::pixmap::SharedPixmap;
use crate::DaemonHandle;
use anyhow::Context;
use framebuffer::{Bitfield, Framebuffer};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::{interval, MissedTickBehavior};

struct Sampler {
    /// A mapping of screen-pixel-index to pixmap-pixel-index
    ///
    /// If it is `None`, no mapping needs to be done because the screen and pixmap have the same sizes
    mapping: Option<Vec<u32>>,
}

impl Sampler {
    pub fn new(src_width: usize, src_height: usize, out_width: usize, out_height: usize) -> Self {
        if src_width == out_width && src_height == out_height {
            Self { mapping: None }
        } else {
            tracing::warn!("Framebuffer has size {}x{} while pixmap has size {}x{}. This requires an additional sampling step which slows down rendering", out_width, out_height, src_width, src_height);
            Self {
                mapping: Some(
                    (0..out_width * out_height)
                        .map(|i_screen_px| {
                            let screen_x = i_screen_px % out_width;
                            let screen_y = i_screen_px / out_width;
                            let px_x = (screen_x * src_width) / out_width;
                            let px_y = (screen_y * src_height) / out_height;
                            (px_y * src_width + px_x) as u32
                        })
                        .collect(),
                ),
            }
        }
    }

    pub fn needs_sampling(&self) -> bool {
        self.mapping.is_some()
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn get_mapping(&self, i_screen_px: usize) -> Option<usize> {
        self.mapping.as_ref()?.get(i_screen_px).map(|x| *x as usize)
    }

    #[inline(always)]
    pub unsafe fn get_mappin_unchecked(&self, i_screen_px: usize) -> usize {
        *self
            .mapping
            .as_ref()
            .unwrap_unchecked()
            .get_unchecked(i_screen_px) as usize
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

    /// Render in a loop at the desired framerate (or as close to it as possible)
    async fn render(self, mut fb: Framebuffer) -> anyhow::Result<!> {
        let mut interval = interval(Duration::from_secs_f64(1.0 / self.options.framerate as f64));
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let (pixmap_width, pixmap_height) = self.pixmap.get_size();
        let screen_width = fb.var_screen_info.xres as usize;
        let screen_height = fb.var_screen_info.yres as usize;
        let sampler = Sampler::new(pixmap_width, pixmap_height, screen_width, screen_height);

        // fetch important info from framebuffer info
        let r_encoding = fb.var_screen_info.red.clone();
        let g_encoding = fb.var_screen_info.green.clone();
        let b_encoding = fb.var_screen_info.blue.clone();
        let bits_per_pixel = fb.var_screen_info.bits_per_pixel as usize;

        let render_once_fn = match bits_per_pixel {
            32 => Self::render_once_u32,
            16 => Self::render_once_u16,
            _ => panic!(
                "Unsupported framebuffer pixel-depth of {} bits per pixel",
                bits_per_pixel
            ),
        };

        loop {
            render_once_fn(
                &self,
                &r_encoding,
                &g_encoding,
                &b_encoding,
                &sampler,
                &mut fb,
                screen_width,
                screen_height,
            );
            interval.tick().await;
        }
    }

    /// Render one frame to the framebuffer using 32-bit pixel depth
    #[allow(clippy::too_many_arguments)]
    fn render_once_u32(
        &self,
        r_encoding: &Bitfield,
        g_encoding: &Bitfield,
        b_encoding: &Bitfield,
        sampler: &Sampler,
        fb: &mut Framebuffer,
        screen_width: usize,
        screen_height: usize,
    ) {
        let pixel_data = unsafe { self.pixmap.get_color_data() };

        // encode pixel data into framebuffer format
        let encoded_pixel_data: Vec<u32> = pixel_data
            .iter()
            .map(|i_px| {
                let encoded_r = (i_px.0 as u32 >> (8 - r_encoding.length as u32)) << (r_encoding.offset);
                let encoded_b = (i_px.1 as u32 >> (8 - g_encoding.length as u32)) << (g_encoding.offset);
                let encoded_c = (i_px.2 as u32 >> (8 - b_encoding.length as u32)) << (b_encoding.offset);
                encoded_r | encoded_b | encoded_c
            })
            .collect();

        // sample pixels to framebuffer size
        let pixels: Vec<u32> = if sampler.needs_sampling() {
            (0..screen_width * screen_height)
                .map(|px| unsafe {
                    let sample_px = sampler.get_mappin_unchecked(px);
                    *encoded_pixel_data.get_unchecked(sample_px)
                })
                .collect()
        } else {
            encoded_pixel_data
        };

        // transmute and copy to framebuffer
        let pixel_bytes = unsafe {
            let (prefix, bytes, suffix) = pixels.align_to::<u8>();
            assert_eq!(prefix.len(), 0);
            assert_eq!(suffix.len(), 0);
            bytes
        };
        fb.write_frame(&pixel_bytes);
    }

    /// Render one frame to the framebuffer using 32-bit pixel depth
    #[allow(clippy::too_many_arguments)]
    fn render_once_u16(
        &self,
        r_encoding: &Bitfield,
        g_encoding: &Bitfield,
        b_encoding: &Bitfield,
        sampler: &Sampler,
        fb: &mut Framebuffer,
        screen_width: usize,
        screen_height: usize,
    ) {
        let pixel_data = unsafe { self.pixmap.get_color_data() };

        // encode pixel data into framebuffer format
        let encoded_pixel_data: Vec<u16> = pixel_data
            .iter()
            .map(|i_px| {
                let encoded_r = (i_px.0 as u16 >> (8 - r_encoding.length as u16)) << (r_encoding.offset);
                let encoded_b = (i_px.1 as u16 >> (8 - g_encoding.length as u16)) << (g_encoding.offset);
                let encoded_c = (i_px.2 as u16 >> (8 - b_encoding.length as u16)) << (b_encoding.offset);
                encoded_r | encoded_b | encoded_c
            })
            .collect();

        // sample pixels to framebuffer size
        let pixels: Vec<u16> = if sampler.needs_sampling() {
            (0..screen_width * screen_height)
                .map(|px| unsafe {
                    let sample_px = sampler.get_mappin_unchecked(px);
                    *encoded_pixel_data.get_unchecked(sample_px)
                })
                .collect()
        } else {
            encoded_pixel_data
        };

        // transmute and copy to framebuffer
        let pixel_bytes = unsafe {
            let (prefix, bytes, suffix) = pixels.align_to::<u8>();
            assert_eq!(prefix.len(), 0);
            assert_eq!(suffix.len(), 0);
            bytes
        };
        fb.write_frame(&pixel_bytes);
    }
}
