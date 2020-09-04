//! Async support with a built-in thread pool.

use crate::{async_support, async_support::BlockingExecutor, Ownership, Result};
use async_trait::async_trait;
use futures_io::{AsyncRead, AsyncWrite};
use std::path::Path;

struct FuturesBlockingExecutor {}

#[async_trait]
impl BlockingExecutor for FuturesBlockingExecutor {
    async fn execute_blocking<T, F>(f: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        Ok(blocking::unblock(f).await)
    }
}

const FUTURES_BLOCKING_EXECUTOR: FuturesBlockingExecutor = FuturesBlockingExecutor {};

/// Async version of [`list_archive_files`](crate::list_archive_files).
pub async fn list_archive_files<R>(source: R) -> Result<Vec<String>>
where
    R: AsyncRead + Unpin,
{
    async_support::list_archive_files(FUTURES_BLOCKING_EXECUTOR, source).await
}

/// Async version of [`uncompress_data`](crate::uncompress_data).
pub async fn uncompress_data<R, W>(source: R, target: W) -> Result<usize>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    async_support::uncompress_data(FUTURES_BLOCKING_EXECUTOR, source, target).await
}

/// Async version of [`uncompress_archive`](crate::uncompress_archive).
pub async fn uncompress_archive<R>(source: R, dest: &Path, ownership: Ownership) -> Result<()>
where
    R: AsyncRead + Unpin,
{
    async_support::uncompress_archive(FUTURES_BLOCKING_EXECUTOR, source, dest, ownership).await
}

/// Async version of
/// [`uncompress_archive_file`](crate::uncompress_archive_file).
pub async fn uncompress_archive_file<R, W>(source: R, target: W, path: &str) -> Result<usize>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    async_support::uncompress_archive_file(FUTURES_BLOCKING_EXECUTOR, source, target, path).await
}
