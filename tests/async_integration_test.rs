// Copyright (C) 2019, 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

#[cfg(feature = "async")]
use compress_tools::asynchronous::*;

#[cfg(feature = "async")]
#[tokio::test]
async fn get_compressed_file_content() {
    let mut source = tokio::fs::File::open("tests/fixtures/file.txt.gz")
        .await
        .unwrap();
    let mut target = Vec::default();

    let written = uncompress_data(&mut source, &mut target)
        .await
        .expect("Failed to uncompress the file");
    assert_eq!(
        String::from_utf8_lossy(&target),
        "some_file_content\n",
        "Uncompressed file did not match",
    );
    assert_eq!(written, 18, "Uncompressed bytes count did not match");
}

#[cfg(feature = "async")]
#[tokio::test]
async fn get_a_file_from_tar() {
    let mut source = tokio::fs::File::open("tests/fixtures/tree.tar")
        .await
        .unwrap();
    let mut target = Vec::default();

    let written = uncompress_archive_file(&mut source, &mut target, "tree/branch2/leaf")
        .await
        .expect("Failed to get the file");
    assert_eq!(
        String::from_utf8_lossy(&target),
        "Goodbye World\n",
        "Uncompressed file did not match",
    );
    assert_eq!(written, 14, "Uncompressed bytes count did not match");
}

#[cfg(feature = "async")]
#[tokio::test]
async fn successfully_list_archive_files() {
    let source = tokio::fs::File::open("tests/fixtures/tree.tar")
        .await
        .unwrap();

    assert_eq!(
        list_archive_files(source).await.unwrap(),
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
