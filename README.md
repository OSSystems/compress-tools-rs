[![Build Status](https://travis-ci.org/OSSystems/compress-tools-rs.svg?branch=master)](https://travis-ci.org/OSSystems/compress-tools-rs) [![Documentation](https://docs.rs/compress-tools/badge.svg)](https://docs.rs/compress-tools)

# compress-tools

The library provide tools for handling compressed and archive files

## Examples
```rust
let dir = tempfile::tempdir().unwrap();
uncompress("tests/fixtures/tree.tar.gz", dir.path(), Kind::TarGZip).unwrap();
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
