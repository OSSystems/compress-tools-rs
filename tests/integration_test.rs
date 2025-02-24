// Copyright (C) 2019-2021 O.S. Systems Software LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use compress_tools::*;
use libc::S_IFREG;
use std::{
    ffi::OsStr,
    io::{Cursor, ErrorKind, Read},
    path::Path,
};

#[test]
fn get_compressed_file_content() {
    let mut source = std::fs::File::open("tests/fixtures/file.txt.gz").unwrap();
    let mut target = Vec::default();

    let written = uncompress_data(&mut source, &mut target).expect("Failed to uncompress the file");
    assert_eq!(
        String::from_utf8_lossy(&target),
        "some_file_content\n",
        "Uncompressed file did not match",
    );
    assert_eq!(written, 18, "Uncompressed bytes count did not match");
}

#[async_std::test]
#[cfg(feature = "futures_support")]
async fn get_compressed_file_content_futures() {
    let mut source = async_std::fs::File::open("tests/fixtures/file.txt.gz")
        .await
        .unwrap();
    let mut target = Vec::default();

    let written = futures_support::uncompress_data(&mut source, &mut target)
        .await
        .expect("Failed to uncompress the file");
    assert_eq!(
        String::from_utf8_lossy(&target),
        "some_file_content\n",
        "Uncompressed file did not match",
    );
    assert_eq!(written, 18, "Uncompressed bytes count did not match");
}

#[tokio::test]
#[cfg(feature = "tokio_support")]
async fn get_compressed_file_content_tokio() {
    let mut source = tokio::fs::File::open("tests/fixtures/file.txt.gz")
        .await
        .unwrap();
    let mut target = Vec::default();

    let written = tokio_support::uncompress_data(&mut source, &mut target)
        .await
        .expect("Failed to uncompress the file");
    assert_eq!(
        String::from_utf8_lossy(&target),
        "some_file_content\n",
        "Uncompressed file did not match",
    );
    assert_eq!(written, 18, "Uncompressed bytes count did not match");
}

#[test]
fn get_a_file_from_tar() {
    let mut source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();
    let mut target = Vec::default();

    let written = uncompress_archive_file(&mut source, &mut target, "tree/branch2/leaf")
        .expect("Failed to get the file");
    assert_eq!(
        String::from_utf8_lossy(&target),
        "Goodbye World\n",
        "Uncompressed file did not match",
    );
    assert_eq!(written, 14, "Uncompressed bytes count did not match");
}

#[test]
fn get_a_file_from_7z() {
    let mut source = std::fs::File::open("tests/fixtures/tree.7z").unwrap();
    let mut target = Vec::default();

    let written = uncompress_archive_file(&mut source, &mut target, "tree/branch2/leaf")
        .expect("Failed to get the file");
    assert_eq!(
        String::from_utf8_lossy(&target),
        "Goodbye World\n",
        "Uncompressed file did not match",
    );
    assert_eq!(written, 14, "Uncompressed bytes count did not match");
}

#[async_std::test]
#[cfg(feature = "futures_support")]
async fn get_a_file_from_tar_futures() {
    let mut source = async_std::fs::File::open("tests/fixtures/tree.tar")
        .await
        .unwrap();
    let mut target = Vec::default();

    let written =
        futures_support::uncompress_archive_file(&mut source, &mut target, "tree/branch2/leaf")
            .await
            .expect("Failed to get the file");
    assert_eq!(
        String::from_utf8_lossy(&target),
        "Goodbye World\n",
        "Uncompressed file did not match",
    );
    assert_eq!(written, 14, "Uncompressed bytes count did not match");
}

#[tokio::test]
#[cfg(feature = "tokio_support")]
async fn get_a_file_from_tar_tokio() {
    let mut source = tokio::fs::File::open("tests/fixtures/tree.tar")
        .await
        .unwrap();
    let mut target = Vec::default();

    let written =
        tokio_support::uncompress_archive_file(&mut source, &mut target, "tree/branch2/leaf")
            .await
            .expect("Failed to get the file");
    assert_eq!(
        String::from_utf8_lossy(&target),
        "Goodbye World\n",
        "Uncompressed file did not match",
    );
    assert_eq!(written, 14, "Uncompressed bytes count did not match");
}

