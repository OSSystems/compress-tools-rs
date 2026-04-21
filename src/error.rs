// Copyright (C) 2019-2021 O.S. Systems Software LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::ffi;
use derive_more::{Display, Error, From};
use std::{borrow::Cow, ffi::CStr, io};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Display, From, Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[display(
        "Extraction error:{}{} '{}'",
        match code {
            Some(_) => " ",
            None => ""
        },
        if let Some(e) = code {
            e as &dyn std::fmt::Display
        } else {
            &"" as &_
        },
        details
    )]
    Extraction {
        /// The code stemming from `archive_errno`, unless it is not a valid
        /// value for `errno(3)`, like `ARCHIVE_ERRNO_MISC`
        #[error(source)]
        code: Option<io::Error>,
        /// The string returned by `archive_error_string`
        details: String,
    },

    Io(io::Error),

    Utf(std::str::Utf8Error),

    #[display("Encoding error: '{}'", _0)]
    Encoding(#[error(not(source))] Cow<'static, str>),

    #[cfg(feature = "tokio_support")]
    JoinError(tokio::task::JoinError),

    #[display("Error to create the archive struct, is null")]
    NullArchive,

    #[display(
        "Unsupported ZIP compression method in {} {}: {:?}",
        _0.len(),
        if _0.len() == 1 { "entry" } else { "entries" },
        _0
    )]
    UnsupportedZipCompression(#[error(not(source))] Vec<(String, u16)>),

    #[display("Unknown error")]
    Unknown,
}

pub(crate) fn archive_result(value: i32, archive: *mut ffi::archive) -> Result<()> {
    match value {
        ffi::ARCHIVE_OK | ffi::ARCHIVE_WARN => Ok(()),
        _ => Err(Error::from(archive)),
    }
}

/// Like [`archive_result`], but treats `ARCHIVE_WARN` as an error.
///
/// Use this on call sites where a warning indicates user-visible data loss —
/// for example, `archive_write_header` and `archive_write_data_block`, which
/// can return `ARCHIVE_WARN` when the target filesystem returns `ENOSPC`. See
/// https://github.com/OSSystems/compress-tools-rs/issues/142.
pub(crate) fn archive_result_strict(value: i32, archive: *mut ffi::archive) -> Result<()> {
    match value {
        ffi::ARCHIVE_OK => Ok(()),
        _ => Err(Error::from(archive)),
    }
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
impl From<*mut ffi::archive> for Error {
    fn from(input: *mut ffi::archive) -> Self {
        let (details, code) = unsafe {
            let error_string = ffi::archive_error_string(input);
            let details = if !error_string.is_null() {
                Some(CStr::from_ptr(error_string).to_string_lossy().into_owned())
            } else {
                None
            };

            let errno = ffi::archive_errno(input);
            let code = if errno > 0 {
                Some(io::Error::from_raw_os_error(errno))
            } else {
                // 0 (unexpected) or ARCHIVE_ERRNO_MISC which is not a valid value of errno(3)
                None
            };
            (details, code)
        };
        match (details, code) {
            (Some(details), code) => Error::Extraction { code, details },
            (None, Some(code)) => Error::Io(code),
            (None, None) => Error::Unknown,
        }
    }
}
