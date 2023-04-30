use std::future::Future;

/// Run a future to completion on the current runtime
pub fn async_to_sync<F>(fut: F) -> F::Output
where
    F: Future,
{
    let rt = tokio::runtime::Handle::current();
    rt.block_on(fut)
}
