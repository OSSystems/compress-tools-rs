// Copyright (C) 2019-2021 O.S. Systems Software LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `compress-tools` crate aims to provide a convenient and easy to use set
//! of methods which builds on top of `libarchive` exposing a small set of it’s
//! functionalities.
//!
//! | Platform | Build Status |
//! | -------- | ------------ |
//! | Linux - x86_64 | [![build status](https://github.com/OSSystems/compress-tools-rs/workflows/CI%20-%20Linux%20-%20x86_64/badge.svg)](https://github.com/OSSystems/compress-tools-rs/actions) |
//! | Linux - AArch64 | [![build status](https://github.com/OSSystems/compress-tools-rs/workflows/CI%20-%20Linux%20-%20AArch64/badge.svg)](https://github.com/OSSystems/compress-tools-rs/actions) |
//! | Linux - ARMv7 | [![build status](https://github.com/OSSystems/compress-tools-rs/workflows/CI%20-%20Linux%20-%20ARMv7/badge.svg)](https://github.com/OSSystems/compress-tools-rs/actions) |
//! | macOS - x86_64 | [![build status](https://github.com/OSSystems/compress-tools-rs/workflows/CI%20-%20macOS%20-%20x86_64/badge.svg)](https://github.com/OSSystems/compress-tools-rs/actions) |
//! | Windows - x86_64 | [![build status](https://github.com/OSSystems/compress-tools-rs/workflows/CI%20-%20Windows%20-%20x86_64/badge.svg)](https://github.com/OSSystems/compress-tools-rs/actions) |
//!
//! ---
//!
//! # Dependencies
//!
//! You must have `libarchive`, 3.2.0 or newer, properly installed on your
//! system in order to use this. If building on *nix and Windows GNU
//! systems, `pkg-config` is used to locate the `libarchive`; on Windows
//! MSVC, `vcpkg` will be used to locating the `libarchive`.
//!
//! The minimum supported Rust version is 1.46.
//!
//! # Features
//!
//! This crate is capable of extracting:
//!
//! * compressed files
//! * archive files
//! * single file from an archive
//!
//! For example, to extract an archive file it is as simple as:
//!
//! ```no_run
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! use compress_tools::*;
//! use std::fs::File;
//! use std::path::Path;
//!
//! let mut source = File::open("tree.tar.gz")?;
//! let dest = Path::new("/tmp/dest");
//!
//! uncompress_archive(&mut source, &dest, Ownership::Preserve)?;
//! # Ok(())
//! # }
//! ```

#[cfg(feature = "async_support")]
pub mod async_support;
mod error;
mod ffi;
#[cfg(feature = "futures_support")]
pub mod futures_support;
#[cfg(feature = "tokio_support")]
pub mod tokio_support;

use error::archive_result;
pub use error::{Error, Result};
use io::{Seek, SeekFrom};
use std::{
    ffi::{CStr, CString},
    io::{self, Read, Write},
    os::raw::{c_int, c_void},
    path::Path,
    slice,
};

const READER_BUFFER_SIZE: usize = 1024;

/// Determine the ownership behavior when unpacking the archive.
pub enum Ownership {
    /// Preserve the ownership of the files when uncompressing the archive.
    Preserve,
    /// Ignore the ownership information of the files when uncompressing the
    /// archive.
    Ignore,
}

struct ReaderPipe<'a> {
    reader: &'a mut dyn Read,
    buffer: &'a mut [u8],
}

trait ReadAndSeek: Read + Seek {}
impl<T> ReadAndSeek for T where T: Read + Seek {}

struct SeekableReaderPipe<'a> {
    reader: &'a mut dyn ReadAndSeek,
    buffer: &'a mut [u8],
}

/// Get all files in a archive using `source` as a reader.
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use compress_tools::*;
/// use std::fs::File;
///
/// let mut source = File::open("tree.tar")?;
///
/// let file_list = list_archive_files(&mut source)?;
/// # Ok(())
/// # }
/// ```
pub fn list_archive_files<R>(source: R) -> Result<Vec<String>>
where
    R: Read + Seek,
{
    let _utf8_guard = ffi::UTF8LocaleGuard::new();
    run_with_archive(
        Ownership::Ignore,
        source,
        |archive_reader, _, mut entry| unsafe {
            let mut file_list = Vec::new();
            #[allow(clippy::vec_init_then_push)]
            loop {
                match ffi::archive_read_next_header(archive_reader, &mut entry) {
                    ffi::ARCHIVE_OK => {
                        file_list.push(
                            CStr::from_ptr(ffi::archive_entry_pathname(entry))
                                .to_string_lossy()
                                .into_owned(),
                        );
                    }
                    ffi::ARCHIVE_EOF => return Ok(file_list),
                    _ => return Err(Error::from(archive_reader)),
                }
            }
        },
    )
}

