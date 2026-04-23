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
//! The minimum supported Rust version is 1.59.
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
//!
//! # Strict archive parsing
//!
//! Archive-listing and archive-extraction entry points (`list_archive_files`,
//! `list_archive_entries`, `uncompress_archive`, `uncompress_archive_file`,
//! `ArchiveIterator`, and their async/`_with_encoding` siblings) no longer
//! register libarchive's "raw" format handler. They return an error for
//! input that isn't a real archive instead of yielding a single entry
//! called `data`, so callers can reliably distinguish archives from other
//! files.
//!
//! Use [`uncompress_data`] for decompressing a single stream (gzip, xz, …)
//! — it continues to support raw input because that is its purpose. For
//! streaming iteration that should accept arbitrary bytes, opt back in
//! with [`ArchiveIteratorBuilder::raw_format`].

#[cfg(feature = "async_support")]
pub mod async_support;
mod error;
mod ffi;
#[cfg(feature = "futures_support")]
pub mod futures_support;
mod iterator;
#[cfg(feature = "tokio_support")]
pub mod tokio_support;
mod zip_preflight;

use error::{archive_result, archive_result_strict};
pub use error::{Error, Result};
use io::{Seek, SeekFrom};
pub use iterator::{ArchiveContents, ArchiveIterator, ArchiveIteratorBuilder, ArchivePassword};
use std::{
    ffi::{CStr, CString},
    io::{self, Read, Write},
    os::raw::{c_int, c_void},
    path::{Component, Path},
    slice,
};

const READER_BUFFER_SIZE: usize = 16384;

/// Re-export of [`libc::stat`] so `crate::stat` resolves uniformly across
/// platforms — Windows has its own layout declared below.
#[cfg(not(target_os = "windows"))]
pub use libc::stat;

/// `stat` layout matching the one exposed by `libarchive` on Windows.
///
/// On Windows `libarchive`'s `archive_entry_stat()` returns a pointer to the
/// struct declared in `<sys/stat.h>`, which differs from `libc::stat` (the
/// latter is actually `stat64`). Using the wrong layout reads garbage for
/// `st_size` and the three `st_*time` fields.
#[cfg(target_os = "windows")]
#[derive(Copy, Clone)]
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct stat {
    pub st_dev: libc::dev_t,
    pub st_ino: libc::ino_t,
    pub st_mode: u16,
    pub st_nlink: libc::c_short,
    pub st_uid: libc::c_short,
    pub st_gid: libc::c_short,
    pub st_rdev: libc::dev_t,
    pub st_size: i32,
    pub st_atime: libc::time_t,
    pub st_mtime: libc::time_t,
    pub st_ctime: libc::time_t,
}

/// Path and uncompressed size for a single archive entry.
///
/// `size` comes from the archive header and may be `0` for formats that do
/// not record it there (some raw compressed streams, ZIP entries using a
/// data descriptor). Tar and standard ZIP populate it reliably.
#[derive(Clone, Debug)]
pub struct ArchiveEntryInfo {
    pub path: String,
    pub size: u64,
}

/// Determine the ownership behavior when unpacking the archive.
#[derive(Clone, Copy, Debug)]
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

pub type DecodeCallback = fn(&[u8]) -> Result<String>;

