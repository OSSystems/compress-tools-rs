// Copyright (C) 2019, 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::ffi;
use derive_more::{Display, Error, From};
use std::ffi::CStr;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Display, From, Error, Debug)]
pub enum Error {
    #[display(fmt = "Extraction error: '{}'", _0)]
    Extract(#[error(not(source))] String),

    #[display(fmt = "Io error: '{}'", _0)]
    Io(std::io::Error),

    #[display(fmt = "Utf error: '{}'", _0)]
    Utf(std::str::Utf8Error),

    #[display(fmt = "Error to create the archive struct, is null")]
    NullArchive,

    #[display(fmt = "The entry is null, failed to set the pathname")]
    NullEntry,

    #[display(fmt = "File not found")]
    FileNotFound,

    #[display(fmt = "Disk access not supported in async environment")]
    AsyncDiskAccessNotSupported,
}

pub(crate) fn archive_result(value: i32, archive: *mut ffi::archive) -> Result<()> {
    if value != ffi::ARCHIVE_OK {
        return Err(Error::from(archive));
    }

    Ok(())
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl From<*mut ffi::archive> for Error {
    fn from(input: *mut ffi::archive) -> Self {
        unsafe {
            let input = ffi::archive_error_string(input);
            Error::Extract(CStr::from_ptr(input).to_string_lossy().to_string())
        }
    }
}
