// Copyright (C) 2019 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::ffi;
use std::ffi::CStr;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Extraction error: '{0}'")]
    ExtractionError(String),

    #[error("Io error: '{0}'")]
    Io(#[from] std::io::Error),

    #[error("Utf error: '{0}'")]
    Utf(#[from] std::str::Utf8Error),

    #[error("Error to create the archive struct, is null")]
    ArchiveNull,

    #[error("The entry is null, failed to set the pathname")]
    EntryNull,

    #[error("File not found")]
    FileNotFound,

    #[error("The end of file")]
    EndOfFile,
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
            Error::ExtractionError(CStr::from_ptr(input).to_string_lossy().to_string())
        }
    }
}
