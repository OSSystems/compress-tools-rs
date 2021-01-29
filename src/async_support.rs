// Copyright (C) 2019, 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! Generic async support with which you can use you own thread pool by
//! implementing the [`BlockingExecutor`] trait.

use crate::{Ownership, Result, READER_BUFFER_SIZE};
use async_trait::async_trait;
use futures_channel::mpsc::{channel, Receiver, Sender};
use futures_core::FusedStream;
use futures_executor::block_on;
use futures_io::{AsyncRead, AsyncWrite};
use futures_util::{
    io::{AsyncReadExt, AsyncWriteExt},
    join,
    sink::SinkExt,
    stream::StreamExt,
};
use std::{
    future::Future,
    io::{ErrorKind, Read, Seek, SeekFrom, Write},
    path::Path,
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

// Hints Rust compiler that the seek is indeed supported, but
// underlying, it is done by the libarchive_seek_callback() callback.
impl Seek for AsyncReadWrapper {
    fn seek(&mut self, _: SeekFrom) -> std::io::Result<u64> {
        unreachable!("We need to use libarchive_seek_callback() underlying.")
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

struct AsyncWriteWrapper {
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

async fn wrap_async_read<B, R, F, T>(_: B, read: R, f: F) -> Result<T>
where
    B: BlockingExecutor,
    R: AsyncRead + Unpin,
    F: FnOnce(AsyncReadWrapper) -> T + Send + 'static,
    T: Send + 'static,
{
    let (async_read_wrapper, async_read_wrapper_worker) = make_async_read_wrapper_and_worker(read);
    let g = B::execute_blocking(move || f(async_read_wrapper));
    let join = join!(async_read_wrapper_worker, g);
    join.0?;
    join.1
}

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

/// Async version of [`list_archive_files`](crate::list_archive_files).
pub async fn list_archive_files<B, R>(blocking_executor: B, source: R) -> Result<Vec<String>>
where
    B: BlockingExecutor,
    R: AsyncRead + Unpin,
{
    wrap_async_read(blocking_executor, source, crate::list_archive_files).await?
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

/// Async version of [`uncompress_archive`](crate::uncompress_archive).
pub async fn uncompress_archive<B, R>(
    blocking_executor: B,
    source: R,
    dest: &Path,
    ownership: Ownership,
) -> Result<()>
where
    B: BlockingExecutor,
    R: AsyncRead + Unpin,
{
    let dest = dest.to_owned();
    wrap_async_read(blocking_executor, source, move |source| {
        crate::uncompress_archive(source, &dest, ownership)
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
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let path = path.to_owned();
    wrap_async_read_and_write(blocking_executor, source, target, move |source, target| {
        crate::uncompress_archive_file(source, target, &path)
    })
    .await?
}
