// Copyright (C) 2019, 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! The `compress-tools` crate aims to provide a convenient and easy to use set
//! of methods which builds on top of `libarchive` exposing a small set of it’s
//! functionalities.
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
//! uncompress_archive(&mut source, &dest)?;
//! # Ok(())
//! # }
//! ```

mod error;
mod ffi;

use error::archive_result;
pub use error::{Error, Result};
use std::{
    ffi::{CStr, CString},
    io::{Read, Write},
    os::raw::c_void,
    path::Path,
    slice,
};

const READER_BUFFER_SIZE: usize = 1024;

struct Pipe<'a> {
    reader: &'a mut dyn Read,
    buffer: &'a mut [u8],
}

enum Mode {
    AllFormat,
    RawFormat,
    WriteDisk,
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
/// uncompress_file(&mut source, &mut target)?;
/// # Ok(())
/// # }
/// ```
pub fn uncompress_file<R, W>(source: &mut R, target: &mut W) -> Result<()>
where
    R: Read + 'static,
    W: Write,
{
    run_with_archive(
        Mode::RawFormat,
        source,
        |archive_reader, _, mut entry| unsafe {
            archive_result(
                ffi::archive_read_next_header(archive_reader, &mut entry),
                archive_reader,
            )?;
            libarchive_write_data_block(archive_reader, target)
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
/// uncompress_archive(&mut source, &dest)?;
/// # Ok(())
/// # }
/// ```
pub fn uncompress_archive<R>(source: &mut R, dest: &Path) -> Result<()>
where
    R: Read + 'static,
{
    run_with_archive(
        Mode::WriteDisk,
        source,
        |archive_reader, archive_writer, mut entry| unsafe {
            loop {
                match ffi::archive_read_next_header(archive_reader, &mut entry) {
                    ffi::ARCHIVE_EOF => return Ok(()),
                    ffi::ARCHIVE_OK => {
                        let target_path =
                            dest.join(CStr::from_ptr(ffi::archive_entry_pathname(entry)).to_str()?);
                        ffi::archive_entry_set_pathname(
                            entry,
                            CString::new(target_path.to_str().unwrap())
                                .unwrap()
                                .into_raw(),
                        );

                        let link_name = ffi::archive_entry_hardlink(entry);
                        if !link_name.is_null() {
                            let target_path = dest.join(CStr::from_ptr(link_name).to_str()?);
                            ffi::archive_entry_set_hardlink(
                                entry,
                                CString::new(target_path.to_str().unwrap())
                                    .unwrap()
                                    .into_raw(),
                            );
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
/// reader, the `target` as a writer and the `path` is the full path for the
/// file to be extracted.
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
pub fn uncompress_archive_file<R, W>(source: &mut R, target: &mut W, path: &str) -> Result<()>
where
    R: Read + 'static,
    W: Write,
{
    run_with_archive(
        Mode::AllFormat,
        source,
        |archive_reader, _, mut entry| unsafe {
            loop {
                match ffi::archive_read_next_header(archive_reader, &mut entry) {
                    ffi::ARCHIVE_OK => {
                        let file_name =
                            CStr::from_ptr(ffi::archive_entry_pathname(entry)).to_str()?;
                        if file_name == path {
                            break;
                        }
                    }
                    ffi::ARCHIVE_EOF => return Err(Error::FileNotFound),
                    _ => return Err(Error::from(archive_reader)),
                }
            }
            libarchive_write_data_block(archive_reader, target)
        },
    )
}

fn run_with_archive<F, R>(mode: Mode, reader: &mut R, f: F) -> Result<()>
where
    F: FnOnce(*mut ffi::archive, *mut ffi::archive, *mut ffi::archive_entry) -> Result<()>,
    R: Read + 'static,
{
    let archive_reader: *mut ffi::archive;
    let archive_writer: *mut ffi::archive;
    let archive_entry: *mut ffi::archive_entry = std::ptr::null_mut();

    unsafe {
        archive_reader = ffi::archive_read_new();
        archive_writer = ffi::archive_write_disk_new();
        archive_result(
            ffi::archive_read_support_filter_all(archive_reader),
            archive_reader,
        )?;
        match mode {
            Mode::RawFormat => archive_result(
                ffi::archive_read_support_format_raw(archive_reader),
                archive_reader,
            )?,
            Mode::AllFormat => archive_result(
                ffi::archive_read_support_format_all(archive_reader),
                archive_reader,
            )?,
            Mode::WriteDisk => {
                let writer_flags = (ffi::ARCHIVE_EXTRACT_TIME
                    | ffi::ARCHIVE_EXTRACT_PERM
                    | ffi::ARCHIVE_EXTRACT_ACL
                    | ffi::ARCHIVE_EXTRACT_FFLAGS
                    | ffi::ARCHIVE_EXTRACT_OWNER
                    | ffi::ARCHIVE_EXTRACT_XATTR) as i32;

                archive_result(
                    ffi::archive_write_disk_set_options(archive_writer, writer_flags),
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
                archive_result(
                    ffi::archive_read_support_format_raw(archive_reader),
                    archive_reader,
                )?;
            }
        }

        if archive_reader.is_null() {
            return Err(Error::ArchiveNull);
        }

        if archive_writer.is_null() {
            return Err(Error::ArchiveNull);
        }

        let mut pipe = Pipe {
            reader: &mut Box::new(reader),
            buffer: &mut [0; READER_BUFFER_SIZE],
        };

        archive_result(
            ffi::archive_read_open(
                archive_reader,
                (&mut pipe as *mut Pipe) as *mut c_void,
                None,
                Some(libarchive_read_callback),
                None,
            ),
            archive_reader,
        )?;

        f(archive_reader, archive_writer, archive_entry)?;

        archive_result(ffi::archive_read_close(archive_reader), archive_reader)?;
        archive_result(ffi::archive_read_free(archive_reader), archive_reader)?;
        archive_result(ffi::archive_write_close(archive_writer), archive_writer)?;
        archive_result(ffi::archive_write_free(archive_writer), archive_writer)?;
        ffi::archive_entry_free(archive_entry);
    }
    Ok(())
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
    target: &mut W,
) -> Result<()>
where
    W: Write,
{
    let mut buffer = std::ptr::null();
    let mut offset = 0;
    let mut size = 0;

    loop {
        match ffi::archive_read_data_block(archive_reader, &mut buffer, &mut size, &mut offset) {
            ffi::ARCHIVE_EOF => return Ok(()),
            ffi::ARCHIVE_OK => {
                let content = slice::from_raw_parts(buffer as *const u8, size);
                target.write_all(content)?;
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
    let pipe = (client_data as *mut Pipe).as_mut().unwrap();

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
