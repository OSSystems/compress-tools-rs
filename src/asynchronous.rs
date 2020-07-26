use crate::{
    error::{archive_result, Error, Result},
    ffi, Mode,
};
use futures::{Future, FutureExt};
use std::{
    ffi::{CStr, CString},
    os::raw::c_void,
    slice,
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const ASYNC_READER_BUFFER_SIZE: usize = 1024;

struct AsyncPipe<'a> {
    reader: &'a mut (dyn AsyncRead + Unpin),
    buffer: &'a mut [u8],
}

/// Get all files in a archive using `source` as a reader.
/// # Example
///
/// ```no_run
/// # async fn test() -> Result<(), Box<dyn std::error::Error>> {
/// use compress_tools::asynchronous::*;
/// use tokio::fs::File;
///
/// let mut source = File::open("tree.tar").await?;
///
/// let file_list = list_archive_files(&mut source).await?;
/// # Ok(())
/// # }
/// ```
pub async fn list_archive_files<R>(source: R) -> Result<Vec<String>>
where
    R: AsyncRead + Unpin,
{
    run_with_archive(
        Mode::AllFormat,
        source,
        |archive_reader, mut entry| unsafe {
            let mut file_list = Vec::new();
            loop {
                match ffi::archive_read_next_header(archive_reader, &mut entry) {
                    ffi::ARCHIVE_OK => {
                        if let Err(e) = CStr::from_ptr(ffi::archive_entry_pathname(entry))
                            .to_str()
                            .map(|s| file_list.push(s.to_string()))
                        {
                            return futures::future::err(Error::Utf(e));
                        }
                    }
                    ffi::ARCHIVE_EOF => return futures::future::ok(file_list),
                    _ => return futures::future::err(Error::from(archive_reader)),
                }
            }
        },
    )
    .await
}

/// Uncompress a file using the `source` need as reader and the `target` as a
/// writer.
///
/// # Example
///
/// ```no_run
/// # async fn test() -> Result<(), Box<dyn std::error::Error>> {
/// use compress_tools::asynchronous::*;
/// use tokio::fs::File;
///
/// let mut source = File::open("file.txt.gz").await?;
/// let mut target = Vec::default();
///
/// uncompress_data(&mut source, &mut target).await?;
/// # Ok(())
/// # }
/// ```
pub async fn uncompress_data<R, W>(source: R, target: W) -> Result<usize>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + std::marker::Unpin,
{
    run_with_archive(
        Mode::RawFormat,
        source,
        |archive_reader, mut entry| unsafe {
            if let Err(e) = archive_result(
                ffi::archive_read_next_header(archive_reader, &mut entry),
                archive_reader,
            ) {
                return futures::future::err(e).boxed_local();
            }
            libarchive_write_data_block_async(archive_reader, target).boxed_local()
        },
    )
    .await
}

/// Uncompress a specific file from an archive. The `source` is used as a
/// reader, the `target` as a writer and the `path` is the relative path for
/// the file to be extracted from the archive.
///
/// # Example
///
/// ```no_run
/// # async fn test() -> Result<(), Box<dyn std::error::Error>> {
/// use compress_tools::asynchronous::*;
/// use tokio::fs::File;
///
/// let mut source = File::open("tree.tar.gz").await?;
/// let mut target = Vec::default();
///
/// uncompress_archive_file(&mut source, &mut target, "file/path").await?;
/// # Ok(())
/// # }
/// ```
pub async fn uncompress_archive_file<R, W>(source: R, target: W, path: &str) -> Result<usize>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + std::marker::Unpin,
{
    run_with_archive(
        Mode::AllFormat,
        source,
        |archive_reader, mut entry| unsafe {
            loop {
                match ffi::archive_read_next_header(archive_reader, &mut entry) {
                    ffi::ARCHIVE_OK => {
                        let file_name = CStr::from_ptr(ffi::archive_entry_pathname(entry)).to_str();
                        if file_name.is_err() {
                            return futures::future::err(Error::Utf(file_name.err().unwrap()))
                                .boxed_local();
                        } else if file_name.unwrap() == path {
                            break;
                        }
                    }
                    ffi::ARCHIVE_EOF => {
                        return futures::future::err(Error::FileNotFound).boxed_local()
                    }
                    _ => return futures::future::err(Error::from(archive_reader)).boxed_local(),
                }
            }
            libarchive_write_data_block_async(archive_reader, target).boxed_local()
        },
    )
    .await
}

