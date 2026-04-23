// Copyright (C) 2019-2021 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Generic async support with which you can use you own thread pool by
//! implementing the [`BlockingExecutor`] trait.

use crate::{
    ArchiveContents, ArchiveIteratorBuilder, ArchivePassword, DecodeCallback, Ownership, Result,
    READER_BUFFER_SIZE,
};
use async_trait::async_trait;
use futures_channel::mpsc::{channel, Receiver, Sender};
use futures_core::{FusedStream, Stream};
use futures_executor::block_on;
use futures_io::{AsyncRead, AsyncSeek, AsyncWrite};
use futures_util::{
    io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt},
    join,
    sink::SinkExt,
    stream::StreamExt,
};
use std::{
    future::Future,
    io::{ErrorKind, Read, Seek, SeekFrom, Write},
    path::Path,
    pin::Pin,
    task::{Context, Poll},
};

#[async_trait]
pub trait BlockingExecutor {
    /// Execute the provided function on a thread where blocking is acceptable
    /// (in some kind of thread pool).
    async fn execute_blocking<T, F>(f: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce() -> T + Send + 'static;
}

// ----------------------------------------------------------------------------
// Stream-only reader wrapper (used by `uncompress_data`, which never seeks)
// ----------------------------------------------------------------------------

struct AsyncReadWrapper {
    rx: Receiver<Vec<u8>>,
}

impl Read for AsyncReadWrapper {
    fn read(&mut self, mut buf: &mut [u8]) -> std::io::Result<usize> {
        if self.rx.is_terminated() {
            return Ok(0);
        }
        assert_eq!(buf.len(), READER_BUFFER_SIZE);
        Ok(match block_on(self.rx.next()) {
            Some(data) => {
                buf.write_all(&data)?;
                data.len()
            }
            None => 0,
        })
    }
}

fn make_async_read_wrapper_and_worker<R>(
    mut read: R,
) -> (AsyncReadWrapper, impl Future<Output = Result<()>>)
where
    R: AsyncRead + Unpin,
{
    let (mut tx, rx) = channel(0);
    (AsyncReadWrapper { rx }, async move {
        loop {
            let mut data = vec![0; READER_BUFFER_SIZE];
            let read = read.read(&mut data).await?;
            data.truncate(read);
            if read == 0 || tx.send(data).await.is_err() {
                break;
            }
        }
        Ok(())
    })
}

// ----------------------------------------------------------------------------
// Seekable read bridge (used by list / extract / iterator paths)
// ----------------------------------------------------------------------------
//
// libarchive's seekable formats (ZIP, 7z, …) issue `seek()` calls through the
// synchronous callback it registers with us. When the caller supplies an
// `AsyncRead + AsyncSeek` source, we cannot call `.await` from inside that
// C callback, so we stand up a request/response channel pair: the sync side
// sends a `BridgeReq` describing the desired operation and blocks on the
// matching `BridgeRes`, while an async worker future awaits the operation on
// the underlying source.

enum BridgeReq {
    Read(usize),
    Seek(SeekFrom),
}

enum BridgeRes {
    Read(std::io::Result<Vec<u8>>),
    Seek(std::io::Result<u64>),
}

pub(crate) struct SeekableAsyncReadWrapper {
    req_tx: Sender<BridgeReq>,
    res_rx: Receiver<BridgeRes>,
}

impl Read for SeekableAsyncReadWrapper {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if block_on(self.req_tx.send(BridgeReq::Read(buf.len()))).is_err() {
            return Ok(0);
        }
        match block_on(self.res_rx.next()) {
            Some(BridgeRes::Read(Ok(data))) => {
                let n = data.len().min(buf.len());
                buf[..n].copy_from_slice(&data[..n]);
                Ok(n)
            }
            Some(BridgeRes::Read(Err(e))) => Err(e),
            Some(BridgeRes::Seek(_)) | None => Ok(0),
        }
    }
}

impl Seek for SeekableAsyncReadWrapper {
    fn seek(&mut self, pos: SeekFrom) -> std::io::Result<u64> {
        if block_on(self.req_tx.send(BridgeReq::Seek(pos))).is_err() {
            return Err(std::io::Error::new(
                ErrorKind::BrokenPipe,
                "async seek bridge closed",
            ));
        }
        match block_on(self.res_rx.next()) {
            Some(BridgeRes::Seek(r)) => r,
            Some(BridgeRes::Read(_)) | None => Err(std::io::Error::new(
                ErrorKind::BrokenPipe,
                "async seek bridge closed",
            )),
        }
    }
}

