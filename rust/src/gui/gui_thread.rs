//! Code related to running the whole GUI on a separate thread or task

use std::sync::Arc;
use anyhow::Result;

use tokio::{
    sync::{mpsc, oneshot},
    task::{spawn_blocking, JoinHandle},
};

use crate::pixmap::traits::PixmapRawRead;

use super::{gui_management::GuiContext, GuiProxy};

/// An owning and authorative handle to the running Gui Thread
#[derive(Debug)]
pub struct GuiThread {
    pub thread: JoinHandle<()>,
}

impl GuiThread {
    pub fn start(pixmap: Arc<impl PixmapRawRead + Send + Sync + 'static>) -> (Self, GuiProxy) {
        // setup communication channels
        let (set_displayed_tx, set_displayed_rx) = mpsc::channel::<(bool, oneshot::Sender<Result<()>>)>(64);
        let (is_displayed_tx, is_displayed_rx) = mpsc::channel::<oneshot::Sender<Result<bool>>>(64);
        let gui_proxy = GuiProxy::new(set_displayed_tx, is_displayed_tx);

        // spawn the gui handling code on a task that is intended for blocking code
        let join_handle = spawn_blocking(move || {
            GuiContext::new().expect("Could not create Gui").run();
        });

        (Self { thread: join_handle }, gui_proxy)
    }
}
