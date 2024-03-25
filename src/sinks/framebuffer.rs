//! A sink implementation for drawing on a linux framebuffer

use crate::pixmap::{Color, SharedPixmap};
use crate::DaemonResult;
use anyhow::Context;
use framebuffer::{Bitfield, Framebuffer};
use std::path::PathBuf;
use std::time::Duration;
use tokio::task::{AbortHandle, JoinSet};
use tokio::time::{interval, Instant, MissedTickBehavior};
use tracing::info;

#[derive(Debug, Clone, Eq, PartialEq)]
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
    pub async fn start(self, join_set: &mut JoinSet<DaemonResult>) -> anyhow::Result<AbortHandle> {
        let fb = self.open_fb_device()?;
        let handle = join_set
            .build_task()
            .name("framebuffer")
            .spawn(async move { self.render(fb).await })?;
        Ok(handle)
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

        let fb_pixels = screen_width * screen_height;
        let encoder = Encoder {
            r: fb.var_screen_info.red.clone(),
            g: fb.var_screen_info.green.clone(),
            b: fb.var_screen_info.blue.clone(),
        };
        let renderer = Renderer { sampler, encoder };

        let bits_per_pixel = fb.var_screen_info.bits_per_pixel as usize;
        let render_once_fn = match bits_per_pixel {
            32 => Renderer::render::<u32>,
            16 => Renderer::render::<u16>,
            _ => panic!(
                "Unsupported framebuffer pixel-depth of {} bits per pixel",
                bits_per_pixel
            ),
        };

        loop {
            let t1 = Instant::now();
            render_once_fn(
                &&renderer,
                unsafe { self.pixmap.get_color_data() },
                &mut fb,
                fb_pixels,
            );
            let t2 = Instant::now();
            info!("Render: {}ms", (t2 - t1).as_millis());
            interval.tick().await;
        }
    }
}

/// A little helper struct that provides a generic render method.
/// You can call render with any type T, as long as T: Copy and the
/// Encoder can encode Pixels to T.
#[derive(Debug, Clone)]
pub struct Renderer {
    encoder: Encoder,
    sampler: Sampler,
}

impl Renderer {
    fn render<T: Copy>(&self, pixel_data: &[Color], fb: &mut Framebuffer, fb_pixels: usize)
    where
        Encoder: Encode<T>,
    {
        // encode pixel data into framebuffer format
        let encoded: Vec<T> = self.encoder.encode_vec(pixel_data);

        // sample pixels to framebuffer size
        let pixels = if self.sampler.needs_sampling() {
            let sampled = sample_vec(&self.sampler, &encoded, fb_pixels);
            sampled
        } else {
            encoded
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

#[inline(always)]
fn sample_iter<'e, 's: 'e, T: Copy>(
    sampler: &'s Sampler,
    encoded: &'e [T],
    px: usize,
) -> impl Iterator<Item = T> + 'e {
    let iter = (0..px).map(|px| unsafe {
        let sample_px = sampler.get_mappin_unchecked(px);
        *encoded.get_unchecked(sample_px)
    });
    iter
}

#[inline(always)]
fn sample_vec<T: Copy>(sampler: &Sampler, encoded: &[T], px: usize) -> Vec<T> {
    sample_iter(sampler, encoded, px).collect()
}

/// A Pixel encoder.
/// The r, g and b fields describe the pixel layout.
/// Call encoding methods trough the Encode<Target> trait.
/// Currently supported are the u16 and u32 Target types.
#[derive(Debug, Clone)]
struct Encoder {
    r: Bitfield,
    g: Bitfield,
    b: Bitfield,
}

/// The Encode<Target> trait represents the encoding of pixels (Pixel -> Target).
trait Encode<Target> {
    fn encode_single(&self, px: Color) -> Target;
    #[inline(always)]
    fn encode_vec(&self, pixmap: &[Color]) -> Vec<Target> {
        pixmap.iter().map(|px| self.encode_single(*px)).collect()
    }
}

impl Encode<u32> for Encoder {
    #[inline(always)]
    fn encode_single(&self, px: Color) -> u32 {
        let px: (u8, u8, u8) = px.into();
        let encoded_r = (px.0 as u32 >> (8 - self.r.length)) << (self.r.offset);
        let encoded_b = (px.1 as u32 >> (8 - self.g.length)) << (self.g.offset);
        let encoded_c = (px.2 as u32 >> (8 - self.b.length)) << (self.b.offset);
        encoded_r | encoded_b | encoded_c
    }
}

impl Encode<u16> for Encoder {
    #[inline(always)]
    fn encode_single(&self, px: Color) -> u16 {
        let px: (u8, u8, u8) = px.into();
        let encoded_r = (px.0 as u16 >> (8 - self.r.length as u16)) << (self.r.offset);
        let encoded_b = (px.1 as u16 >> (8 - self.g.length as u16)) << (self.g.offset);
        let encoded_c = (px.2 as u16 >> (8 - self.b.length as u16)) << (self.b.offset);
        encoded_r | encoded_b | encoded_c
    }
}