fn make_seekable_read_wrapper_and_worker<R>(
    mut read: R,
) -> (SeekableAsyncReadWrapper, impl Future<Output = Result<()>>)
where
    R: AsyncRead + AsyncSeek + Unpin,
{
    let (req_tx, mut req_rx) = channel::<BridgeReq>(0);
    let (mut res_tx, res_rx) = channel::<BridgeRes>(0);
    let worker = async move {
        while let Some(req) = req_rx.next().await {
            let res = match req {
                BridgeReq::Read(n) => {
                    let mut buf = vec![0u8; n];
                    match read.read(&mut buf).await {
                        Ok(size) => {
                            buf.truncate(size);
                            BridgeRes::Read(Ok(buf))
                        }
                        Err(e) => BridgeRes::Read(Err(e)),
                    }
                }
                BridgeReq::Seek(pos) => BridgeRes::Seek(read.seek(pos).await),
            };
            if res_tx.send(res).await.is_err() {
                break;
            }
        }
        Ok(())
    };
    (SeekableAsyncReadWrapper { req_tx, res_rx }, worker)
}

// ----------------------------------------------------------------------------
// Write bridge (unchanged)
// ----------------------------------------------------------------------------

pub(crate) struct AsyncWriteWrapper {
    tx: Sender<Vec<u8>>,
}

impl Write for AsyncWriteWrapper {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        match block_on(self.tx.send(buf.to_owned())) {
            Ok(()) => Ok(buf.len()),
            Err(err) => Err(std::io::Error::new(ErrorKind::Other, err)),
        }
    }

    fn flush(&mut self) -> std::io::Result<()> {
        block_on(self.tx.send(vec![])).map_err(|err| std::io::Error::new(ErrorKind::Other, err))
    }
}

fn make_async_write_wrapper_and_worker<W>(
    mut write: W,
) -> (AsyncWriteWrapper, impl Future<Output = Result<()>>)
where
    W: AsyncWrite + Unpin,
{
    let (tx, mut rx) = channel(0);
    (AsyncWriteWrapper { tx }, async move {
        while let Some(v) = rx.next().await {
            if v.is_empty() {
                write.flush().await?;
            } else {
                write.write_all(&v).await?;
            }
        }
        Ok(())
    })
}

// ----------------------------------------------------------------------------
// High-level wrappers
// ----------------------------------------------------------------------------

async fn wrap_async_read_and_write<B, R, W, F, T>(_: B, read: R, write: W, f: F) -> Result<T>
where
    B: BlockingExecutor,
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
    F: FnOnce(AsyncReadWrapper, AsyncWriteWrapper) -> T + Send + 'static,
    T: Send + 'static,
{
    let (async_read_wrapper, async_read_wrapper_worker) = make_async_read_wrapper_and_worker(read);
    let (async_write_wrapper, async_write_wrapper_worker) =
        make_async_write_wrapper_and_worker(write);
    let g = B::execute_blocking(move || f(async_read_wrapper, async_write_wrapper));
    let join = join!(async_read_wrapper_worker, async_write_wrapper_worker, g);
    join.0?;
    join.1?;
    join.2
}

async fn wrap_async_seek_read<B, R, F, T>(_: B, read: R, f: F) -> Result<T>
where
    B: BlockingExecutor,
    R: AsyncRead + AsyncSeek + Unpin,
    F: FnOnce(SeekableAsyncReadWrapper) -> T + Send + 'static,
    T: Send + 'static,
{
    let (seekable_wrapper, seekable_worker) = make_seekable_read_wrapper_and_worker(read);
    let g = B::execute_blocking(move || f(seekable_wrapper));
    let join = join!(seekable_worker, g);
    join.0?;
    join.1
}

async fn wrap_async_seek_read_and_write<B, R, W, F, T>(_: B, read: R, write: W, f: F) -> Result<T>
where
    B: BlockingExecutor,
    R: AsyncRead + AsyncSeek + Unpin,
    W: AsyncWrite + Unpin,
    F: FnOnce(SeekableAsyncReadWrapper, AsyncWriteWrapper) -> T + Send + 'static,
    T: Send + 'static,
{
    let (seekable_wrapper, seekable_worker) = make_seekable_read_wrapper_and_worker(read);
    let (async_write_wrapper, async_write_wrapper_worker) =
        make_async_write_wrapper_and_worker(write);
    let g = B::execute_blocking(move || f(seekable_wrapper, async_write_wrapper));
    let join = join!(seekable_worker, async_write_wrapper_worker, g);
    join.0?;
    join.1?;
    join.2
}