/// Uncompress a file using the `source` need as reader and the `target` as a
/// writer.
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use compress_tools::*;
/// use std::fs::File;
///
/// let mut source = File::open("file.txt.gz")?;
/// let mut target = Vec::default();
///
/// uncompress_data(&mut source, &mut target)?;
/// # Ok(())
/// # }
/// ```
///
/// Slices can be used if you know the exact length of the uncompressed data.
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use compress_tools::*;
/// use std::fs::File;
///
/// let mut source = File::open("file.txt.gz")?;
/// let mut target = [0 as u8; 313];
///
/// uncompress_data(&mut source, &mut target as &mut [u8])?;
/// # Ok(())
/// # }
/// ```
pub fn uncompress_data<R, W>(source: R, target: W) -> Result<usize>
where
    R: Read,
    W: Write,
{
    let _utf8_guard = ffi::UTF8LocaleGuard::new();
    run_with_unseekable_archive(source, |archive_reader, _, mut entry| unsafe {
        archive_result(
            ffi::archive_read_next_header(archive_reader, &mut entry),
            archive_reader,
        )?;
        libarchive_write_data_block(archive_reader, target)
    })
}

/// Uncompress an archive using `source` as a reader and `dest` as the
/// destination directory.
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use compress_tools::*;
/// use std::fs::File;
/// use std::path::Path;
///
/// let mut source = File::open("tree.tar.gz")?;
/// let dest = Path::new("/tmp/dest");
///
/// uncompress_archive(&mut source, &dest, Ownership::Preserve)?;
/// # Ok(())
/// # }
/// ```
pub fn uncompress_archive<R>(source: R, dest: &Path, ownership: Ownership) -> Result<()>
where
    R: Read + Seek,
{
    let _utf8_guard = ffi::UTF8LocaleGuard::new();
    run_with_archive(
        ownership,
        source,
        |archive_reader, archive_writer, mut entry| unsafe {
            loop {
                match ffi::archive_read_next_header(archive_reader, &mut entry) {
                    ffi::ARCHIVE_EOF => return Ok(()),
                    ffi::ARCHIVE_OK => {
                        let target_path = CString::new(
                            dest.join(
                                CStr::from_ptr(ffi::archive_entry_pathname(entry))
                                    .to_string_lossy()
                                    .into_owned(),
                            )
                            .to_str()
                            .unwrap(),
                        )
                        .unwrap();

                        ffi::archive_entry_set_pathname(entry, target_path.as_ptr());

                        let link_name = ffi::archive_entry_hardlink(entry);
                        if !link_name.is_null() {
                            let target_path = CString::new(
                                dest.join(CStr::from_ptr(link_name).to_string_lossy().into_owned())
                                    .to_str()
                                    .unwrap(),
                            )
                            .unwrap();

                            ffi::archive_entry_set_hardlink(entry, target_path.as_ptr());
                        }

                        ffi::archive_write_header(archive_writer, entry);
                        libarchive_copy_data(archive_reader, archive_writer)?;

                        if ffi::archive_write_finish_entry(archive_writer) != ffi::ARCHIVE_OK {
                            return Err(Error::from(archive_writer));
                        }
                    }
                    _ => return Err(Error::from(archive_reader)),
                }
            }
        },
    )
}

