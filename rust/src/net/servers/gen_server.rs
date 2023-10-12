use crate::pixmap::traits::{PixmapRead, PixmapWrite};
use crate::pixmap::SharedPixmap;
use crate::state_encoding::SharedMultiEncodings;
use async_trait::async_trait;
use tokio::task::JoinHandle;

#[async_trait]
pub trait GenServer {
    type Options;

    fn new(options: Self::Options) -> Self;

    async fn start<P>(
        self,
        pixmap: SharedPixmap<P>,
        encodings: SharedMultiEncodings,
    ) -> anyhow::Result<GenServerHandle>
    where
        P: PixmapRead + PixmapWrite + Send + Sync + 'static;
}

pub struct GenServerHandle {
    join_handle: JoinHandle<anyhow::Result<()>>,
}

impl GenServerHandle {
    pub(super) fn new(join_handle: JoinHandle<anyhow::Result<()>>) -> Self {
        Self { join_handle }
    }

    /// Stop the running server
    pub fn stop(&mut self) {
        self.join_handle.abort();
    }

    /// Whether the server is currently (still) running
    pub fn is_running(&self) -> bool {
        !self.join_handle.is_finished()
    }

    /// Join the execution of this server into the current task
    pub async fn join(self) -> anyhow::Result<()> {
        match self.join_handle.await {
            Ok(task_result) => match task_result {
                Ok(_) => Ok(()),
                Err(task_err) => Err(task_err),
            },
            Err(tokio_err) => Err(tokio_err.into()),
        }
    }
}
