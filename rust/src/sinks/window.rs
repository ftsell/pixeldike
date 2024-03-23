//! A sink for drawing on an X or Wayland window

use crate::pixmap::SharedPixmap;
use crate::DaemonHandle;
use minifb::{Window, WindowOptions};
use std::mem;
use tokio::task::LocalSet;

/// Start the window in the background.
///
/// Note that handles to X/Wayland windows are not Send so the background task must always be scheduled on the same thread.
/// This is achieved by passing an existing `LocalSet` in which the background task will execute.
pub fn start(local_set: &mut LocalSet, pixmap: SharedPixmap) -> DaemonHandle {
    let (width, height) = pixmap.get_size();
    let mut window =
        Window::new("pixelflut", width, height, WindowOptions::default()).expect("Could not create window");

    // Limit to max ~60 fps update rate
    window.set_title("Pixelflut Server");
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
