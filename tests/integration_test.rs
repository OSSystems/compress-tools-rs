// Copyright (C) 2019-2021 O.S. Systems Software LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use compress_tools::*;
use std::path::Path;

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

fn collect_iterate_results(source: std::fs::File) -> Vec<(String, usize)> {
    let mut results = Vec::new();
    let mut name = String::default();
    let mut size = 0;

    let mut iter = ArchiveIterator::from_read(source).expect("Failed to get the file");

    for content in &mut iter {
        match content {
            ArchiveContents::StartOfEntry(file_name) => {
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

    assert!(iter.close().is_ok());
    assert!(name.is_empty());
    assert_eq!(size, 0);

    results
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
fn iterate_truncated_archive() {
    let source = std::fs::File::open("tests/fixtures/truncated.log.gz").unwrap();

    for content in ArchiveIterator::from_read(source).unwrap() {
        if let ArchiveContents::Err(Error::Unknown) = content {
            return;
        }
    }

    panic!("Did not find expected error");
}

#[test]
fn uncompress_archive_zip_slip_vulnerability() {
    assert!(
        uncompress_archive(
            &mut std::fs::File::open("tests/fixtures/zip-slip.zip").unwrap(),
            tempfile::TempDir::new()
                .expect("Failed to create the tmp directory")
                .path(),
            Ownership::Ignore,
        )
        .is_err(),
        "SECURITY ERROR: evil.txt has been uncompressed in /tmp!"
    );
}

#[test]
fn uncompress_archive_absolute_path() {
    let temp_dir = tempfile::TempDir::new().expect("Failed to create the tmp directory");
    let dest = temp_dir.path();

    let correct_dest = dest.join("test.txt");
    let incorrect_dest = Path::new("/test.txt");

    assert!(
        uncompress_archive(
            &mut std::fs::File::open("tests/fixtures/absolute-path.tar").unwrap(),
            dest,
            Ownership::Ignore,
        )
        .is_ok(),
        "SECURITY ERROR: test.txt has been uncompressed in /!"
    );

    assert!(correct_dest.exists());
    assert!(!Path::new(incorrect_dest).exists());
}