#[test]
fn successfully_list_archive_files() {
    let source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();

    assert_eq!(
        list_archive_files(source).unwrap(),
        vec![
            "tree/".to_string(),
            "tree/branch1/".to_string(),
            "tree/branch1/leaf".to_string(),
            "tree/branch2/".to_string(),
            "tree/branch2/leaf".to_string(),
        ],
        "file list inside the archive did not match"
    );
}

#[test]
fn list_archive_zip() {
    let source = std::fs::File::open("tests/fixtures/test.zip").unwrap();

    assert_eq!(
        list_archive_files(source).unwrap(),
        vec![
            "content/".to_string(),
            "content/first".to_string(),
            "content/third".to_string(),
            "content/nested/".to_string(),
            "content/nested/second".to_string(),
        ],
        "file list inside the archive did not match"
    );
}

#[async_std::test]
#[cfg(feature = "futures_support")]
async fn list_archive_zip_futures() {
    let source = async_std::fs::File::open("tests/fixtures/test.zip")
        .await
        .unwrap();

    assert_eq!(
        futures_support::list_archive_files(source).await.unwrap(),
        vec![
            "content/".to_string(),
            "content/first".to_string(),
            "content/third".to_string(),
            "content/nested/".to_string(),
            "content/nested/second".to_string(),
        ],
        "file list inside the archive did not match"
    );
}

#[tokio::test]
#[cfg(feature = "tokio_support")]
async fn list_archive_zip_tokio() {
    let source = tokio::fs::File::open("tests/fixtures/test.zip")
        .await
        .unwrap();

    assert_eq!(
        tokio_support::list_archive_files(source).await.unwrap(),
        vec![
            "content/".to_string(),
            "content/first".to_string(),
            "content/third".to_string(),
            "content/nested/".to_string(),
            "content/nested/second".to_string(),
        ],
        "file list inside the archive did not match"
    );
}

#[async_std::test]
#[cfg(feature = "futures_support")]
async fn successfully_list_archive_files_futures() {
    let source = async_std::fs::File::open("tests/fixtures/tree.tar")
        .await
        .unwrap();

    assert_eq!(
        futures_support::list_archive_files(source).await.unwrap(),
        vec![
            "tree/".to_string(),
            "tree/branch1/".to_string(),
            "tree/branch1/leaf".to_string(),
            "tree/branch2/".to_string(),
            "tree/branch2/leaf".to_string(),
        ],
        "file list inside the archive did not match"
    );
}

#[tokio::test]
#[cfg(feature = "tokio_support")]
async fn successfully_list_archive_files_tokio() {
    let source = tokio::fs::File::open("tests/fixtures/tree.tar")
        .await
        .unwrap();

    assert_eq!(
        tokio_support::list_archive_files(source).await.unwrap(),
        vec![
            "tree/".to_string(),
            "tree/branch1/".to_string(),
            "tree/branch1/leaf".to_string(),
            "tree/branch2/".to_string(),
            "tree/branch2/leaf".to_string(),
        ],
        "file list inside the archive did not match"
    );
}

