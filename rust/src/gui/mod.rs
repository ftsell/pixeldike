//! GUI Management and rendering

use crate::pixmap::SharedPixmap;
use crate::DaemonHandle;
use minifb::{Window, WindowOptions};
use std::mem;
use tokio::task::LocalSet;

/// Start a window in a background thread that is appropriate for blocking work
pub fn start_gui(local_set: &mut LocalSet, pixmap: SharedPixmap) -> DaemonHandle {
    let (width, height) = pixmap.get_size();
    let mut window =
        Window::new("pixelflut", width, height, WindowOptions::default()).expect("Could not create window");

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let join_handle = local_set.spawn_local(async move { render(pixmap, window).await });
    DaemonHandle::new(join_handle)
}

async fn render(pixmap: SharedPixmap, mut window: Window) -> anyhow::Result<!> {
    let (width, height) = pixmap.get_size();
    loop {
        let buffer = unsafe { mem::transmute::<_, &[u32]>(pixmap.get_color_data()) };
        window
            .update_with_buffer(buffer, width, height)
            .expect("Could not update window data");
    }
}