async fn run_with_archive<F, R, T, I>(mode: Mode, mut reader: R, f: F) -> Result<T>
where
    F: FnOnce(*mut ffi::archive, *mut ffi::archive_entry) -> I,
    R: AsyncRead + Unpin,
    I: Future<Output = Result<T>>,
{
    let archive_reader: *mut ffi::archive;
    let archive_entry: *mut ffi::archive_entry = std::ptr::null_mut();

    unsafe {
        archive_reader = ffi::archive_read_new();
        archive_result(
            ffi::archive_read_support_filter_all(archive_reader),
            archive_reader,
        )?;
        #[allow(unused_variables)]
        match mode {
            Mode::RawFormat => archive_result(
                ffi::archive_read_support_format_raw(archive_reader),
                archive_reader,
            )?,
            Mode::AllFormat => archive_result(
                ffi::archive_read_support_format_all(archive_reader),
                archive_reader,
            )?,
            Mode::WriteDisk { ownership } => return Err(Error::AsyncDiskAccessNotSupported),
        }

        if archive_reader.is_null() {
            return Err(Error::NullArchive);
        }

        let mut pipe = AsyncPipe {
            reader: &mut reader,
            buffer: &mut [0; ASYNC_READER_BUFFER_SIZE],
        };

        archive_result(
            ffi::archive_read_open(
                archive_reader,
                (&mut pipe as *mut AsyncPipe) as *mut c_void,
                None,
                Some(libarchive_read_callback),
                None,
            ),
            archive_reader,
        )?;

        let res = f(archive_reader, archive_entry).await?;

        archive_result(ffi::archive_read_close(archive_reader), archive_reader)?;
        archive_result(ffi::archive_read_free(archive_reader), archive_reader)?;
        ffi::archive_entry_free(archive_entry);

        Ok(res)
    }
}

async unsafe fn libarchive_write_data_block_async<W>(
    archive_reader: *mut ffi::archive,
    mut target: W,
) -> Result<usize>
where
    W: AsyncWrite + std::marker::Unpin,
{
    let mut written = 0;

    let mut buffer = std::ptr::null();
    let mut offset = 0;
    let mut size = 0;
    loop {
        match ffi::archive_read_data_block(archive_reader, &mut buffer, &mut size, &mut offset) {
            ffi::ARCHIVE_EOF => return Ok(written),
            ffi::ARCHIVE_OK => {
                let content = slice::from_raw_parts(buffer as *const u8, size);
                written += size;
                target.write_all(content).await?;
            }
            _ => return Err(Error::from(archive_reader)),
        }
    }
}

unsafe extern "C" fn libarchive_read_callback(
    archive: *mut ffi::archive,
    client_data: *mut c_void,
    buffer: *mut *const c_void,
) -> ffi::la_ssize_t {
    futures::executor::block_on(libarchive_read_callback_async(archive, client_data, buffer))
}

async unsafe extern "C" fn libarchive_read_callback_async(
    archive: *mut ffi::archive,
    client_data: *mut c_void,
    buffer: *mut *const c_void,
) -> ffi::la_ssize_t {
    let pipe = (client_data as *mut AsyncPipe).as_mut().unwrap();

    *buffer = pipe.buffer.as_ptr() as *const c_void;

    match pipe.reader.read(&mut pipe.buffer).await {
        Ok(size) => size as ffi::la_ssize_t,
        Err(e) => {
            let description = CString::new(e.to_string()).unwrap();

            ffi::archive_set_error(archive, e.raw_os_error().unwrap_or(0), description.as_ptr());

            -1
        }
    }
}