#[test]
#[ignore]
#[cfg(unix)]
fn uncompress_to_dir_preserve_owner() {
    use std::os::unix::fs::MetadataExt;

    let dir = tempfile::TempDir::new().expect("Failed to create the tmp directory");
    let mut source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();

    uncompress_archive(&mut source, dir.path(), Ownership::Preserve)
        .expect("Failed to uncompress the file");

    assert!(
        dir.path().join("tree/branch1/leaf").exists(),
        "the path doesn't exist"
    );
    assert!(
        dir.path().join("tree/branch2/leaf").exists(),
        "the path doesn't exist"
    );
    assert_eq!(
        dir.path()
            .join("tree/branch1/leaf")
            .metadata()
            .unwrap()
            .mode()
            % 0o1000,
        0o664,
        "the permissions did not match"
    );
    assert_eq!(
        dir.path()
            .join("tree/branch2/leaf")
            .metadata()
            .unwrap()
            .mode()
            % 0o1000,
        0o664,
        "the permissions did not match"
    );

    let contents = std::fs::read_to_string(dir.path().join("tree/branch2/leaf")).unwrap();
    assert_eq!(
        contents, "Goodbye World\n",
        "Uncompressed file did not match"
    );
}

#[test]
#[ignore]
fn uncompress_same_file_preserve_owner() {
    uncompress_archive(
        &mut std::fs::File::open("tests/fixtures/tree.tar").unwrap(),
        tempfile::TempDir::new()
            .expect("Failed to create the tmp directory")
            .path(),
        Ownership::Preserve,
    )
    .expect("Failed to uncompress the file");
    uncompress_archive(
        &mut std::fs::File::open("tests/fixtures/tree.tar").unwrap(),
        tempfile::TempDir::new()
            .expect("Failed to create the tmp directory")
            .path(),
        Ownership::Preserve,
    )
    .expect("Failed to uncompress the file");
}

#[test]
fn uncompress_to_dir_not_preserve_owner() {
    let dir = tempfile::TempDir::new().expect("Failed to create the tmp directory");
    let mut source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();

    uncompress_archive(&mut source, dir.path(), Ownership::Ignore)
        .expect("Failed to uncompress the file");

    assert!(
        dir.path().join("tree/branch1/leaf").exists(),
        "the path doesn't exist"
    );
    assert!(
        dir.path().join("tree/branch2/leaf").exists(),
        "the path doesn't exist"
    );

    // This block is Unix specific; keep the rest of test platform agnostic.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        assert_eq!(
            dir.path()
                .join("tree/branch1/leaf")
                .metadata()
                .unwrap()
                .permissions()
                .mode()
                % 0o1000,
            0o664,
            "the permissions did not match"
        );
        assert_eq!(
            dir.path()
                .join("tree/branch2/leaf")
                .metadata()
                .unwrap()
                .permissions()
                .mode()
                % 0o1000,
            0o664,
            "the permissions did not match"
        );
    }

    let contents = std::fs::read_to_string(dir.path().join("tree/branch2/leaf")).unwrap();
    assert_eq!(
        contents, "Goodbye World\n",
        "Uncompressed file did not match"
    );
}

#[test]
fn uncompress_7z_to_dir_not_preserve_owner() {
    let dir = tempfile::TempDir::new().expect("Failed to create the tmp directory");
    let mut source = std::fs::File::open("tests/fixtures/tree.7z").unwrap();

    uncompress_archive(&mut source, dir.path(), Ownership::Ignore)
        .expect("Failed to uncompress the file");

    assert!(
        dir.path().join("tree/branch1/leaf").exists(),
        "the path doesn't exist"
    );
    assert!(
        dir.path().join("tree/branch2/leaf").exists(),
        "the path doesn't exist"
    );

    let contents = std::fs::read_to_string(dir.path().join("tree/branch2/leaf")).unwrap();
    assert_eq!(
        contents, "Goodbye World\n",
        "Uncompressed file did not match"
    );
}

#[test]
fn uncompress_to_dir_with_utf8_pathname() {
    let dir = tempfile::TempDir::new().expect("Failed to create the tmp directory");
    let mut source = std::fs::File::open("tests/fixtures/utf8.tar").unwrap();

    uncompress_archive(&mut source, dir.path(), Ownership::Ignore)
        .expect("Failed to uncompress the file");

    assert!(
        dir.path().join("utf-8-file-name-őúíá").exists(),
        "the path doesn't exist"
    );
}

