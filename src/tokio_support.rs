//! Async support that uses [`tokio::task::spawn_blocking`] and its I/O traits.

use crate::{
    async_support::{
        self, new_async_archive_iterator, AsyncArchiveIterator, AsyncEntryFilterCallbackFn,
        BlockingExecutor,
    },
    ArchivePassword, DecodeCallback, Ownership, Result,
};
use async_trait::async_trait;
use std::path::Path;
use tokio::io::{AsyncRead, AsyncSeek, AsyncWrite};
use tokio_util::compat::{TokioAsyncReadCompatExt, TokioAsyncWriteCompatExt};

pub(crate) struct TokioBlockingExecutor {}

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

/// Async version of
/// [`list_archive_files_with_encoding`](crate::
/// list_archive_files_with_encoding).
pub async fn list_archive_files_with_encoding<R>(
    source: R,
    decode: DecodeCallback,
) -> Result<Vec<String>>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    async_support::list_archive_files_with_encoding(
        TOKIO_BLOCKING_EXECUTOR,
        source.compat(),
        decode,
    )
    .await
}

/// Async version of [`list_archive_files`](crate::list_archive_files).
pub async fn list_archive_files<R>(source: R) -> Result<Vec<String>>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    async_support::list_archive_files(TOKIO_BLOCKING_EXECUTOR, source.compat()).await
}

/// Async version of
/// [`list_archive_entries_with_encoding`](crate::
/// list_archive_entries_with_encoding).
pub async fn list_archive_entries_with_encoding<R>(
    source: R,
    decode: DecodeCallback,
) -> Result<Vec<crate::ArchiveEntryInfo>>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    async_support::list_archive_entries_with_encoding(
        TOKIO_BLOCKING_EXECUTOR,
        source.compat(),
        decode,
    )
    .await
}

/// Async version of [`list_archive_entries`](crate::list_archive_entries).
pub async fn list_archive_entries<R>(source: R) -> Result<Vec<crate::ArchiveEntryInfo>>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    async_support::list_archive_entries(TOKIO_BLOCKING_EXECUTOR, source.compat()).await
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

/// Async version of
/// [`uncompress_archive_with_encoding`](crate::
/// uncompress_archive_with_encoding).
pub async fn uncompress_archive_with_encoding<R>(
    source: R,
    dest: &Path,
    ownership: Ownership,
    decode: DecodeCallback,
) -> Result<()>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    async_support::uncompress_archive_with_encoding(
        TOKIO_BLOCKING_EXECUTOR,
        source.compat(),
        dest,
        ownership,
        decode,
    )
    .await
}

/// Async version of [`uncompress_archive`](crate::uncompress_archive).
pub async fn uncompress_archive<R>(source: R, dest: &Path, ownership: Ownership) -> Result<()>
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    async_support::uncompress_archive(TOKIO_BLOCKING_EXECUTOR, source.compat(), dest, ownership)
        .await
}

/// Async version of
/// [`uncompress_archive_file_with_encoding`](crate::
/// uncompress_archive_file_with_encoding).
pub async fn uncompress_archive_file_with_encoding<R, W>(
    source: R,
    target: W,
    path: &str,
    decode: DecodeCallback,
) -> Result<usize>
where
    R: AsyncRead + AsyncSeek + Unpin,
    W: AsyncWrite + Unpin,
{
    async_support::uncompress_archive_file_with_encoding(
        TOKIO_BLOCKING_EXECUTOR,
        source.compat(),
        target.compat_write(),
        path,
        decode,
    )
    .await
}

/// Async version of
/// [`uncompress_archive_file`](crate::uncompress_archive_file).
pub async fn uncompress_archive_file<R, W>(source: R, target: W, path: &str) -> Result<usize>
where
    R: AsyncRead + AsyncSeek + Unpin,
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

// ---------------------------------------------------------------------------
// Async archive iterator
// ---------------------------------------------------------------------------

/// Builder for a tokio-backed [`AsyncArchiveIterator`].
///
/// Mirrors [`crate::ArchiveIteratorBuilder`] but produces an asynchronous
/// [`futures_core::Stream`] of entries over an `AsyncRead + AsyncSeek`
/// source. The source must be `Send + 'static` because it is moved into a
/// blocking worker that hosts the sync libarchive state for the iterator's
/// lifetime.
#[must_use]
pub struct ArchiveIteratorBuilder<R> {
    source: R,
    decoder: DecodeCallback,
    filter: Option<Box<AsyncEntryFilterCallbackFn>>,
    password: Option<ArchivePassword>,
}

impl<R> ArchiveIteratorBuilder<R>
where
    R: AsyncRead + AsyncSeek + Unpin + Send + 'static,
{
    pub fn new(source: R) -> ArchiveIteratorBuilder<R> {
        ArchiveIteratorBuilder {
            source,
            decoder: crate::decode_utf8,
            filter: None,
            password: None,
        }
    }

    pub fn decoder(mut self, decoder: DecodeCallback) -> ArchiveIteratorBuilder<R> {
        self.decoder = decoder;
        self
    }

    pub fn filter<F>(mut self, filter: F) -> ArchiveIteratorBuilder<R>
    where
        F: Fn(&str, &crate::stat) -> bool + Send + Sync + 'static,
    {
        self.filter = Some(Box::new(filter));
        self
    }

    pub fn with_password(mut self, password: ArchivePassword) -> ArchiveIteratorBuilder<R> {
        self.password = Some(password);
        self
    }

    pub fn build(self) -> AsyncArchiveIterator {
        new_async_archive_iterator::<TokioBlockingExecutor, _>(
            self.source.compat(),
            self.decoder,
            self.filter,
            self.password,
        )
    }
}