/// Uncompress a specific file from an archive. The `source` is used as a
/// reader, the `target` as a writer and the `path` is the relative path for
/// the file to be extracted from the archive.
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use compress_tools::*;
/// use std::fs::File;
///
/// let mut source = File::open("tree.tar.gz")?;
/// let mut target = Vec::default();
///
/// uncompress_archive_file(&mut source, &mut target, "file/path")?;
/// # Ok(())
/// # }
/// ```
pub fn uncompress_archive_file<R, W>(source: R, target: W, path: &str) -> Result<usize>
where
    R: Read + Seek,
    W: Write,
{
    let _utf8_guard = ffi::UTF8LocaleGuard::new();
    run_with_archive(
        Ownership::Ignore,
        source,
        |archive_reader, _, mut entry| unsafe {
            loop {
                match ffi::archive_read_next_header(archive_reader, &mut entry) {
                    ffi::ARCHIVE_OK => {
                        let file_name = CStr::from_ptr(ffi::archive_entry_pathname(entry))
                            .to_string_lossy()
                            .into_owned();
                        if file_name == path {
                            break;
                        }
                    }
                    ffi::ARCHIVE_EOF => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("path {} doesn't exist inside archive", path),
                        )
                        .into())
                    }
                    _ => return Err(Error::from(archive_reader)),
                }
            }
            libarchive_write_data_block(archive_reader, target)
        },
    )
}

fn run_with_archive<F, R, T>(ownership: Ownership, mut reader: R, f: F) -> Result<T>
where
    F: FnOnce(*mut ffi::archive, *mut ffi::archive, *mut ffi::archive_entry) -> Result<T>,
    R: Read + Seek,
{
    let _utf8_guard = ffi::UTF8LocaleGuard::new();
    unsafe {
        let archive_entry: *mut ffi::archive_entry = std::ptr::null_mut();
        let archive_reader = ffi::archive_read_new();
        let archive_writer = ffi::archive_write_disk_new();

        let res = (|| {
            archive_result(
                ffi::archive_read_support_filter_all(archive_reader),
                archive_reader,
            )?;

            archive_result(
                ffi::archive_read_support_format_raw(archive_reader),
                archive_reader,
            )?;

            archive_result(
                ffi::archive_read_set_seek_callback(archive_reader, Some(libarchive_seek_callback)),
                archive_reader,
            )?;

            let mut writer_flags = ffi::ARCHIVE_EXTRACT_TIME
                | ffi::ARCHIVE_EXTRACT_PERM
                | ffi::ARCHIVE_EXTRACT_ACL
                | ffi::ARCHIVE_EXTRACT_FFLAGS
                | ffi::ARCHIVE_EXTRACT_XATTR;

            if let Ownership::Preserve = ownership {
                writer_flags |= ffi::ARCHIVE_EXTRACT_OWNER;
            };

            archive_result(
                ffi::archive_write_disk_set_options(archive_writer, writer_flags as i32),
                archive_writer,
            )?;
            archive_result(
                ffi::archive_write_disk_set_standard_lookup(archive_writer),
                archive_writer,
            )?;
            archive_result(
                ffi::archive_read_support_format_all(archive_reader),
                archive_reader,
            )?;

            if archive_reader.is_null() || archive_writer.is_null() {
                return Err(Error::NullArchive);
            }

            let mut pipe = SeekableReaderPipe {
                reader: &mut reader,
                buffer: &mut [0; READER_BUFFER_SIZE],
            };

            archive_result(
                ffi::archive_read_open(
                    archive_reader,
                    (&mut pipe as *mut SeekableReaderPipe) as *mut c_void,
                    None,
                    Some(libarchive_seekable_read_callback),
                    None,
                ),
                archive_reader,
            )?;

            f(archive_reader, archive_writer, archive_entry)
        })();

        archive_result(ffi::archive_read_close(archive_reader), archive_reader)?;
        archive_result(ffi::archive_read_free(archive_reader), archive_reader)?;

        archive_result(ffi::archive_write_close(archive_writer), archive_writer)?;
        archive_result(ffi::archive_write_free(archive_writer), archive_writer)?;

        ffi::archive_entry_free(archive_entry);

        res
    }
}