#[test]
fn uncompress_to_dir_with_cjk_pathname() {
    use encoding_rs::{GBK, SHIFT_JIS};

    let dir = tempfile::TempDir::new().expect("Failed to create the tmp directory");
    let mut source_utf8 = std::fs::File::open("tests/fixtures/encoding-utf8.zip").unwrap();
    let mut source_gbk = std::fs::File::open("tests/fixtures/encoding-gbk.zip").unwrap();
    let mut source_sjis = std::fs::File::open("tests/fixtures/encoding-sjis.zip").unwrap();
    let decode_gbk = |bytes: &[u8]| {
        GBK.decode_without_bom_handling_and_without_replacement(bytes)
            .map(String::from)
            .ok_or(Error::Encoding(std::borrow::Cow::Borrowed("GBK failure")))
    };
    let decode_sjis = |bytes: &[u8]| {
        SHIFT_JIS
            .decode_without_bom_handling_and_without_replacement(bytes)
            .map(String::from)
            .ok_or(Error::Encoding(std::borrow::Cow::Borrowed(
                "SHIFT_JIS failure",
            )))
    };

    uncompress_archive_with_encoding(&mut source_utf8, dir.path(), Ownership::Ignore, decode_utf8)
        .expect("Failed to uncompress the file");
    uncompress_archive_with_encoding(&mut source_gbk, dir.path(), Ownership::Ignore, decode_gbk)
        .expect("Failed to uncompress the file");
    uncompress_archive_with_encoding(&mut source_sjis, dir.path(), Ownership::Ignore, decode_sjis)
        .expect("Failed to uncompress the file");

    let read_to_bytes = |path: std::path::PathBuf| {
        use std::io::prelude::*;
        let mut file = std::fs::File::open(path).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        buffer
    };
    let utf8_filepath = dir.path().join("encoding-utf8-chinese-中文.txt");
    let gbk_filepath = dir.path().join("encoding-gbk-chinese-中文.txt");
    let sjis_filepath = dir.path().join("encoding-sjis-japanese-日本語.txt");

    assert!(utf8_filepath.exists(), "the path doesn't exist");
    assert!(gbk_filepath.exists(), "the path doesn't exist");
    assert!(sjis_filepath.exists(), "the path doesn't exist");
    assert_eq!(
        decode_utf8(&read_to_bytes(utf8_filepath)).unwrap(),
        "0123456789中文示例",
        "Uncompressed file did not match"
    );
    assert_eq!(
        decode_gbk(&read_to_bytes(gbk_filepath)).unwrap(),
        "0123456789中文示例",
        "Uncompressed file did not match"
    );
    assert_eq!(
        decode_sjis(&read_to_bytes(sjis_filepath)).unwrap(),
        "0123456789日本語の例",
        "Uncompressed file did not match"
    );
}

#[test]
fn uncompress_same_file_not_preserve_owner() {
    uncompress_archive(
        &mut std::fs::File::open("tests/fixtures/tree.tar").unwrap(),
        tempfile::TempDir::new()
            .expect("Failed to create the tmp directory")
            .path(),
        Ownership::Ignore,
    )
    .expect("Failed to uncompress the file");
    uncompress_archive(
        &mut std::fs::File::open("tests/fixtures/tree.tar").unwrap(),
        tempfile::TempDir::new()
            .expect("Failed to create the tmp directory")
            .path(),
        Ownership::Ignore,
    )
    .expect("Failed to uncompress the file");
}

#[async_std::test]
#[cfg(feature = "futures_support")]
async fn uncompress_same_file_not_preserve_owner_futures() {
    futures_support::uncompress_archive(
        &mut async_std::fs::File::open("tests/fixtures/tree.tar")
            .await
            .unwrap(),
        tempfile::TempDir::new()
            .expect("Failed to create the tmp directory")
            .path(),
        Ownership::Ignore,
    )
    .await
    .expect("Failed to uncompress the file");
    futures_support::uncompress_archive(
        &mut async_std::fs::File::open("tests/fixtures/tree.tar")
            .await
            .unwrap(),
        tempfile::TempDir::new()
            .expect("Failed to create the tmp directory")
            .path(),
        Ownership::Ignore,
    )
    .await
    .expect("Failed to uncompress the file");
}

