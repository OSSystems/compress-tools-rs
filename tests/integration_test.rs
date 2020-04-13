// Copyright (C) 2019, 2020 O.S. Systems Sofware LTDA
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use compress_tools::*;

#[test]
fn get_compressed_file_content() {
    let mut source = std::fs::File::open("tests/fixtures/file.txt.gz").unwrap();
    let mut target = Vec::default();

    uncompress_file(&mut source, &mut target).expect("Failed to uncompress the file");
    assert_eq!(
        String::from_utf8_lossy(&target),
        "some_file_content\n",
        "Uncompressed file did not match",
    );
}

#[test]
fn get_a_file_from_tar() {
    let mut source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();
    let mut target = Vec::default();

    uncompress_archive_file(&mut source, &mut target, &"tree/branch2/leaf")
        .expect("Failed to get the file");
    assert_eq!(
        String::from_utf8_lossy(&target),
        "Goodbye World\n",
        "Uncompressed file did not match",
    );
}

#[test]
#[ignore]
fn uncompress_to_dir_preserve_owner() {
    use std::os::unix::fs::MetadataExt;
    use tempfile;

    let dir = tempfile::TempDir::new().expect("Failed to create the tmp directory");
    let mut source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();

    uncompress_archive(&mut source, dir.path(), Ownership::Preserve)
        .expect("Failed to uncompress the file");

    assert_eq!(
        dir.path().join("tree/branch1/leaf").exists(),
        true,
        "the path doesn't exist"
    );
    assert_eq!(
        dir.path().join("tree/branch2/leaf").exists(),
        true,
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
    use tempfile;

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
    use std::os::unix::fs::PermissionsExt;
    use tempfile;

    let dir = tempfile::TempDir::new().expect("Failed to create the tmp directory");
    let mut source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();

    uncompress_archive(&mut source, dir.path(), Ownership::Ignore)
        .expect("Failed to uncompress the file");

    assert_eq!(
        dir.path().join("tree/branch1/leaf").exists(),
        true,
        "the path doesn't exist"
    );
    assert_eq!(
        dir.path().join("tree/branch2/leaf").exists(),
        true,
        "the path doesn't exist"
    );
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

    let contents = std::fs::read_to_string(dir.path().join("tree/branch2/leaf")).unwrap();
    assert_eq!(
        contents, "Goodbye World\n",
        "Uncompressed file did not match"
    );
}

#[test]
fn uncompress_same_file_not_preserve_owner() {
    use tempfile;

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
