use crate::pixmap::traits::{PixmapRead, PixmapWrite};
use crate::pixmap::SharedPixmap;
use crate::state_encoding::SharedMultiEncodings;
use crate::DaemonHandle;
use async_trait::async_trait;

#[async_trait]
pub trait GenServer {
    type Options;

    fn new(options: Self::Options) -> Self;

    async fn start<P>(
        self,
        pixmap: SharedPixmap<P>,
        encodings: SharedMultiEncodings,
    ) -> anyhow::Result<DaemonHandle>
    where
        P: PixmapRead + PixmapWrite + Send + Sync + 'static;
}
