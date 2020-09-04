//! Async support that uses [`tokio::task::spawn_blocking`] and its I/O traits.

use crate::{async_support, async_support::BlockingExecutor, Ownership, Result};
use async_trait::async_trait;
use std::path::Path;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::compat::{Tokio02AsyncReadCompatExt, Tokio02AsyncWriteCompatExt};

struct TokioBlockingExecutor {}

#[async_trait]
impl BlockingExecutor for TokioBlockingExecutor {
    async fn execute_blocking<T, F>(f: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static,
    {
        tokio::task::spawn_blocking(f).await.map_err(Into::into)
    }
}

const TOKIO_BLOCKING_EXECUTOR: TokioBlockingExecutor = TokioBlockingExecutor {};

/// Async version of [`list_archive_files`](crate::list_archive_files).
pub async fn list_archive_files<R>(source: R) -> Result<Vec<String>>
where
    R: AsyncRead + Unpin,
{
    async_support::list_archive_files(TOKIO_BLOCKING_EXECUTOR, source.compat()).await
}

/// Async version of [`uncompress_data`](crate::uncompress_data).
pub async fn uncompress_data<R, W>(source: R, target: W) -> Result<usize>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    async_support::uncompress_data(
        TOKIO_BLOCKING_EXECUTOR,
        source.compat(),
        target.compat_write(),
    )
    .await
}

/// Async version of [`uncompress_archive`](crate::uncompress_archive).
pub async fn uncompress_archive<R>(source: R, dest: &Path, ownership: Ownership) -> Result<()>
where
    R: AsyncRead + Unpin,
{
    async_support::uncompress_archive(TOKIO_BLOCKING_EXECUTOR, source.compat(), dest, ownership)
        .await
}

/// Async version of
/// [`uncompress_archive_file`](crate::uncompress_archive_file).
pub async fn uncompress_archive_file<R, W>(source: R, target: W, path: &str) -> Result<usize>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    async_support::uncompress_archive_file(
        TOKIO_BLOCKING_EXECUTOR,
        source.compat(),
        target.compat_write(),
        path,
    )
    .await
}
