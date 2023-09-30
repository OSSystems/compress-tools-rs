use std::{
    ffi::{CStr, CString},
    io::{Read, Seek, SeekFrom, Write},
    slice,
};

use libc::{c_int, c_void};

use crate::{
    error::archive_result, ffi, ffi::UTF8LocaleGuard, DecodeCallback, Error, Result,
    READER_BUFFER_SIZE,
};

struct HeapReadSeekerPipe<R: Read + Seek> {
    reader: R,
    buffer: [u8; READER_BUFFER_SIZE],
}

/// The contents of an archive, yielded in order from the beginning to the end
/// of the archive.
///
/// Each entry, file or directory, will have a
/// [`ArchiveContents::StartOfEntry`], zero or more
/// [`ArchiveContents::DataChunk`], and then a corresponding
/// [`ArchiveContents::EndOfEntry`] to mark that the entry has been read to
/// completion.
pub enum ArchiveContents {
    /// Marks the start of an entry, either a file or a directory.
    StartOfEntry(String, libc::stat),
    /// A chunk of uncompressed data from the entry. Entries may have zero or
    /// more chunks.
    DataChunk(Vec<u8>),
    /// Marks the end of the entry that was started by the previous
    /// StartOfEntry.
    EndOfEntry,
    Err(Error),
}

/// Filter for an archive iterator to skip decompression of unwanted
/// entries.
///
/// Gets called on an encounter of a new archive entry with the filename and
/// file status information of that entry.
/// The entry is processed on a return value of `true` and ignored on `false`.
pub type EntryFilterCallbackFn = dyn Fn(&str, &libc::stat) -> bool;

/// An iterator over the contents of an archive.
#[allow(clippy::module_name_repetitions)]
pub struct ArchiveIterator<R: Read + Seek> {
    archive_entry: *mut ffi::archive_entry,
    archive_reader: *mut ffi::archive,

    decode: DecodeCallback,
    in_file: bool,
    closed: bool,
    error: bool,
    filter: Option<Box<EntryFilterCallbackFn>>,

    _pipe: Box<HeapReadSeekerPipe<R>>,
    _utf8_guard: UTF8LocaleGuard,
}

impl<R: Read + Seek> Iterator for ArchiveIterator<R> {
    type Item = ArchiveContents;

    fn next(&mut self) -> Option<Self::Item> {
        debug_assert!(!self.closed);

        if self.error {
            return None;
        }

        loop {
            let next = if self.in_file {
                unsafe { self.next_data_chunk() }
            } else {
                unsafe { self.next_header() }
            };

            match &next {
                ArchiveContents::StartOfEntry(name, stat) => {
                    debug_assert!(!self.in_file);

                    if let Some(filter) = &self.filter {
                        if !filter(name, stat) {
                            continue;
                        }
                    }

                    self.in_file = true;
                    break Some(next);
                }
                ArchiveContents::DataChunk(_) => {
                    debug_assert!(self.in_file);
                    break Some(next);
                }
                ArchiveContents::EndOfEntry if self.in_file => {
                    self.in_file = false;
                    break Some(next);
                }
                ArchiveContents::EndOfEntry => break None,
                ArchiveContents::Err(_) => {
                    self.error = true;
                    break Some(next);
                }
            }
        }
    }
}

impl<R: Read + Seek> Drop for ArchiveIterator<R> {
    fn drop(&mut self) {
        drop(self.free());
    }
}

