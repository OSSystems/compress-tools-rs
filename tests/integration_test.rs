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

#[test]
#[cfg(feature = "futures_support")]
fn get_compressed_file_content_futures() {
    smol::block_on(async {
        let mut source = smol::fs::File::open("tests/fixtures/file.txt.gz")
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
    });
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

#[test]
#[cfg(feature = "futures_support")]
fn get_a_file_from_tar_futures() {
    smol::block_on(async {
        let mut source = smol::fs::File::open("tests/fixtures/tree.tar")
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
    });
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
fn reader_is_rewound_between_calls() {
    let mut source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();

    let listed = list_archive_files(&mut source).unwrap();
    assert!(listed.contains(&"tree/branch2/leaf".to_string()));

    let mut target = Vec::new();
    let written = uncompress_archive_file(&mut source, &mut target, "tree/branch2/leaf")
        .expect("extract should succeed even though the reader was left past the archive");
    assert_eq!(written, 14);
    assert_eq!(String::from_utf8_lossy(&target), "Goodbye World\n");

    let listed_again = list_archive_files(&mut source).unwrap();
    assert_eq!(listed, listed_again);

    let names: Vec<String> = ArchiveIteratorBuilder::new(&mut source)
        .build()
        .unwrap()
        .filter_map(|c| match c {
            ArchiveContents::StartOfEntry(name, _) => Some(name),
            _ => None,
        })
        .collect();
    assert_eq!(names, listed);
}

#[test]
fn successfully_list_archive_entries() {
    let source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();

    let entries = list_archive_entries(source).unwrap();
    let observed: Vec<(String, u64)> = entries.into_iter().map(|e| (e.path, e.size)).collect();

    assert_eq!(
        observed,
        vec![
            ("tree/".to_string(), 0),
            ("tree/branch1/".to_string(), 0),
            ("tree/branch1/leaf".to_string(), 12),
            ("tree/branch2/".to_string(), 0),
            ("tree/branch2/leaf".to_string(), 14),
        ],
        "entry list (path, size) did not match"
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

#[test]
#[cfg(feature = "futures_support")]
fn list_archive_zip_futures() {
    smol::block_on(async {
        let source = smol::fs::File::open("tests/fixtures/test.zip")
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
    });
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

#[test]
#[cfg(feature = "futures_support")]
fn extract_archive_zip_futures() {
    smol::block_on(async {
        let dir = tempfile::TempDir::new().expect("Failed to create the tmp directory");
        let source = smol::fs::File::open("tests/fixtures/test.zip")
            .await
            .unwrap();

        futures_support::uncompress_archive(source, dir.path(), Ownership::Ignore)
            .await
            .expect("Failed to extract zip");

        assert!(dir.path().join("content/first").exists());
        assert!(dir.path().join("content/third").exists());
        assert!(dir.path().join("content/nested/second").exists());
    });
}

#[tokio::test]
#[cfg(feature = "tokio_support")]
async fn extract_archive_zip_tokio() {
    let dir = tempfile::TempDir::new().expect("Failed to create the tmp directory");
    let source = tokio::fs::File::open("tests/fixtures/test.zip")
        .await
        .unwrap();

    tokio_support::uncompress_archive(source, dir.path(), Ownership::Ignore)
        .await
        .expect("Failed to extract zip");

    assert!(dir.path().join("content/first").exists());
    assert!(dir.path().join("content/third").exists());
    assert!(dir.path().join("content/nested/second").exists());
}

#[test]
#[cfg(feature = "futures_support")]
fn uncompress_archive_file_zip_futures() {
    smol::block_on(async {
        let source = smol::fs::File::open("tests/fixtures/test.zip")
            .await
            .unwrap();
        let mut target = Vec::<u8>::default();

        futures_support::uncompress_archive_file(source, &mut target, "content/first")
            .await
            .expect("Failed to extract content/first");

        assert!(
            !target.is_empty(),
            "extracted file content should not be empty"
        );
    });
}

#[tokio::test]
#[cfg(feature = "tokio_support")]
async fn uncompress_archive_file_zip_tokio() {
    let source = tokio::fs::File::open("tests/fixtures/test.zip")
        .await
        .unwrap();
    let mut target = Vec::<u8>::default();

    tokio_support::uncompress_archive_file(source, &mut target, "content/first")
        .await
        .expect("Failed to extract content/first");

    assert!(
        !target.is_empty(),
        "extracted file content should not be empty"
    );
}

#[test]
#[cfg(feature = "futures_support")]
fn iterate_archive_zip_futures() {
    use futures_util::stream::StreamExt;

    smol::block_on(async {
        let source = smol::fs::File::open("tests/fixtures/test.zip")
            .await
            .unwrap();

        let mut iter = futures_support::ArchiveIteratorBuilder::new(source).build();

        let mut names = Vec::new();
        while let Some(content) = iter.next().await {
            if let ArchiveContents::StartOfEntry(name, _) = content {
                names.push(name);
            }
        }

        assert_eq!(
            names,
            vec![
                "content/".to_string(),
                "content/first".to_string(),
                "content/third".to_string(),
                "content/nested/".to_string(),
                "content/nested/second".to_string(),
            ],
        );
    });
}

#[tokio::test]
#[cfg(feature = "tokio_support")]
async fn iterate_archive_zip_tokio() {
    use futures_util::stream::StreamExt;

    let source = tokio::fs::File::open("tests/fixtures/test.zip")
        .await
        .unwrap();

    let mut iter = tokio_support::ArchiveIteratorBuilder::new(source).build();

    let mut names = Vec::new();
    while let Some(content) = iter.next().await {
        if let ArchiveContents::StartOfEntry(name, _) = content {
            names.push(name);
        }
    }

    assert_eq!(
        names,
        vec![
            "content/".to_string(),
            "content/first".to_string(),
            "content/third".to_string(),
            "content/nested/".to_string(),
            "content/nested/second".to_string(),
        ],
    );
}

#[test]
#[cfg(feature = "futures_support")]
fn successfully_list_archive_files_futures() {
    smol::block_on(async {
        let source = smol::fs::File::open("tests/fixtures/tree.tar")
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
    });
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
fn get_a_file_from_rar() {
    let mut source = std::fs::File::open("tests/fixtures/tree.rar").unwrap();
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
fn iterate_rar_entries() {
    let source = std::fs::File::open("tests/fixtures/tree.rar").unwrap();

    let mut names = Vec::new();
    let mut content = Vec::new();

    for item in ArchiveIterator::from_read(source).expect("Failed to read archive") {
        match item {
            ArchiveContents::StartOfEntry(name, _) => names.push(name),
            ArchiveContents::DataChunk(chunk) => {
                if names
                    .last()
                    .map(|n| n == "tree/branch2/leaf")
                    .unwrap_or(false)
                {
                    content.extend_from_slice(&chunk);
                }
            }
            ArchiveContents::EndOfEntry => {}
            ArchiveContents::Err(e) => panic!("iterator errored: {}", e),
        }
    }

    assert!(
        names.iter().any(|n| n == "tree/branch1"),
        "directory entry missing from iteration: {:?}",
        names
    );
    assert!(
        names.iter().any(|n| n == "tree/branch2/leaf"),
        "tree/branch2/leaf missing from iteration: {:?}",
        names
    );
    assert_eq!(String::from_utf8_lossy(&content), "Goodbye World\n");
}

#[test]
fn uncompress_rar_to_dir() {
    let dir = tempfile::TempDir::new().expect("Failed to create the tmp directory");
    let mut source = std::fs::File::open("tests/fixtures/tree.rar").unwrap();

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

#[test]
#[cfg(feature = "futures_support")]
fn uncompress_same_file_not_preserve_owner_futures() {
    smol::block_on(async {
        futures_support::uncompress_archive(
            &mut smol::fs::File::open("tests/fixtures/tree.tar")
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
            &mut smol::fs::File::open("tests/fixtures/tree.tar")
                .await
                .unwrap(),
            tempfile::TempDir::new()
                .expect("Failed to create the tmp directory")
                .path(),
            Ownership::Ignore,
        )
        .await
        .expect("Failed to uncompress the file");
    });
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

#[test]
#[cfg(feature = "futures_support")]
fn uncompress_truncated_archive_futures() {
    smol::block_on(async {
        assert!(matches!(
            futures_support::uncompress_data(
                smol::fs::File::open("tests/fixtures/truncated.log.gz")
                    .await
                    .unwrap(),
                Vec::new()
            )
            .await,
            Err(Error::Unknown)
        ));
    });
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
            ArchiveContents::StartOfEntry(file_name, stat) => {
                assert!(name.is_empty());
                assert_eq!(size, 0);
                assert_eq!(stat.st_size == 0, file_name.ends_with('/'));
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

/// Regression test for <https://github.com/OSSystems/compress-tools-rs/issues/138>.
///
/// Verifies that the `stat` fields surfaced via
/// `ArchiveContents::StartOfEntry` match the values actually stored in
/// `tree.tar`. On Windows the crate used to map `archive_entry_stat()` onto
/// `libc::stat` — which on that target is `stat64`, a different layout — so
/// `st_size` and the `st_*time` fields were read from the wrong offsets and
/// returned garbage. A mis-aligned struct fails this test immediately because
/// the expected values are fixed and verifiable from the archive.
#[test]
// `st_size` is `i32` on Windows and `i64` on Unix; `st_mtime` is `time_t`
// (64-bit on both). The `i64::from`/`.into()` calls below normalise the
// values across platforms — on Unix that's a no-op conversion, hence the
// allow.
#[allow(clippy::useless_conversion)]
fn iterate_tar_stat_fields() {
    let source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();

    let mut iter = ArchiveIterator::from_read(source).expect("Failed to read archive");

    let mut stats: Vec<(String, i64, i64)> = Vec::new();
    for content in &mut iter {
        if let ArchiveContents::StartOfEntry(file_name, stat) = content {
            stats.push((file_name, i64::from(stat.st_size), stat.st_mtime.into()));
        }
    }
    iter.close().unwrap();

    let expected: Vec<(&str, i64, i64)> = vec![
        ("tree/", 0, 1_556_038_329),
        ("tree/branch1/", 0, 1_556_038_347),
        ("tree/branch1/leaf", 12, 1_556_038_389),
        ("tree/branch2/", 0, 1_556_038_351),
        ("tree/branch2/leaf", 14, 1_556_038_397),
    ];

    assert_eq!(stats.len(), expected.len(), "entry count mismatch");
    for (got, want) in stats.iter().zip(expected.iter()) {
        assert_eq!(got.0, want.0, "name mismatch");
        assert_eq!(got.1, want.1, "st_size mismatch for {}", got.0);
        assert_eq!(got.2, want.2, "st_mtime mismatch for {}", got.0);
    }
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

    for content in ArchiveIteratorBuilder::new(source)
        .raw_format(true)
        .build()
        .unwrap()
    {
        if let ArchiveContents::Err(Error::Unknown) = content {
            return;
        }
    }

    panic!("Did not find expected error");
}

fn uncompress_bytes_helper(bytes: &[u8]) {
    let wrapper = Cursor::new(bytes);

    for content in ArchiveIteratorBuilder::new(wrapper)
        .raw_format(true)
        .build()
        .unwrap()
    {
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
    let source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();
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

/// Regression test for <https://github.com/OSSystems/compress-tools-rs/issues/136>.
#[test]
fn deflate64_zip_listing_rejects_unsupported_method() {
    let source = std::fs::File::open("tests/fixtures/deflate64.zip").unwrap();
    match list_archive_files(source).expect_err("listing should fail on Deflate64") {
        Error::UnsupportedZipCompression(entries) => {
            assert_eq!(entries, vec![("file.txt".to_string(), 9)]);
        }
        other => panic!("expected Error::UnsupportedZipCompression, got {other:?}"),
    }
}

#[test]
fn deflate64_zip_extraction_rejects_unsupported_method() {
    let mut source = std::fs::File::open("tests/fixtures/deflate64.zip").unwrap();
    let dir = tempfile::TempDir::new().expect("Failed to create the tmp directory");
    match uncompress_archive(&mut source, dir.path(), Ownership::Ignore)
        .expect_err("expected extraction to fail on Deflate64")
    {
        Error::UnsupportedZipCompression(entries) => {
            assert_eq!(entries, vec![("file.txt".to_string(), 9)]);
        }
        other => panic!("expected Error::UnsupportedZipCompression, got {other:?}"),
    }
}

#[test]
fn deflate64_zip_iterator_rejects_unsupported_method() {
    let source = std::fs::File::open("tests/fixtures/deflate64.zip").unwrap();
    match ArchiveIterator::from_read(source)
        .err()
        .expect("iterator build should fail on Deflate64")
    {
        Error::UnsupportedZipCompression(entries) => {
            assert_eq!(entries, vec![("file.txt".to_string(), 9)]);
        }
        other => panic!("expected Error::UnsupportedZipCompression, got {other:?}"),
    }
}

#[test]
fn deflate_zip_listing_still_works() {
    let source = std::fs::File::open("tests/fixtures/encoding-utf8.zip").unwrap();
    list_archive_files(source).expect("plain deflate ZIP must keep working");
}

#[test]
fn test_slice_from_raw_parts() {
    let mut source = std::fs::File::open("tests/fixtures/slice_from_raw_parts.zip").unwrap();
    let mut outfile = tempfile::NamedTempFile::new().unwrap();
    uncompress_archive_file(&mut source, &mut outfile, "1/2/1.txt").unwrap();
}

// The fixture is encrypted with WinZip AES (ZIP method 99). vcpkg's
// `x64-windows-static` libarchive build omits the AES crypto backend, so
// `archive_read_data_block` fails at runtime on that triplet even though the
// dynamic Windows build decrypts it correctly.
#[cfg(not(all(windows, target_feature = "crt-static")))]
#[test]
fn iterate_archive_with_password() {
    let source = std::fs::File::open("tests/fixtures/with-password.zip").unwrap();
    let password = ArchivePassword::new("123").unwrap();

    let mut files_result: Vec<String> = Vec::new();
    let mut current_file_content: Vec<u8> = vec![];
    let mut current_file_name = String::new();

    let mut iter = ArchiveIteratorBuilder::new(source)
        .with_password(password)
        .filter(|name, _| name.ends_with(".txt"))
        .build()
        .unwrap();

    for content in &mut iter {
        match content {
            ArchiveContents::StartOfEntry(name, _stat) => {
                current_file_name = name;
            }
            ArchiveContents::DataChunk(dt) => {
                current_file_content.extend(dt);
            }
            ArchiveContents::EndOfEntry => {
                let content_raw = String::from_utf8(current_file_content.clone()).unwrap();
                current_file_content.clear();

                let content = format!("{}={}", current_file_name, content_raw);
                files_result.push(content);
            }
            _ => {}
        }
    }

    iter.close().unwrap();

    assert_eq!(files_result.len(), 2);
    assert_eq!(
        files_result,
        vec![
            "with-password/file1.txt=its encrypted file".to_string(),
            "with-password/file2.txt=file 2 in archive encrypted!".to_string()
        ]
    );
}

#[test]
fn iterate_encrypted_archive_without_password_errors() {
    let source = std::fs::File::open("tests/fixtures/with-password.zip").unwrap();

    let iter = ArchiveIterator::from_read(source).unwrap();
    let saw_error = iter
        .into_iter()
        .any(|c| matches!(c, ArchiveContents::Err(_)));
    assert!(
        saw_error,
        "iterating an encrypted archive without a password should surface an error",
    );
}

#[test]
fn iterate_encrypted_archive_with_wrong_password_errors() {
    let source = std::fs::File::open("tests/fixtures/with-password.zip").unwrap();
    let password = ArchivePassword::new("wrong").unwrap();

    let iter = ArchiveIteratorBuilder::new(source)
        .with_password(password)
        .build()
        .unwrap();
    let saw_error = iter
        .into_iter()
        .any(|c| matches!(c, ArchiveContents::Err(_)));
    assert!(
        saw_error,
        "iterating an encrypted archive with a wrong password should surface an error",
    );
}

#[test]
fn archive_password_with_nul_byte_rejected() {
    assert!(
        ArchivePassword::new("abc\0def").is_err(),
        "passwords containing NUL must be rejected, not panic",
    );
}

// Arbitrary binary blob that matches no libarchive format handler. Used to
// prove the "raw" handler is the thing making this parseable: strict mode
// errors, raw_format(true) accepts it as a single "data" entry.
const NON_ARCHIVE_BYTES: &[u8] = &[
    0x00, 0x01, 0x02, 0x03, 0xff, 0xfe, 0xfd, 0xfc, 0x10, 0x20, 0x30, 0x40, 0x50, 0x60, 0x70, 0x80,
];

#[test]
fn list_archive_files_rejects_non_archive_bytes() {
    let source = Cursor::new(NON_ARCHIVE_BYTES);
    assert!(
        list_archive_files(source).is_err(),
        "arbitrary bytes must no longer be listed as a single \"data\" entry",
    );
}

#[test]
fn uncompress_archive_rejects_non_archive_bytes() {
    let source = Cursor::new(NON_ARCHIVE_BYTES);
    let dir = tempfile::TempDir::new().unwrap();
    assert!(uncompress_archive(source, dir.path(), Ownership::Ignore).is_err());
}

#[test]
fn iterator_default_rejects_non_archive_bytes() {
    let source = Cursor::new(NON_ARCHIVE_BYTES);
    let saw_err = match ArchiveIterator::from_read(source) {
        Err(_) => true,
        Ok(iter) => iter
            .into_iter()
            .any(|c| matches!(c, ArchiveContents::Err(_))),
    };
    assert!(
        saw_err,
        "strict iterator must surface an error on non-archive input"
    );
}

#[test]
fn iterator_mtree_format_opt_out_rejects_gzip_text() {
    let source = std::fs::File::open("tests/fixtures/file.txt.gz").unwrap();
    let saw_err = match ArchiveIteratorBuilder::new(source)
        .mtree_format(false)
        .build()
    {
        Err(_) => true,
        Ok(mut iter) => iter.any(|c| matches!(c, ArchiveContents::Err(_))),
    };
    assert!(
        saw_err,
        "mtree_format(false) must reject libarchive's permissive mtree match on plain text"
    );
}

#[test]
fn iterator_raw_format_opt_in_accepts_non_archive_bytes() {
    let source = Cursor::new(NON_ARCHIVE_BYTES);
    let mut names = Vec::new();
    for content in ArchiveIteratorBuilder::new(source)
        .raw_format(true)
        .build()
        .expect("raw_format(true) should accept arbitrary bytes")
    {
        if let ArchiveContents::StartOfEntry(name, _) = content {
            names.push(name);
        }
    }
    assert_eq!(names, vec!["data".to_string()]);
}
