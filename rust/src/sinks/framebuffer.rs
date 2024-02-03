//! A sink implementation for drawing on a linux framebuffer

use crate::pixmap::SharedPixmap;
use crate::DaemonHandle;
use anyhow::Context;
use framebuffer::{Bitfield, Framebuffer};
use std::path::PathBuf;
use std::time::Duration;
use std::{mem, ptr};
use tokio::time::{interval, Instant, MissedTickBehavior};

struct Sampler {
    /// A mapping of sreen-pixel-index to pixmap-pixel-index
    mapping: Vec<usize>,
}

impl Sampler {
    pub fn new(src_width: usize, src_height: usize, out_width: usize, out_height: usize) -> Self {
        Self {
            mapping: (0..out_width * out_height)
                .map(|i_screen_px| {
                    let screen_x = i_screen_px % out_width;
                    let screen_y = i_screen_px / out_width;
                    let px_x = (screen_x * src_width) / out_width;
                    let px_y = (screen_y * src_height) / out_height;
                    px_y * src_width + px_x
                })
                .collect(),
        }
    }

    #[allow(unused)]
    #[inline(always)]
    pub fn get_mapping(&self, i_screen_px: usize) -> Option<usize> {
        self.mapping.get(i_screen_px).cloned()
    }

    #[inline(always)]
    pub unsafe fn get_mappin_unchecked(&self, i_screen_px: usize) -> usize {
        *self.mapping.get_unchecked(i_screen_px)
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

        // TODO Add support for 16-bit pixel depth
        let render_fun = match bits_per_pixel {
            32 => &Self::render_once::<{ mem::size_of::<u32>() }>,
            _ => panic!("Unsupported framebuffer pixel-depth {}", bits_per_pixel),
        };

        let mut frame = vec![0u8; screen_width * screen_height * bits_per_pixel / 8];

        loop {
            render_fun(
                &self,
                &r_encoding,
                &g_encoding,
                &b_encoding,
                &sampler,
                &mut frame,
                screen_width,
                screen_height,
            );
            fb.write_frame(&frame);
            interval.tick().await;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_once<const BYTES_PER_PIXEL: usize>(
        &self,
        r_encoding: &Bitfield,
        g_encoding: &Bitfield,
        b_encoding: &Bitfield,
        sampler: &Sampler,
        frame: &mut [u8],
        screen_width: usize,
        screen_height: usize,
    ) {
        const STEP_SIZE: usize = 8;
        let iteration_max = (screen_width * screen_height).next_multiple_of(STEP_SIZE) - STEP_SIZE;

        let t1 = Instant::now();
        let pixel_data = self.pixmap.get_raw_data();
        let t2 = Instant::now();

        // encode pixel data into framebuffer format
        let encoded_pixel_data: Vec<u32> = pixel_data
            .iter()
            .map(|i_px| {
                let encoded_r = (i_px.0 as u32 >> (8u32 - r_encoding.length)) << (r_encoding.offset);
                let encoded_b = (i_px.1 as u32 >> (8u32 - g_encoding.length)) << (g_encoding.offset);
                let encoded_c = (i_px.2 as u32 >> (8u32 - b_encoding.length)) << (b_encoding.offset);
                encoded_r | encoded_b | encoded_c
            })
            .collect();
        let t3 = Instant::now();

        // render pixels into a buffer (not the actual fb device)
        #[inline(always)]
        unsafe fn loop_body<const BYTES_PER_PIXEL: usize>(
            frame: &mut [u8],
            encoded_px: &[u32],
            sampling: &Sampler,
            i_screen_px: usize,
        ) {
            let frame_data =
                frame.get_unchecked_mut(i_screen_px * BYTES_PER_PIXEL..(i_screen_px + 1) * BYTES_PER_PIXEL);
            let encoded_data = encoded_px
                .get_unchecked(sampling.get_mappin_unchecked(i_screen_px))
                .to_ne_bytes();
            ptr::copy_nonoverlapping(encoded_data.as_ptr(), frame_data.as_mut_ptr(), BYTES_PER_PIXEL);
        }
        for i_screen_px in (0..iteration_max).step_by(STEP_SIZE) {
            unsafe {
                loop_body::<4>(frame, &encoded_pixel_data, &sampler, i_screen_px);
                loop_body::<4>(frame, &encoded_pixel_data, &sampler, i_screen_px + 1);
                loop_body::<4>(frame, &encoded_pixel_data, &sampler, i_screen_px + 2);
                loop_body::<4>(frame, &encoded_pixel_data, &sampler, i_screen_px + 3);
                loop_body::<4>(frame, &encoded_pixel_data, &sampler, i_screen_px + 4);
                loop_body::<4>(frame, &encoded_pixel_data, &sampler, i_screen_px + 5);
                loop_body::<4>(frame, &encoded_pixel_data, &sampler, i_screen_px + 6);
                loop_body::<4>(frame, &encoded_pixel_data, &sampler, i_screen_px + 7);
            }
        }
        for i_screen_px in iteration_max..screen_width * screen_height {
            unsafe {
                loop_body::<4>(frame, &encoded_pixel_data, &sampler, i_screen_px);
            }
        }

        let t4 = Instant::now();

        tracing::info!(
                "rendering data: get_raw_data(): {:2}ms    encoding: {:2}ms    rendering: {:2}ms    total: {:3}ms ({:.2}fps)",
                (t2 - t1).as_millis(),
                (t3 - t2).as_millis(),
                (t4 - t3).as_millis(),
                (t4 - t1).as_millis(),
                1.0 / (t4 - t1).as_secs_f64(),
            );
    }
}