pub(crate) fn decode_utf8(bytes: &[u8]) -> Result<String> {
    Ok(std::str::from_utf8(bytes)?.to_owned())
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
/// let decode_utf8 = |bytes: &[u8]| Ok(std::str::from_utf8(bytes)?.to_owned());
///
/// let file_list = list_archive_files_with_encoding(&mut source, decode_utf8)?;
/// # Ok(())
/// # }
/// ```
pub fn list_archive_files_with_encoding<R>(source: R, decode: DecodeCallback) -> Result<Vec<String>>
where
    R: Read + Seek,
{
    Ok(list_archive_entries_with_encoding(source, decode)?
        .into_iter()
        .map(|e| e.path)
        .collect())
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
    list_archive_files_with_encoding(source, decode_utf8)
}

/// Get entry metadata (path and uncompressed size) for every entry in an
/// archive without extracting their contents.
///
/// See [`ArchiveEntryInfo`] for caveats on `size` reporting across formats.
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use compress_tools::*;
/// use std::fs::File;
///
/// let mut source = File::open("tree.tar")?;
/// let decode_utf8 = |bytes: &[u8]| Ok(std::str::from_utf8(bytes)?.to_owned());
///
/// for entry in list_archive_entries_with_encoding(&mut source, decode_utf8)? {
///     println!("{}: {} bytes", entry.path, entry.size);
/// }
/// # Ok(())
/// # }
/// ```
pub fn list_archive_entries_with_encoding<R>(
    source: R,
    decode: DecodeCallback,
) -> Result<Vec<ArchiveEntryInfo>>
where
    R: Read + Seek,
{
    let _utf8_guard = ffi::UTF8LocaleGuard::new();
    run_with_archive(
        Ownership::Ignore,
        source,
        |archive_reader, _, mut entry| unsafe {
            let mut entries = Vec::new();
            loop {
                match ffi::archive_read_next_header(archive_reader, &mut entry) {
                    ffi::ARCHIVE_EOF => return Ok(entries),
                    value => archive_result(value, archive_reader)?,
                }

                let _utf8_guard = ffi::WindowsUTF8LocaleGuard::new();
                let cstr = libarchive_entry_pathname(entry)?;
                let path = decode(cstr.to_bytes())?;
                let size = libarchive_entry_size(entry);
                entries.push(ArchiveEntryInfo { path, size });
            }
        },
    )
}

/// Get entry metadata (path and uncompressed size) for every entry in an
/// archive without extracting their contents.
///
/// See [`ArchiveEntryInfo`] for caveats on `size` reporting across formats.
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use compress_tools::*;
/// use std::fs::File;
///
/// let mut source = File::open("tree.tar")?;
///
/// for entry in list_archive_entries(&mut source)? {
///     println!("{}: {} bytes", entry.path, entry.size);
/// }
/// # Ok(())
/// # }
/// ```
pub fn list_archive_entries<R>(source: R) -> Result<Vec<ArchiveEntryInfo>>
where
    R: Read + Seek,
{
    list_archive_entries_with_encoding(source, decode_utf8)
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
/// let decode_utf8 = |bytes: &[u8]| Ok(std::str::from_utf8(bytes)?.to_owned());
///
/// uncompress_archive_with_encoding(&mut source, &dest, Ownership::Preserve, decode_utf8)?;
/// # Ok(())
/// # }
/// ```
pub fn uncompress_archive_with_encoding<R>(
    source: R,
    dest: &Path,
    ownership: Ownership,
    decode: DecodeCallback,
) -> Result<()>
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
                    value => archive_result(value, archive_reader)?,
                }

                let _utf8_guard = ffi::WindowsUTF8LocaleGuard::new();
                let cstr = libarchive_entry_pathname(entry)?;
                let target_path = CString::new(
                    dest.join(sanitize_destination_path(Path::new(&decode(
                        cstr.to_bytes(),
                    )?))?)
                    .to_str()
                    .unwrap(),
                )
                .unwrap();

                ffi::archive_entry_set_pathname(entry, target_path.as_ptr());

                let link_name = ffi::archive_entry_hardlink(entry);
                if !link_name.is_null() {
                    let target_path = CString::new(
                        dest.join(sanitize_destination_path(Path::new(&decode(
                            CStr::from_ptr(link_name).to_bytes(),
                        )?))?)
                        .to_str()
                        .unwrap(),
                    )
                    .unwrap();

                    ffi::archive_entry_set_hardlink(entry, target_path.as_ptr());
                }

                archive_result_strict(
                    ffi::archive_write_header(archive_writer, entry),
                    archive_writer,
                )?;
                if !libarchive_entry_is_dir(entry) {
                    libarchive_copy_data(archive_reader, archive_writer)?;
                }

                archive_result_strict(
                    ffi::archive_write_finish_entry(archive_writer),
                    archive_writer,
                )?;
            }
        },
    )
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
    uncompress_archive_with_encoding(source, dest, ownership, decode_utf8)
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
/// let decode_utf8 = |bytes: &[u8]| Ok(std::str::from_utf8(bytes)?.to_owned());
///
/// uncompress_archive_file_with_encoding(&mut source, &mut target, "file/path", decode_utf8)?;
/// # Ok(())
/// # }
/// ```
pub fn uncompress_archive_file_with_encoding<R, W>(
    source: R,
    target: W,
    path: &str,
    decode: DecodeCallback,
) -> Result<usize>
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
                    ffi::ARCHIVE_EOF => {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("path {} doesn't exist inside archive", path),
                        )
                        .into())
                    }
                    value => archive_result(value, archive_reader)?,
                }

                let _utf8_guard = ffi::WindowsUTF8LocaleGuard::new();
                let cstr = libarchive_entry_pathname(entry)?;
                let file_name = decode(cstr.to_bytes())?;
                if file_name == path {
                    break;
                }
            }

            if libarchive_entry_is_dir(entry) {
                return Ok(0);
            }
            libarchive_write_data_block(archive_reader, target)
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
    uncompress_archive_file_with_encoding(source, target, path, decode_utf8)
}

