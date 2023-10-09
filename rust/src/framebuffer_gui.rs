use crate::pixmap::traits::{PixmapBase, PixmapRawRead};
use crate::pixmap::SharedPixmap;
use framebuffer::Framebuffer;
use std::mem;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Notify;
use tokio::task::JoinHandle;
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

#[derive(Debug)]
pub struct FramebufferGui {
    framebuffer: Framebuffer,
}

impl FramebufferGui {
    pub fn new(path: PathBuf) -> Self {
        // configure framebuffer on the given path
        let framebuffer =
            Framebuffer::new(&path).expect("Could not obtain handle to the framebuffer device /dev/fb");
        let mut var_screeninfo = framebuffer.var_screen_info.clone();
        var_screeninfo.xres = var_screeninfo.xres_virtual;
        var_screeninfo.yres = var_screeninfo.yres_virtual;

        Framebuffer::put_var_screeninfo(&framebuffer.device, &var_screeninfo).unwrap();

        // re-read framebuffer struct from applied configuration
        Self {
            framebuffer: Framebuffer::new(&path)
                .expect("Could not obtain handle to the framebuffer device /dev/fb"),
        }
    }
}

/// Start an async task that renders the current pixmap state to a framebuffer gui
pub fn start_gui_task<P>(gui: FramebufferGui, pixmap: SharedPixmap<P>) -> (JoinHandle<()>, Arc<Notify>)
where
    P: PixmapRawRead + Send + Sync + 'static,
{
    let notify = Arc::new(Notify::new());
    let notify2 = notify.clone();
    let handle = tokio::spawn(async move { render(gui, pixmap, notify2).await });

    (handle, notify)
}

async fn render<P>(mut gui: FramebufferGui, pixmap: SharedPixmap<P>, _cancel: Arc<Notify>)
where
    P: PixmapRawRead + Send + Sync + 'static,
{
    let mut render_interval = interval(Duration::from_millis(1000 / 30));

    loop {
        render_interval.tick().await;
        render_one_frame(&mut gui, &pixmap);
    }
}

fn render_one_frame<P>(gui: &mut FramebufferGui, pixmap: &SharedPixmap<P>)
where
    P: PixmapRawRead + Send + Sync + 'static,
{
    let pixel_data = pixmap.get_raw_data().unwrap();
    let (pixmap_width, pixmap_height) = pixmap.get_size().unwrap();

    let r_encoding = &gui.framebuffer.var_screen_info.red;
    let g_encoding = &gui.framebuffer.var_screen_info.green;
    let b_encoding = &gui.framebuffer.var_screen_info.blue;

    let bits_per_pixel = gui.framebuffer.var_screen_info.bits_per_pixel as usize;
    let bytes_per_pixel = (bits_per_pixel / 8) as usize;
    let line_length = gui.framebuffer.fix_screen_info.line_length as usize;

    let sampler = Sampler::new(
        pixmap_width,
        pixmap_height,
        gui.framebuffer.var_screen_info.xres as usize,
        gui.framebuffer.var_screen_info.yres as usize,
    );

    let frame = &mut gui.framebuffer.frame;
    for screen_x in 0..gui.framebuffer.var_screen_info.xres as usize {
        for screen_y in 0..gui.framebuffer.var_screen_info.yres as usize {
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
                    gui.framebuffer.var_screen_info.bits_per_pixel
                )
            }
        }
    }
}
