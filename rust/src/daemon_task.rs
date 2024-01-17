use tokio::task::JoinHandle;

/// A handle to a background task that can be used to control it
#[derive(Debug)]
pub struct DaemonHandle {
    join_handle: JoinHandle<anyhow::Result<!>>,
}

impl DaemonHandle {
    pub(super) fn new(join_handle: JoinHandle<anyhow::Result<!>>) -> Self {
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

    /// Join the execution of this background into the currently running task
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