fn run_with_archive<F, R, T>(ownership: Ownership, mut reader: R, f: F) -> Result<T>
where
    F: FnOnce(*mut ffi::archive, *mut ffi::archive, *mut ffi::archive_entry) -> Result<T>,
    R: Read + Seek,
{
    let _utf8_guard = ffi::UTF8LocaleGuard::new();
    // libarchive only sniffs the format from offset 0.
    reader.seek(SeekFrom::Start(0))?;
    zip_preflight::reject_unsupported_zip_methods(&mut reader)?;
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
                    std::ptr::addr_of_mut!(pipe) as *mut c_void,
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
                    std::ptr::addr_of_mut!(pipe) as *mut c_void,
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

// This ensures we're not affected by the zip-slip vulnerability. In summary, it
// uses relative destination paths to unpack files in unexpected places. This
// also handles absolute paths, where the leading '/' will be stripped, matching
// behaviour from gnu tar and bsdtar.
//
// More details can be found at: http://snyk.io/research/zip-slip-vulnerability
fn sanitize_destination_path(dest: &Path) -> Result<&Path> {
    let dest = dest.strip_prefix("/").unwrap_or(dest);

    dest.components()
        .find(|c| c == &Component::ParentDir)
        .map_or(Ok(dest), |_| {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "cannot use relative destination directory",
            )
            .into())
        })
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
                value => archive_result(value, archive_reader)?,
            }

            archive_result_strict(
                /* Might depending on the version of libarchive on success
                 * return 0 or the number of bytes written,
                 * see man:archive_write_data(3) */
                match ffi::archive_write_data_block(archive_writer, buffer, size, offset) {
                    x if x >= 0 => 0,
                    x => i32::try_from(x).unwrap(),
                },
                archive_writer,
            )?;
        }
    }
}

fn libarchive_entry_size(entry: *mut ffi::archive_entry) -> u64 {
    // `st_size` is `i32` on Windows (see the `stat` struct above) and `i64`
    // on Unix. Widen through `i64` to keep the cast platform-agnostic.
    let size = unsafe { (*ffi::archive_entry_stat(entry)).st_size } as i64;
    size.max(0) as u64
}

// Raw POSIX mode bits: `libc::S_IFDIR` is not exposed on Windows, where our
// `stat` mirrors libarchive's own layout.
pub(crate) fn libarchive_entry_is_dir(entry: *mut ffi::archive_entry) -> bool {
    const S_IFMT: u32 = 0o170000;
    const S_IFDIR: u32 = 0o040000;
    let mode = unsafe { (*ffi::archive_entry_stat(entry)).st_mode } as u32;
    (mode & S_IFMT) == S_IFDIR
}

fn libarchive_entry_pathname<'a>(entry: *mut ffi::archive_entry) -> Result<&'a CStr> {
    let pathname = unsafe { ffi::archive_entry_pathname(entry) };
    if pathname.is_null() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "archive entry has unreadable filename.".to_string(),
        )
        .into());
    }

    Ok(unsafe { CStr::from_ptr(pathname) })
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
            value => archive_result(value, archive_reader)?,
        }

        if size == 0 {
            continue;
        }

        let content = slice::from_raw_parts(buffer as *const u8, size);
        target.write_all(content)?;
        written += size;
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

    match pipe.reader.read(pipe.buffer) {
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

    match pipe.reader.read(pipe.buffer) {
        Ok(size) => size as ffi::la_ssize_t,
        Err(e) => {
            let description = CString::new(e.to_string()).unwrap();

            ffi::archive_set_error(archive, e.raw_os_error().unwrap_or(0), description.as_ptr());

            -1
        }
    }
}