#[tokio::test]
#[cfg(feature = "tokio_support")]
async fn uncompress_same_file_not_preserve_owner_tokio() {
    tokio_support::uncompress_archive(
        &mut tokio::fs::File::open("tests/fixtures/tree.tar")
            .await
            .unwrap(),
        tempfile::TempDir::new()
            .expect("Failed to create the tmp directory")
            .path(),
        Ownership::Ignore,
    )
    .await
    .expect("Failed to uncompress the file");
    tokio_support::uncompress_archive(
        &mut tokio::fs::File::open("tests/fixtures/tree.tar")
            .await
            .unwrap(),
        tempfile::TempDir::new()
            .expect("Failed to create the tmp directory")
            .path(),
        Ownership::Ignore,
    )
    .await
    .expect("Failed to uncompress the file");
}

#[test]
fn uncompress_truncated_archive() {
    assert!(matches!(
        uncompress_data(
            std::fs::File::open("tests/fixtures/truncated.log.gz").unwrap(),
            Vec::new()
        ),
        Err(Error::Unknown)
    ));
}

#[async_std::test]
#[cfg(feature = "futures_support")]
async fn uncompress_truncated_archive_futures() {
    assert!(matches!(
        futures_support::uncompress_data(
            async_std::fs::File::open("tests/fixtures/truncated.log.gz")
                .await
                .unwrap(),
            Vec::new()
        )
        .await,
        Err(Error::Unknown)
    ));
}

#[tokio::test]
#[cfg(feature = "tokio_support")]
async fn uncompress_truncated_archive_tokio() {
    assert!(matches!(
        tokio_support::uncompress_data(
            tokio::fs::File::open("tests/fixtures/truncated.log.gz")
                .await
                .unwrap(),
            Vec::new()
        )
        .await,
        Err(Error::Unknown)
    ));
}

fn decode_utf8(bytes: &[u8]) -> Result<String> {
    Ok(std::str::from_utf8(bytes)?.to_owned())
}

fn collect_iterate_results_with_encoding(
    source: std::fs::File,
    decode: DecodeCallback,
) -> Vec<(String, usize)> {
    let mut results = Vec::new();
    let mut name = String::default();
    let mut size = 0;

    let mut iter =
        ArchiveIterator::from_read_with_encoding(source, decode).expect("Failed to get the file");

    for content in &mut iter {
        match content {
            ArchiveContents::StartOfEntry(file_name, _) => {
                assert!(name.is_empty());
                assert_eq!(size, 0);
                name = file_name;
            }
            ArchiveContents::DataChunk(data) => {
                assert!(!name.is_empty());
                size += data.len();
            }
            ArchiveContents::EndOfEntry => {
                assert!(!name.is_empty());
                results.push((name, size));
                name = String::default();
                size = 0;
            }
            ArchiveContents::Err(e) => panic!("{:?}", e),
        }
    }

    iter.close().unwrap();
    assert!(name.is_empty());
    assert_eq!(size, 0);

    results
}

fn collect_iterate_results(source: std::fs::File) -> Vec<(String, usize)> {
    collect_iterate_results_with_encoding(source, decode_utf8)
}

#[test]
fn iterate_tar() {
    let source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();

    let contents = collect_iterate_results(source);

    let expected: Vec<(String, usize)> = vec![
        ("tree/", 0),
        ("tree/branch1/", 0),
        ("tree/branch1/leaf", 12),
        ("tree/branch2/", 0),
        ("tree/branch2/leaf", 14),
    ]
    .into_iter()
    .map(|(a, b)| (a.into(), b))
    .collect();

    assert_eq!(contents, expected);
}

