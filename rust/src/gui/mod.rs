//! GUI Management and rendering

mod rendering;
mod shader;
mod texture;
mod utils;
mod window_management;

use crate::gui::window_management::GuiContext;
use crate::pixmap::traits::PixmapRawRead;
use crate::pixmap::SharedPixmap;
use crate::DaemonHandle;
use tokio::task::spawn_blocking;

/// Start a window in a background thread that is appropriate for blocking work
pub fn start_gui<P>(pixmap: SharedPixmap<P>) -> DaemonHandle
where
    P: PixmapRawRead + Send + Sync + 'static,
{
    let join_handle = spawn_blocking(move || {
        GuiContext::new(pixmap).expect("Could not create GUI").run();
    });
    DaemonHandle::new(join_handle)
}
