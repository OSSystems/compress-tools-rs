// Copyright (C) 2026 O.S. Systems Software LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

//! ZIP central-directory preflight for compression methods libarchive cannot
//! decode (currently Deflate64 / method 9). Non-ZIP inputs, malformed ZIPs,
//! and unseekable readers fall through so libarchive keeps its normal behavior.

use crate::{Error, Result};
use std::io::{self, Read, Seek, SeekFrom};

const LOCAL_FILE_HEADER_SIG: u32 = 0x04034b50;
const EOCD_SIG: u32 = 0x06054b50;
const ZIP64_EOCD_LOCATOR_SIG: u32 = 0x07064b50;
const ZIP64_EOCD_SIG: u32 = 0x06064b50;
const CD_FILE_HEADER_SIG: u32 = 0x02014b50;

const EOCD_SIZE: usize = 22;
const CD_FILE_HEADER_SIZE: usize = 46;
const ZIP64_EOCD_LOCATOR_SIZE: usize = 20;
const ZIP64_EOCD_SIZE: usize = 56;
const MAX_EOCD_COMMENT: u64 = 65_535;

const METHOD_DEFLATE64: u16 = 9;

// Cap CD allocation: the size comes from the archive itself, so a malformed
// or hostile file could otherwise request arbitrary memory.
const MAX_CD_SIZE: u64 = 16 * 1024 * 1024;

fn is_unsupported_method(method: u16) -> bool {
    // Encryption (method 99) is intentionally not flagged: libarchive can
    // decrypt via `archive_read_add_passphrase`, and issue #82 tracks
    // exposing that from this crate.
    method == METHOD_DEFLATE64
}

pub(crate) fn reject_unsupported_zip_methods<R: Read + Seek>(reader: &mut R) -> Result<()> {
    let offending = scan(reader).unwrap_or_default();
    let _ = reader.seek(SeekFrom::Start(0));
    if offending.is_empty() {
        Ok(())
    } else {
        Err(Error::UnsupportedZipCompression(offending))
    }
}

fn scan<R: Read + Seek>(reader: &mut R) -> io::Result<Vec<(String, u16)>> {
    reader.seek(SeekFrom::Start(0))?;
    let mut magic = [0u8; 4];
    reader.read_exact(&mut magic)?;
    let sig = u32::from_le_bytes(magic);
    if sig != LOCAL_FILE_HEADER_SIG && sig != EOCD_SIG {
        return Ok(Vec::new());
    }

    let total_len = reader.seek(SeekFrom::End(0))?;
    let Some(cd) = find_cd(reader, total_len)? else {
        return Ok(Vec::new());
    };
    walk_central_directory(reader, cd)
}

struct CdLocation {
    offset: u64,
    size: u64,
    entries: u64,
}

fn find_cd<R: Read + Seek>(reader: &mut R, total_len: u64) -> io::Result<Option<CdLocation>> {
    if total_len < EOCD_SIZE as u64 {
        return Ok(None);
    }
    let window = (MAX_EOCD_COMMENT + EOCD_SIZE as u64).min(total_len);
    let window_start = total_len - window;
    reader.seek(SeekFrom::Start(window_start))?;
    let mut buf = vec![0u8; window as usize];
    reader.read_exact(&mut buf)?;

    let sig = EOCD_SIG.to_le_bytes();
    let Some(eocd_rel) = buf.windows(EOCD_SIZE).rposition(|w| w[..4] == sig) else {
        return Ok(None);
    };
    let eocd = &buf[eocd_rel..];
    let entries_u16 = read_u16(eocd, 10);
    let size_u32 = read_u32(eocd, 12);
    let offset_u32 = read_u32(eocd, 16);

    if entries_u16 != 0xFFFF && size_u32 != 0xFFFFFFFF && offset_u32 != 0xFFFFFFFF {
        return Ok(Some(CdLocation {
            offset: offset_u32 as u64,
            size: size_u32 as u64,
            entries: entries_u16 as u64,
        }));
    }

    // Saturated field(s) — consult the Zip64 locator (sits immediately before
    // EOCD).
    let eocd_abs = window_start + eocd_rel as u64;
    if eocd_abs < ZIP64_EOCD_LOCATOR_SIZE as u64 {
        return Ok(None);
    }
    reader.seek(SeekFrom::Start(eocd_abs - ZIP64_EOCD_LOCATOR_SIZE as u64))?;
    let mut loc = [0u8; ZIP64_EOCD_LOCATOR_SIZE];
    reader.read_exact(&mut loc)?;
    if read_u32(&loc, 0) != ZIP64_EOCD_LOCATOR_SIG {
        return Ok(None);
    }
    reader.seek(SeekFrom::Start(read_u64(&loc, 8)))?;
    let mut zeocd = [0u8; ZIP64_EOCD_SIZE];
    reader.read_exact(&mut zeocd)?;
    if read_u32(&zeocd, 0) != ZIP64_EOCD_SIG {
        return Ok(None);
    }
    Ok(Some(CdLocation {
        entries: read_u64(&zeocd, 24),
        size: read_u64(&zeocd, 40),
        offset: read_u64(&zeocd, 48),
    }))
}

fn walk_central_directory<R: Read + Seek>(
    reader: &mut R,
    cd: CdLocation,
) -> io::Result<Vec<(String, u16)>> {
    if cd.size > MAX_CD_SIZE {
        return Ok(Vec::new());
    }
    reader.seek(SeekFrom::Start(cd.offset))?;
    let mut buf = vec![0u8; cd.size as usize];
    reader.read_exact(&mut buf)?;

    let mut offenders = Vec::new();
    let mut pos = 0usize;
    for _ in 0..cd.entries {
        if pos + CD_FILE_HEADER_SIZE > buf.len() || read_u32(&buf, pos) != CD_FILE_HEADER_SIG {
            break;
        }
        let method = read_u16(&buf, pos + 10);
        let name_len = read_u16(&buf, pos + 28) as usize;
        let extra_len = read_u16(&buf, pos + 30) as usize;
        let comment_len = read_u16(&buf, pos + 32) as usize;
        let name_start = pos + CD_FILE_HEADER_SIZE;
        let name_end = name_start + name_len;
        if name_end > buf.len() {
            break;
        }
        if is_unsupported_method(method) {
            offenders.push((
                String::from_utf8_lossy(&buf[name_start..name_end]).into_owned(),
                method,
            ));
        }
        pos = name_end + extra_len + comment_len;
    }
    Ok(offenders)
}

fn read_u16(buf: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes(buf[offset..offset + 2].try_into().unwrap())
}

fn read_u32(buf: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(buf[offset..offset + 4].try_into().unwrap())
}

fn read_u64(buf: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(buf[offset..offset + 8].try_into().unwrap())
}