fn collect_iterate_names_with_encoding(
    source: std::fs::File,
    decode: DecodeCallback,
) -> Vec<String> {
    let mut results = Vec::new();

    let mut iter =
        ArchiveIterator::from_read_with_encoding(source, decode).expect("Failed to get the file");

    while let Some(content) = iter.next_header() {
        match content {
            ArchiveContents::StartOfEntry(file_name, _) => {
                results.push(file_name);
            }
            ArchiveContents::DataChunk(_) => {
                panic!("expected StartOfntry got DataChunk")
            }
            ArchiveContents::EndOfEntry => {
                panic!("expected StartOfntry got EndOfEntry")
            }
            ArchiveContents::Err(e) => panic!("{:?}", e),
        }
    }

    iter.close().unwrap();

    results
}

#[test]
fn iterate_tar_names() {
    let source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();

    let contents = collect_iterate_names_with_encoding(source, decode_utf8);

    let expected: Vec<String> = vec![
        "tree/",
        "tree/branch1/",
        "tree/branch1/leaf",
        "tree/branch2/",
        "tree/branch2/leaf",
    ]
    .into_iter()
    .map(|a| a.into())
    .collect();

    assert_eq!(contents, expected);
}

#[test]
fn iterate_7z() {
    let source = std::fs::File::open("tests/fixtures/tree.7z").unwrap();

    let contents = collect_iterate_results(source);

    let expected: Vec<(String, usize)> = vec![
        ("tree/", 0),
        ("tree/branch1/", 0),
        ("tree/branch2/", 0),
        ("tree/branch1/leaf", 12),
        ("tree/branch2/leaf", 14),
    ]
    .into_iter()
    .map(|(a, b)| (a.into(), b))
    .collect();

    assert_eq!(contents, expected);
}

#[test]
fn iterate_zip_with_cjk_pathname() {
    use encoding_rs::GBK;

    let source = std::fs::File::open("tests/fixtures/encoding-gbk-tree.zip").unwrap();

    let decode_gbk = |bytes: &[u8]| {
        GBK.decode_without_bom_handling_and_without_replacement(bytes)
            .map(String::from)
            .ok_or(Error::Encoding(std::borrow::Cow::Borrowed("GBK failure")))
    };
    let contents = collect_iterate_results_with_encoding(source, decode_gbk);

    let expected: Vec<(String, usize)> = vec![
        ("tree/", 0),
        ("tree/branch1/", 0),
        ("tree/branch1/leaf1-encoding-gbk-chinese-中文.txt", 18),
        ("tree/branch2/", 0),
        ("tree/branch2/leaf2-encoding-gbk-chinese-中文.txt", 12),
    ]
    .into_iter()
    .map(|(a, b)| (a.into(), b))
    .collect();

    assert_eq!(contents, expected);
}

#[test]
fn iterate_truncated_archive() {
    let source = std::fs::File::open("tests/fixtures/truncated.log.gz").unwrap();

    for content in ArchiveIterator::from_read(source).unwrap() {
        if let ArchiveContents::Err(Error::Unknown) = content {
            return;
        }
    }

    panic!("Did not find expected error");
}

fn uncompress_bytes_helper(bytes: &[u8]) {
    let wrapper = Cursor::new(bytes);

    for content in ArchiveIterator::from_read(wrapper).unwrap() {
        if let ArchiveContents::Err(Error::Unknown) = content {
            return;
        }
    }

    panic!("Did not find expected error");
}

#[test]
fn uncompress_bytes() {
    let mut source = std::fs::File::open("tests/fixtures/truncated.log.gz").unwrap();

    let mut buffer = Vec::new();
    source.read_to_end(&mut buffer).unwrap();

    uncompress_bytes_helper(&buffer)
}

#[test]
fn uncompress_archive_zip_slip_vulnerability() {
    uncompress_archive(
        &mut std::fs::File::open("tests/fixtures/zip-slip.zip").unwrap(),
        tempfile::TempDir::new()
            .expect("Failed to create the tmp directory")
            .path(),
        Ownership::Ignore,
    )
    .unwrap_err();
}

