[![Build Status](https://travis-ci.org/OSSystems/compress-tools-rs.svg?branch=master)](https://travis-ci.org/OSSystems/compress-tools-rs) [![Documentation](https://docs.rs/compress-tools/badge.svg)](https://docs.rs/compress-tools)

# compress-tools

The library provides tools for handling compressed and archive files.

## Examples Uncompress
### Archive
```rust
let dir = tempfile::TempDir::new().expect("Failed to create the tmp directory");
let mut source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();

uncompress_archive(&mut source, dir.path())?;
```

### Archive file
```rust
let mut source = std::fs::File::open("tests/fixtures/tree.tar").unwrap();
let mut target = Vec::default();

uncompress_archive_file(&mut source, &mut target, &"tree/branch2/leaf")?;
```

### File
```rust
let mut source = std::fs::File::open("tests/fixtures/file.txt.gz").unwrap();
let mut target = Vec::default();

uncompress_file(&mut source, &mut target)?;
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or https://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or https://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the
Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