impl<R: Read + Seek> ArchiveIterator<R> {
    fn new(
        source: R,
        decode: DecodeCallback,
        filter: Option<Box<EntryFilterCallbackFn>>,
    ) -> Result<ArchiveIterator<R>>
    where
        R: Read + Seek,
    {
        let utf8_guard = ffi::UTF8LocaleGuard::new();
        let reader = source;
        let buffer = [0; READER_BUFFER_SIZE];
        let mut pipe = Box::new(HeapReadSeekerPipe { reader, buffer });

        unsafe {
            let archive_entry: *mut ffi::archive_entry = std::ptr::null_mut();
            let archive_reader = ffi::archive_read_new();

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
                    ffi::archive_read_set_seek_callback(
                        archive_reader,
                        Some(libarchive_heap_seek_callback::<R>),
                    ),
                    archive_reader,
                )?;

                if archive_reader.is_null() {
                    return Err(Error::NullArchive);
                }

                archive_result(
                    ffi::archive_read_support_format_all(archive_reader),
                    archive_reader,
                )?;

                archive_result(
                    ffi::archive_read_open(
                        archive_reader,
                        std::ptr::addr_of_mut!(*pipe) as *mut c_void,
                        None,
                        Some(libarchive_heap_seekableread_callback::<R>),
                        None,
                    ),
                    archive_reader,
                )?;

                Ok(())
            })();

            let iter = ArchiveIterator {
                archive_entry,
                archive_reader,

                decode,
                in_file: false,
                closed: false,
                error: false,
                filter,

                _pipe: pipe,
                _utf8_guard: utf8_guard,
            };

            res?;
            Ok(iter)
        }
    }

    /// Iterate over the contents of an archive, streaming the contents of each
    /// entry in small chunks.
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use compress_tools::*;
    /// use std::fs::File;
    ///
    /// let file = File::open("tree.tar")?;
    ///
    /// let mut name = String::default();
    /// let mut size = 0;
    /// let decode_utf8 = |bytes: &[u8]| Ok(std::str::from_utf8(bytes)?.to_owned());
    /// let mut iter = ArchiveIterator::from_read_with_encoding(file, decode_utf8)?;
    ///
    /// for content in &mut iter {
    ///     match content {
    ///         ArchiveContents::StartOfEntry(s, _) => name = s,
    ///         ArchiveContents::DataChunk(v) => size += v.len(),
    ///         ArchiveContents::EndOfEntry => {
    ///             println!("Entry {} was {} bytes", name, size);
    ///             size = 0;
    ///         }
    ///         ArchiveContents::Err(e) => {
    ///             Err(e)?;
    ///         }
    ///     }
    /// }
    ///
    /// iter.close()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_read_with_encoding(source: R, decode: DecodeCallback) -> Result<ArchiveIterator<R>>
    where
        R: Read + Seek,
    {
        Self::new(source, decode, None)
    }

    /// Iterate over the contents of an archive, streaming the contents of each
    /// entry in small chunks.
    ///
    /// ```no_run
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use compress_tools::*;
    /// use std::fs::File;
    ///
    /// let file = File::open("tree.tar")?;
    ///
    /// let mut name = String::default();
    /// let mut size = 0;
    /// let mut iter = ArchiveIterator::from_read(file)?;
    ///
    /// for content in &mut iter {
    ///     match content {
    ///         ArchiveContents::StartOfEntry(s, _) => name = s,
    ///         ArchiveContents::DataChunk(v) => size += v.len(),
    ///         ArchiveContents::EndOfEntry => {
    ///             println!("Entry {} was {} bytes", name, size);
    ///             size = 0;
    ///         }
    ///         ArchiveContents::Err(e) => {
    ///             Err(e)?;
    ///         }
    ///     }
    /// }
    ///
    /// iter.close()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_read(source: R) -> Result<ArchiveIterator<R>>
    where
        R: Read + Seek,
    {
        Self::new(source, crate::decode_utf8, None)
    }

    /// Close the iterator, freeing up the associated resources.
    ///
    /// Resources will be freed on drop if this is not called, but any errors
    /// during freeing on drop will be lost.
    pub fn close(mut self) -> Result<()> {
        self.free()
    }

    fn free(&mut self) -> Result<()> {
        if self.closed {
            return Ok(());
        }

        self.closed = true;
        unsafe {
            archive_result(
                ffi::archive_read_close(self.archive_reader),
                self.archive_reader,
            )?;
            archive_result(
                ffi::archive_read_free(self.archive_reader),
                self.archive_reader,
            )?;
        }
        Ok(())
    }

    unsafe fn next_header(&mut self) -> ArchiveContents {
        match ffi::archive_read_next_header(self.archive_reader, &mut self.archive_entry) {
            ffi::ARCHIVE_EOF => ArchiveContents::EndOfEntry,
            ffi::ARCHIVE_OK | ffi::ARCHIVE_WARN => {
                let _utf8_guard = ffi::WindowsUTF8LocaleGuard::new();
                let cstr = CStr::from_ptr(ffi::archive_entry_pathname(self.archive_entry));
                let file_name = match (self.decode)(cstr.to_bytes()) {
                    Ok(f) => f,
                    Err(e) => return ArchiveContents::Err(e),
                };
                let stat = *ffi::archive_entry_stat(self.archive_entry);
                ArchiveContents::StartOfEntry(file_name, stat)
            }
            _ => ArchiveContents::Err(Error::from(self.archive_reader)),
        }
    }

    unsafe fn next_data_chunk(&mut self) -> ArchiveContents {
        let mut buffer = std::ptr::null();
        let mut offset = 0;
        let mut size = 0;
        let mut target = Vec::with_capacity(READER_BUFFER_SIZE);

        match ffi::archive_read_data_block(self.archive_reader, &mut buffer, &mut size, &mut offset)
        {
            ffi::ARCHIVE_EOF => ArchiveContents::EndOfEntry,
            ffi::ARCHIVE_OK | ffi::ARCHIVE_WARN => {
                let content = slice::from_raw_parts(buffer as *const u8, size);
                let write = target.write_all(content);
                if let Err(e) = write {
                    ArchiveContents::Err(e.into())
                } else {
                    ArchiveContents::DataChunk(target)
                }
            }
            _ => ArchiveContents::Err(Error::from(self.archive_reader)),
        }
    }
}