#[test]
fn uncompress_archive_absolute_path() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create the tmp directory");
    let dest = temp_dir.path();

    let correct_dest = dest.join("test.txt");
    let incorrect_dest = Path::new("/test.txt");

    uncompress_archive(
        &mut std::fs::File::open("tests/fixtures/absolute-path.tar").unwrap(),
        dest,
        Ownership::Ignore,
    )
    .unwrap();

    assert!(correct_dest.exists());
    assert!(!Path::new(incorrect_dest).exists());
}

#[test]
fn decode_failure() {
    let source = std::fs::File::open("tests/fixtures/file.txt.gz").unwrap();
    let decode_fail = |_bytes: &[u8]| Err(Error::Io(std::io::Error::from(ErrorKind::BrokenPipe)));

    for content in ArchiveIterator::from_read_with_encoding(source, decode_fail).unwrap() {
        if let ArchiveContents::Err(Error::Io(err)) = content {
            if err.kind() == ErrorKind::BrokenPipe {
                return;
            }
        }
    }

    panic!("Did not find expected error");
}

#[test]
fn decode_chinese_zip() {
    let source = std::fs::File::open("tests/fixtures/chinese-name.zip").unwrap();
    let files = list_archive_files(source).expect("Failed to list archives");
    let expected = ["中文/", "中文/文件/"];
    assert_eq!(files, expected);
}

#[test]
fn iterate_archive_with_filter_name() {
    let source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();
    let expected_name = "leaf";

    let mut entries = Vec::new();
    for content in ArchiveIteratorBuilder::new(source)
        .filter(|name, _stat| Path::new(name).file_name() == Some(OsStr::new(expected_name)))
        .build()
        .unwrap()
    {
        if let ArchiveContents::StartOfEntry(name, _stat) = content {
            entries.push(name);
        }
    }

    assert_eq!(
        entries,
        vec![
            "tree/branch1/leaf".to_string(),
            "tree/branch2/leaf".to_string(),
        ],
        "filtered file list inside the archive did not match"
    );
}

#[test]
fn iterate_archive_with_filter_type() {
    let source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();

    let mut entries = Vec::new();
    #[allow(clippy::unnecessary_cast)]
    for content in ArchiveIteratorBuilder::new(source)
        .filter(|_name, stat| {
            /* Use explicit casts to achieve windows portability,
             * see https://github.com/rust-lang/libc/issues/3161 */
            (stat.st_mode as u32 & libc::S_IFMT as u32) == S_IFREG as u32
        })
        .build()
        .unwrap()
    {
        if let ArchiveContents::StartOfEntry(name, _stat) = content {
            entries.push(name);
        }
    }

    assert_eq!(
        entries,
        vec![
            "tree/branch1/leaf".to_string(),
            "tree/branch2/leaf".to_string(),
        ],
        "filtered file list inside the archive did not match"
    );
}

#[test]
fn iterate_archive_with_filter_path() {
    let source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();

    let mut entries = Vec::new();
    for content in ArchiveIteratorBuilder::new(source)
        .filter(|name, _stat| name.starts_with("tree/branch2/"))
        .build()
        .unwrap()
    {
        if let ArchiveContents::StartOfEntry(name, _stat) = content {
            entries.push(name);
        }
    }

    assert_eq!(
        entries,
        vec!["tree/branch2/".to_string(), "tree/branch2/leaf".to_string(),],
        "filtered file list inside the archive did not match"
    );
}

#[test]
fn test_slice_from_raw_parts() {
    let mut source = std::fs::File::open("tests/fixtures/slice_from_raw_parts.zip").unwrap();
    let mut outfile = tempfile::NamedTempFile::new().unwrap();
    uncompress_archive_file(&mut source, &mut outfile, "1/2/1.txt").unwrap();
}