// ----------------------------------------------------------------------------
// Public async entry points
// ----------------------------------------------------------------------------

/// Async version of
/// [`list_archive_files_with_encoding`](crate::
/// list_archive_files_with_encoding).
pub async fn list_archive_files_with_encoding<B, R>(
    blocking_executor: B,
    source: R,
    decode: DecodeCallback,
) -> Result<Vec<String>>
where
    B: BlockingExecutor,
    R: AsyncRead + AsyncSeek + Unpin,
{
    wrap_async_seek_read(blocking_executor, source, move |source| {
        crate::list_archive_files_with_encoding(source, decode)
    })
    .await?
}

/// Async version of [`list_archive_files`](crate::list_archive_files).
pub async fn list_archive_files<B, R>(blocking_executor: B, source: R) -> Result<Vec<String>>
where
    B: BlockingExecutor,
    R: AsyncRead + AsyncSeek + Unpin,
{
    wrap_async_seek_read(blocking_executor, source, crate::list_archive_files).await?
}

/// Async version of
/// [`list_archive_entries_with_encoding`](crate::
/// list_archive_entries_with_encoding).
pub async fn list_archive_entries_with_encoding<B, R>(
    blocking_executor: B,
    source: R,
    decode: DecodeCallback,
) -> Result<Vec<crate::ArchiveEntryInfo>>
where
    B: BlockingExecutor,
    R: AsyncRead + AsyncSeek + Unpin,
{
    wrap_async_seek_read(blocking_executor, source, move |source| {
        crate::list_archive_entries_with_encoding(source, decode)
    })
    .await?
}

/// Async version of [`list_archive_entries`](crate::list_archive_entries).
pub async fn list_archive_entries<B, R>(
    blocking_executor: B,
    source: R,
) -> Result<Vec<crate::ArchiveEntryInfo>>
where
    B: BlockingExecutor,
    R: AsyncRead + AsyncSeek + Unpin,
{
    wrap_async_seek_read(blocking_executor, source, crate::list_archive_entries).await?
}

/// Async version of [`uncompress_data`](crate::uncompress_data).
pub async fn uncompress_data<B, R, W>(blocking_executor: B, source: R, target: W) -> Result<usize>
where
    B: BlockingExecutor,
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    wrap_async_read_and_write(blocking_executor, source, target, |source, target| {
        crate::uncompress_data(source, target)
    })
    .await?
}

/// Async version of
/// [`uncompress_archive_with_encoding`](crate::
/// uncompress_archive_with_encoding).
pub async fn uncompress_archive_with_encoding<B, R>(
    blocking_executor: B,
    source: R,
    dest: &Path,
    ownership: Ownership,
    decode: DecodeCallback,
) -> Result<()>
where
    B: BlockingExecutor,
    R: AsyncRead + AsyncSeek + Unpin,
{
    let dest = dest.to_owned();
    wrap_async_seek_read(blocking_executor, source, move |source| {
        crate::uncompress_archive_with_encoding(source, &dest, ownership, decode)
    })
    .await?
}

/// Async version of [`uncompress_archive`](crate::uncompress_archive).
pub async fn uncompress_archive<B, R>(
    blocking_executor: B,
    source: R,
    dest: &Path,
    ownership: Ownership,
) -> Result<()>
where
    B: BlockingExecutor,
    R: AsyncRead + AsyncSeek + Unpin,
{
    let dest = dest.to_owned();
    wrap_async_seek_read(blocking_executor, source, move |source| {
        crate::uncompress_archive(source, &dest, ownership)
    })
    .await?
}

/// Async version of
/// [`uncompress_archive_file_with_encoding`](crate::
/// uncompress_archive_file_with_encoding).
pub async fn uncompress_archive_file_with_encoding<B, R, W>(
    blocking_executor: B,
    source: R,
    target: W,
    path: &str,
    decode: DecodeCallback,
) -> Result<usize>
where
    B: BlockingExecutor,
    R: AsyncRead + AsyncSeek + Unpin,
    W: AsyncWrite + Unpin,
{
    let path = path.to_owned();
    wrap_async_seek_read_and_write(blocking_executor, source, target, move |source, target| {
        crate::uncompress_archive_file_with_encoding(source, target, &path, decode)
    })
    .await?
}