unsafe extern "C" fn libarchive_heap_seek_callback<R: Read + Seek>(
    _: *mut ffi::archive,
    client_data: *mut c_void,
    offset: ffi::la_int64_t,
    whence: c_int,
) -> i64 {
    let pipe = (client_data as *mut HeapReadSeekerPipe<R>)
        .as_mut()
        .unwrap();
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

unsafe extern "C" fn libarchive_heap_seekableread_callback<R: Read + Seek>(
    archive: *mut ffi::archive,
    client_data: *mut c_void,
    buffer: *mut *const c_void,
) -> ffi::la_ssize_t {
    let pipe = (client_data as *mut HeapReadSeekerPipe<R>)
        .as_mut()
        .unwrap();

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

#[must_use]
pub struct ArchiveIteratorBuilder<R>
where
    R: Read + Seek,
{
    source: R,
    decoder: DecodeCallback,
    filter: Option<Box<EntryFilterCallbackFn>>,
}

/// A builder to generate an archive iterator over the contents of an
/// archive, streaming the contents of each entry in small chunks.
/// The default configuration is identical to `ArchiveIterator::from_read`.
///
/// # Example
///
/// ```no_run
/// use compress_tools::{ArchiveContents, ArchiveIteratorBuilder};
/// use std::path::Path;
/// use std::ffi::OsStr;
///
/// let source = std::fs::File::open("tests/fixtures/tree.tar").expect("Failed to open file");
/// let decode_utf8 = |bytes: &[u8]| Ok(std::str::from_utf8(bytes)?.to_owned());
///
/// for content in ArchiveIteratorBuilder::new(source)
///     .decoder(decode_utf8)
///     .filter(|name, stat| Path::new(name).file_name() == Some(OsStr::new("foo")) || stat.st_size == 42)
///     .build()
///     .expect("Failed to initialize archive")
///     {
///         if let ArchiveContents::StartOfEntry(name, _stat) = content {
///             println!("{name}");
///         }
///     }
/// ```
impl<R> ArchiveIteratorBuilder<R>
where
    R: Read + Seek,
{
    /// Create a new builder for an archive iterator. Default configuration is
    /// identical to `ArchiveIterator::from_read`.
    pub fn new(source: R) -> ArchiveIteratorBuilder<R> {
        ArchiveIteratorBuilder {
            source,
            decoder: crate::decode_utf8,
            filter: None,
        }
    }

    /// Use a custom decoder to decode filenames of archive entries.
    /// By default an UTF-8 decoder (`decode_utf8`) is used.
    pub fn decoder(mut self, decoder: DecodeCallback) -> ArchiveIteratorBuilder<R> {
        self.decoder = decoder;
        self
    }

    /// Use a filter to skip unwanted entries and their decompression.
    /// By default all entries are iterated.
    pub fn filter<F>(mut self, filter: F) -> ArchiveIteratorBuilder<R>
    where
        F: Fn(&str, &libc::stat) -> bool + 'static,
    {
        self.filter = Some(Box::new(filter));
        self
    }

    /// Finish the builder and generate the configured `ArchiveIterator`.
    pub fn build(self) -> Result<ArchiveIterator<R>> {
        ArchiveIterator::new(self.source, self.decoder, self.filter)
    }
}
