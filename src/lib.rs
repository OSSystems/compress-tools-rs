// Copyright (C) 2019 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

/*! The library provide tools for handling compressed and archive files

# Examples
```
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

    GZip,
    BZip2,
    Xz,
    LZip,
    LZMA,
}

/// Uncompress a archive of known [Kind](Kind) pointed by source to target location for file
pub fn uncompress<P1, P2>(source: P1, target: P2, kind: Kind) -> Result<(), failure::Error>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
{
    let source = uncompress::Uncompress::new(source.as_ref());
    let target = target.as_ref();
    match kind {
        Kind::TarGZip => source.gz().tar(target),
        Kind::TarBZip2 => source.bz2().tar(target),
        Kind::TarXz => source.xz().tar(target),
        Kind::TarLZMA => source.lzma().tar(target),
        Kind::TarLZip => source.lz().tar(target),
        Kind::Tar => source.tar(target),

        Kind::GZip => source.gz().file(target),
        Kind::BZip2 => source.bz2().file(target),
        Kind::Xz => source.xz().file(target),
        Kind::LZMA => source.lzma().file(target),
        Kind::LZip => source.lz().file(target),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::os::unix::fs::MetadataExt;
    use tempfile;

    fn assert_tree(p: &Path) {
        let leaf1 = p
            .join("tree/branch1/leaf")
            .metadata()
            .expect("tree/branch1/leaf not found in extracted directory structure");
        assert_eq!(leaf1.mode() % 0o1000, 0o664);

        let leaf2 = p
            .join("tree/branch2/leaf")
            .metadata()
            .expect("tree/branch1/leaf not found in extracted directory structure");
        assert_eq!(leaf2.mode() % 0o1000, 0o664);
    }

    fn assert_file(p: &Path) {
        let tree = p
            .metadata()
            .expect("tree.tar not found in extracted directory");
        assert_eq!(tree.mode() % 0o1000, 0o664);
    }

    #[test]
    fn uncompress_tar_gz() {
        let dir = tempfile::tempdir().expect("Unable to create temp dir");
        uncompress("tests/fixtures/tree.tar.gz", dir.path(), Kind::TarGZip)
            .expect("Failed to uncompress file");
        assert_tree(dir.path())
    }

    #[test]
    fn uncompress_tar_bz2() {
        let dir = tempfile::tempdir().expect("Unable to create temp dir");
        uncompress("tests/fixtures/tree.tar.bz2", dir.path(), Kind::TarBZip2)
            .expect("Failed to uncompress file");
        assert_tree(dir.path())
    }

    #[test]
    fn uncompress_tar_xz() {
        let dir = tempfile::tempdir().expect("Unable to create temp dir");
        uncompress("tests/fixtures/tree.tar.xz", dir.path(), Kind::TarXz)
            .expect("Failed to uncompress file");
        assert_tree(dir.path())
    }

    #[test]
    fn uncompress_tar_lzma() {
        let dir = tempfile::tempdir().expect("Unable to create temp dir");
        uncompress("tests/fixtures/tree.tar.lzma", dir.path(), Kind::TarLZMA)
            .expect("Failed to uncompress file");
        assert_tree(dir.path())
    }

    #[test]
    fn uncompress_tar_lz() {
        let dir = tempfile::tempdir().expect("Unable to create temp dir");
        uncompress("tests/fixtures/tree.tar.lz", dir.path(), Kind::TarLZip)
            .expect("Failed to uncompress file");
        assert_tree(dir.path())
    }

    #[test]
    fn uncompress_tar() {
        let dir = tempfile::tempdir().expect("Unable to create temp dir");
        uncompress("tests/fixtures/tree.tar", dir.path(), Kind::Tar)
            .expect("Failed to uncompress file");
        assert_tree(dir.path())
    }

    #[test]
    fn uncompress_gz() {
        let dir = tempfile::tempdir().expect("Unable to create temp dir");
        uncompress(
            "tests/fixtures/tree.tar.gz",
            &dir.path().join("tree.tar"),
            Kind::GZip,
        )
        .expect("Failed to uncompress file");
        assert_file(&dir.path().join("tree.tar"))
    }

    #[test]
    fn uncompress_bz2() {
        let dir = tempfile::tempdir().expect("Unable to create temp dir");
        uncompress(
            "tests/fixtures/tree.tar.bz2",
            &dir.path().join("tree.tar"),
            Kind::BZip2,
        )
        .expect("Failed to uncompress file");
        assert_file(&dir.path().join("tree.tar"))
    }

    #[test]
    fn uncompress_xz() {
        let dir = tempfile::tempdir().expect("Unable to create temp dir");
        uncompress(
            "tests/fixtures/tree.tar.xz",
            &dir.path().join("tree.tar"),
            Kind::Xz,
        )
        .expect("Failed to uncompress file");
        assert_file(&dir.path().join("tree.tar"))
    }

    #[test]
    fn uncompress_lzma() {
        let dir = tempfile::tempdir().expect("Unable to create temp dir");
        uncompress(
            "tests/fixtures/tree.tar.lzma",
            &dir.path().join("tree.tar"),
            Kind::LZMA,
        )
        .expect("Failed to uncompress file");
        assert_file(&dir.path().join("tree.tar"))
    }

    #[test]
    fn uncompress_lz() {
        let dir = tempfile::tempdir().expect("Unable to create temp dir");
        uncompress(
            "tests/fixtures/tree.tar.lz",
            &dir.path().join("tree.tar"),
            Kind::LZip,
        )
        .expect("Failed to uncompress file");
        assert_file(&dir.path().join("tree.tar"))
    }
}