fn run_with_unseekable_archive<F, R, T>(mut reader: R, f: F) -> Result<T>
where
    F: FnOnce(*mut ffi::archive, *mut ffi::archive, *mut ffi::archive_entry) -> Result<T>,
    R: Read,
{
    let _utf8_guard = ffi::UTF8LocaleGuard::new();
    unsafe {
        let archive_entry: *mut ffi::archive_entry = std::ptr::null_mut();
        let archive_reader = ffi::archive_read_new();
        let archive_writer = ffi::archive_write_disk_new();

        let res = (|| {
            archive_result(
                ffi::archive_read_support_filter_all(archive_reader),
                archive_reader,
            )?;

            archive_result(
                ffi::archive_read_support_format_raw(archive_reader),
                archive_reader,
            )?;

            if archive_reader.is_null() || archive_writer.is_null() {
                return Err(Error::NullArchive);
            }

            let mut pipe = ReaderPipe {
                reader: &mut reader,
                buffer: &mut [0; READER_BUFFER_SIZE],
            };

            archive_result(
                ffi::archive_read_open(
                    archive_reader,
                    (&mut pipe as *mut ReaderPipe) as *mut c_void,
                    None,
                    Some(libarchive_read_callback),
                    None,
                ),
                archive_reader,
            )?;

            f(archive_reader, archive_writer, archive_entry)
        })();

        archive_result(ffi::archive_read_close(archive_reader), archive_reader)?;
        archive_result(ffi::archive_read_free(archive_reader), archive_reader)?;

        archive_result(ffi::archive_write_close(archive_writer), archive_writer)?;
        archive_result(ffi::archive_write_free(archive_writer), archive_writer)?;

        ffi::archive_entry_free(archive_entry);

        res
    }
}

fn libarchive_copy_data(
    archive_reader: *mut ffi::archive,
    archive_writer: *mut ffi::archive,
) -> Result<()> {
    let mut buffer = std::ptr::null();
    let mut offset = 0;
    let mut size = 0;

    unsafe {
        loop {
            match ffi::archive_read_data_block(archive_reader, &mut buffer, &mut size, &mut offset)
            {
                ffi::ARCHIVE_EOF => return Ok(()),
                ffi::ARCHIVE_OK => {
                    archive_result(
                        ffi::archive_write_data_block(archive_writer, buffer, size, offset) as i32,
                        archive_writer,
                    )?;
                }
                _ => return Err(Error::from(archive_reader)),
            }
        }
    }
}

unsafe fn libarchive_write_data_block<W>(
    archive_reader: *mut ffi::archive,
    mut target: W,
) -> Result<usize>
where
    W: Write,
{
    let mut buffer = std::ptr::null();
    let mut offset = 0;
    let mut size = 0;
    let mut written = 0;

    loop {
        match ffi::archive_read_data_block(archive_reader, &mut buffer, &mut size, &mut offset) {
            ffi::ARCHIVE_EOF => return Ok(written),
            ffi::ARCHIVE_OK => {
                let content = slice::from_raw_parts(buffer as *const u8, size);
                target.write_all(content)?;
                written += size;
            }
            _ => return Err(Error::from(archive_reader)),
        }
    }
}

unsafe extern "C" fn libarchive_seek_callback(
    _: *mut ffi::archive,
    client_data: *mut c_void,
    offset: ffi::la_int64_t,
    whence: c_int,
) -> i64 {
    let pipe = (client_data as *mut SeekableReaderPipe).as_mut().unwrap();
    let whence = match whence {
        0 => SeekFrom::Start(offset as u64),
        1 => SeekFrom::Current(offset),
        2 => SeekFrom::End(offset),
        _ => return -1,
    };

    match pipe.reader.seek(whence) {
        Ok(offset) => offset as i64,
        Err(_) => -1,
    }
}

unsafe extern "C" fn libarchive_seekable_read_callback(
    archive: *mut ffi::archive,
    client_data: *mut c_void,
    buffer: *mut *const c_void,
) -> ffi::la_ssize_t {
    let pipe = (client_data as *mut SeekableReaderPipe).as_mut().unwrap();

    *buffer = pipe.buffer.as_ptr() as *const c_void;

    match pipe.reader.read(&mut pipe.buffer) {
        Ok(size) => size as ffi::la_ssize_t,
        Err(e) => {
            let description = CString::new(e.to_string()).unwrap();

            ffi::archive_set_error(archive, e.raw_os_error().unwrap_or(0), description.as_ptr());

            -1
        }
    }
}

unsafe extern "C" fn libarchive_read_callback(
    archive: *mut ffi::archive,
    client_data: *mut c_void,
    buffer: *mut *const c_void,
) -> ffi::la_ssize_t {
    let pipe = (client_data as *mut ReaderPipe).as_mut().unwrap();

    *buffer = pipe.buffer.as_ptr() as *const c_void;

    match pipe.reader.read(&mut pipe.buffer) {
        Ok(size) => size as ffi::la_ssize_t,
        Err(e) => {
            let description = CString::new(e.to_string()).unwrap();

            ffi::archive_set_error(archive, e.raw_os_error().unwrap_or(0), description.as_ptr());

            -1
        }
    }
}
