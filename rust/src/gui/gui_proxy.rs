use anyhow::{Context, Result};
use tokio::sync::{mpsc, oneshot};

/// A proxy to the Gui running in another thread
#[derive(Debug, Clone)]
pub struct GuiProxy {
    set_displayed_tx: mpsc::Sender<(bool, oneshot::Sender<Result<()>>)>,
    is_displayed_tx: mpsc::Sender<oneshot::Sender<Result<bool>>>,
}

impl GuiProxy {
    pub(super) fn new(
        set_displayed_tx: mpsc::Sender<(bool, oneshot::Sender<Result<()>>)>,
        is_displayed_tx: mpsc::Sender<oneshot::Sender<Result<bool>>>,
    ) -> Self {
        Self {
            set_displayed_tx,
            is_displayed_tx,
        }
    }

    pub async fn set_displayed(&self, val: bool) -> Result<()> {
        let (response_tx, response_rx) = oneshot::channel();
        self.set_displayed_tx
            .send((val, response_tx))
            .await
            .context("Could not send set_displayed to gui thread")?;
        response_rx
            .await
            .context("Could not receive set_displayed response")?
    }

    pub async fn is_displayed(&self) -> Result<bool> {
        let (response_tx, response_rx) = oneshot::channel();
        self.is_displayed_tx
            .send(response_tx)
            .await
            .context("Could not send is_displayed to gui thread")?;
        response_rx
            .await
            .context("Could not receive is_displayed response")?
    }
}
