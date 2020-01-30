// Copyright (C) 2019 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

/*! The library provide tools for handling compressed and archive files

# Examples
```no_run
use compress_tools;

let dir = tempfile::tempdir().unwrap();
compress_tools::uncompress("tests/fixtures/tree.tar.gz", dir.path(), compress_tools::Kind::TarGZip).unwrap();
```
*/

mod uncompress;

use std::path::Path;

/// Type of compressed file or archive
#[derive(Copy, Clone, Debug)]
pub enum Kind {
    TarGZip,
    TarBZip2,
    TarXz,
    TarLZMA,
    TarLZip,
    Tar,
    Zip,

    GZip,
    BZip2,
    Xz,
    LZip,
    LZMA,
}

/// Uncompress a archive of known [Kind](Kind) pointed by source to target
/// location for file
pub fn uncompress<P1, P2>(source: P1, target: P2, kind: Kind) -> Result<(), failure::Error>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let source = uncompress::Uncompress::new(source.as_ref());
    let target = target.as_ref();
    match kind {
        // Single files
        Kind::BZip2 => source.bz2().file(target),
        Kind::GZip => source.gz().file(target),
        Kind::LZMA => source.lzma().file(target),
        Kind::LZip => source.lzip().file(target),
        Kind::Xz => source.xz().file(target),

        // Archive files
        Kind::Tar => source.tar(target),
        Kind::TarBZip2 => source.bz2().tar(target),
        Kind::TarGZip => source.gz().tar(target),
        Kind::TarLZMA => source.lzma().tar(target),
        Kind::TarLZip => source.lzip().tar(target),
        Kind::TarXz => source.xz().tar(target),
        Kind::Zip => source.unzip(target),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::{
        env, fs,
        io::{Read, Write},
        os::unix::fs::PermissionsExt,
        path::PathBuf,
    };
    use tempfile::TempDir;

    fn create_echo_bin(bin: &Path, output: &Path) -> Result<(), failure::Error> {
        let mut file = std::fs::File::create(bin)?;
        file.write_all(
            format!(
                "#!/bin/sh\necho {} $@ >> {:?};sleep 0.2\n",
                bin.file_name().unwrap().to_str().unwrap(),
                output
            )
            .as_bytes(),
        )?;
        file.set_permissions(fs::Permissions::from_mode(0o777))?;

        Ok(())
    }

    pub fn create_echo_bins(bins: &[&str]) -> Result<(TempDir, PathBuf), failure::Error> {
        let mocks = tempfile::tempdir()?;
        let mocks_dir = mocks.path();
        let calls = mocks_dir.join("calls");

        for bin in bins {
            create_echo_bin(&mocks_dir.join(bin), &calls)?;
        }

        env::set_var(
            "PATH",
            format!(
                "{}{}",
                mocks_dir.display(),
                &env::var("PATH")
                    .map(|s| format!(":{}", s))
                    .unwrap_or_default()
            ),
        );

        Ok((mocks, calls))
    }

    fn assert_calls(p: &Path, expected: &[&str]) {
        let mut content = String::default();
        fs::File::open(p)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        for call in expected {
            assert!(
                content.contains(call),
                format!(
                    "uncompress did not call the expected: '{}'\nFull content:\n{}",
                    call, content
                )
            );
        }
    }

    #[test]
    fn uncompress_tar_gz() {
        let (_dir_handle, calls) = create_echo_bins(&["cat", "tar", "zcat"]).unwrap();
        uncompress("test.tar.gz", &PathBuf::from("target"), Kind::TarGZip)
            .expect("Failed to uncompress file");
        assert_calls(
            &calls,
            &[
                "cat test.tar.gz",
                "zcat",
                "tar --same-owner --preserve-permissions --xattrs -xC target",
            ],
        );
    }

    #[test]
    fn uncompress_tar_bz2() {
        let (_dir_handle, calls) = create_echo_bins(&["cat", "tar", "bzcat"]).unwrap();
        uncompress("test.tar.bz2", &PathBuf::from("target"), Kind::TarBZip2)
            .expect("Failed to uncompress file");
        assert_calls(
            &calls,
            &[
                "cat test.tar.bz2",
                "bzcat",
                "tar --same-owner --preserve-permissions --xattrs -xC target",
            ],
        );
    }

    #[test]
    fn uncompress_tar_xz() {
        let (_dir_handle, calls) = create_echo_bins(&["cat", "tar", "xzcat"]).unwrap();
        uncompress("test.tar.xz", &PathBuf::from("target"), Kind::TarXz)
            .expect("Failed to uncompress file");
        assert_calls(
            &calls,
            &[
                "cat test.tar.xz",
                "xzcat",
                "tar --same-owner --preserve-permissions --xattrs -xC target",
            ],
        );
    }

    #[test]
    fn uncompress_tar_lzma() {
        let (_dir_handle, calls) = create_echo_bins(&["cat", "tar", "lzcat"]).unwrap();
        uncompress("test.tar.lzma", &PathBuf::from("target"), Kind::TarLZMA)
            .expect("Failed to uncompress file");
        assert_calls(
            &calls,
            &[
                "cat test.tar.lzma",
                "lzcat",
                "tar --same-owner --preserve-permissions --xattrs -xC target",
            ],
        );
    }

    #[test]
    fn uncompress_tar_lzip() {
        let (_dir_handle, calls) = create_echo_bins(&["cat", "tar", "lzip"]).unwrap();
        uncompress("test.tar.lzip", &PathBuf::from("target"), Kind::TarLZip)
            .expect("Failed to uncompress file");
        assert_calls(
            &calls,
            &[
                "cat test.tar.lzip",
                "lzip -dc",
                "tar --same-owner --preserve-permissions --xattrs -xC target",
            ],
        );
    }

    #[test]
    fn uncompress_tar() {
        let (_dir_handle, calls) = create_echo_bins(&["cat", "tar"]).unwrap();
        uncompress("test.tar", &PathBuf::from("target"), Kind::Tar)
            .expect("Failed to uncompress file");
        assert_calls(
            &calls,
            &[
                "cat test.tar",
                "tar --same-owner --preserve-permissions --xattrs -xC target",
            ],
        );
    }

    #[test]
    fn uncompress_gz() {
        let (dir_handle, calls) = create_echo_bins(&["cat", "zcat"]).unwrap();
        uncompress("test.gz", &dir_handle.path().join("target"), Kind::GZip)
            .expect("Failed to uncompress file");
        assert_calls(&calls, &["cat test.gz", "zcat"]);
    }

    #[test]
    fn uncompress_bz2() {
        let (dir_handle, calls) = create_echo_bins(&["cat", "bzcat"]).unwrap();
        uncompress("test.bz2", &dir_handle.path().join("target"), Kind::BZip2)
            .expect("Failed to uncompress file");
        assert_calls(&calls, &["cat test.bz2", "bzcat"]);
    }

    #[test]
    fn uncompress_xz() {
        let (dir_handle, calls) = create_echo_bins(&["cat", "xzcat"]).unwrap();
        uncompress("test.xz", &dir_handle.path().join("target"), Kind::Xz)
            .expect("Failed to uncompress file");
        assert_calls(&calls, &["cat test.xz", "xzcat"]);
    }

    #[test]
    fn uncompress_lzma() {
        let (dir_handle, calls) = create_echo_bins(&["cat", "lzcat"]).unwrap();
        uncompress("test.lzma", &dir_handle.path().join("target"), Kind::LZMA)
            .expect("Failed to uncompress file");
        assert_calls(&calls, &["cat test.lzma", "lzcat"]);
    }

    #[test]
    fn uncompress_lzip() {
        let (dir_handle, calls) = create_echo_bins(&["cat", "lzip"]).unwrap();
        uncompress("test.lzip", &dir_handle.path().join("target"), Kind::LZip)
            .expect("Failed to uncompress file");
        assert_calls(&calls, &["cat test.lzip", "lzip -dc"]);
    }

    #[test]
    fn uncompress_zip() {
        let (dir_handle, calls) = create_echo_bins(&["cat", "unzip"]).unwrap();
        uncompress("test.zip", &dir_handle.path().join("target"), Kind::Zip)
            .expect("Failed to uncompress file");
        assert_calls(&calls, &["cat test.zip", "unzip -X -d"]);
    }
}
