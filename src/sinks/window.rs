//! A sink for drawing on an X or Wayland window

use crate::pixmap::SharedPixmap;
use crate::DaemonResult;
use anyhow::anyhow;
use minifb::{Window, WindowOptions};
use std::mem;
use std::time::Duration;
use tokio::task::{AbortHandle, JoinSet};
use tokio::time::MissedTickBehavior;

/// Start the window in the background.
///
/// Note that handles to X/Wayland windows are not Send so the background task must always be scheduled on the same thread.
/// This is achieved by passing an existing `LocalSet` in which the background task will execute.
pub fn start(join_set: &mut JoinSet<DaemonResult>, pixmap: SharedPixmap) -> anyhow::Result<AbortHandle> {
    let (width, height) = pixmap.get_size();
    let mut window = Window::new("pixelflut", width, height, WindowOptions::default())?;

    window.set_title("Pixelflut Server");

    let handle = join_set
        .build_task()
        .name("window_renderer")
        .spawn_local(async move { render(pixmap, window).await })?;
    Ok(handle)
}

async fn render(pixmap: SharedPixmap, mut window: Window) -> anyhow::Result<!> {
    let (width, height) = pixmap.get_size();
    let mut interval = tokio::time::interval(Duration::from_millis(1000 / 60));
    interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
    loop {
        if !window.is_open() {
            return Err(anyhow!(
                "rendering window has been closed, assuming server should exit"
            ));
        }

        let buffer = unsafe { mem::transmute::<_, &[u32]>(pixmap.get_color_data()) };
        window
            .update_with_buffer(buffer, width, height)
            .expect("Could not update window data");

        interval.tick().await;
    }
}