/// Async version of
/// [`uncompress_archive_file`](crate::uncompress_archive_file).
pub async fn uncompress_archive_file<B, R, W>(
    blocking_executor: B,
    source: R,
    target: W,
    path: &str,
) -> Result<usize>
where
    B: BlockingExecutor,
    R: AsyncRead + AsyncSeek + Unpin,
    W: AsyncWrite + Unpin,
{
    let path = path.to_owned();
    wrap_async_seek_read_and_write(blocking_executor, source, target, move |source, target| {
        crate::uncompress_archive_file(source, target, &path)
    })
    .await?
}

// ----------------------------------------------------------------------------
// Async archive iterator
// ----------------------------------------------------------------------------

/// A filter callback for the async archive iterator.
///
/// Differs from the synchronous [`crate::EntryFilterCallbackFn`] only in that
/// it must be `Send + Sync` so that the filter can cross into the blocking
/// worker driving the sync iterator.
pub type AsyncEntryFilterCallbackFn = dyn Fn(&str, &crate::stat) -> bool + Send + Sync;

/// Asynchronous streaming iterator over the contents of an archive.
///
/// Yields [`ArchiveContents`] items in the same order and shape as the
/// synchronous [`ArchiveIterator`]. The sync iterator and its libarchive
/// state live on a dedicated blocking worker; entries are forwarded through
/// a bounded channel and surfaced through this [`Stream`] impl.
///
/// Polling this stream also drives the bridge worker future that services
/// the sync side's `read`/`seek` requests and the blocking pump's
/// `JoinHandle` — so progress only happens while the consumer polls.
/// Dropping the iterator closes the entry channel; the pump notices on its
/// next send and exits.
pub struct AsyncArchiveIterator {
    rx: Receiver<ArchiveContents>,
    worker: Option<Pin<Box<dyn Future<Output = Result<()>> + Send>>>,
    pump: Option<Pin<Box<dyn Future<Output = Result<()>> + Send>>>,
}

impl Stream for AsyncArchiveIterator {
    type Item = ArchiveContents;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;
        if let Some(worker) = this.worker.as_mut() {
            if let Poll::Ready(res) = worker.as_mut().poll(cx) {
                this.worker = None;
                if let Err(e) = res {
                    return Poll::Ready(Some(ArchiveContents::Err(e)));
                }
            }
        }
        if let Some(pump) = this.pump.as_mut() {
            if let Poll::Ready(res) = pump.as_mut().poll(cx) {
                this.pump = None;
                if let Err(e) = res {
                    return Poll::Ready(Some(ArchiveContents::Err(e)));
                }
            }
        }
        Pin::new(&mut this.rx).poll_next(cx)
    }
}

pub(crate) fn new_async_archive_iterator<B, R>(
    source: R,
    decode: DecodeCallback,
    filter: Option<Box<AsyncEntryFilterCallbackFn>>,
    password: Option<ArchivePassword>,
) -> AsyncArchiveIterator
where
    B: BlockingExecutor + 'static,
    R: AsyncRead + AsyncSeek + Unpin + Send + 'static,
{
    let (mut entry_tx, entry_rx) = channel::<ArchiveContents>(1);
    let (seekable_wrapper, seekable_worker) = make_seekable_read_wrapper_and_worker(source);

    let pump_fut = async move {
        let r: Result<()> = B::execute_blocking(move || -> Result<()> {
            let mut builder = ArchiveIteratorBuilder::new(seekable_wrapper).decoder(decode);
            if let Some(filter) = filter {
                builder = builder.filter(move |name, stat| filter(name, stat));
            }
            if let Some(password) = password {
                builder = builder.with_password(password);
            }
            let mut iter = builder.build()?;
            for content in iter.by_ref() {
                if block_on(entry_tx.send(content)).is_err() {
                    // Consumer dropped the receiver; stop forwarding and
                    // close the iterator so libarchive state is released
                    // promptly.
                    break;
                }
            }
            iter.close()
        })
        .await?;
        r
    };

    AsyncArchiveIterator {
        rx: entry_rx,
        worker: Some(Box::pin(seekable_worker)),
        pump: Some(Box::pin(pump_fut)),
    }
}
