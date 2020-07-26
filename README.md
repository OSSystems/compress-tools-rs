[![Coverage Status](https://coveralls.io/repos/github/OSSystems/compress-tools-rs/badge.svg?branch=master)](https://coveralls.io/github/OSSystems/compress-tools-rs?branch=master)
[![Documentation](https://docs.rs/compress-tools/badge.svg)](https://docs.rs/compress-tools)

# compress-tools

The `compress-tools` crate aims to provide a convenient and easy to use set
of methods which builds on top of `libarchive` exposing a small set of it’s
functionalities.

| Platform | Build Status |
| -------- | ------------ |
| Linux - x86_64 | [![build status](https://github.com/OSSystems/compress-tools-rs/workflows/CI%20-%20Linux%20-%20x86_64/badge.svg)](https://github.com/OSSystems/compress-tools-rs/actions) |
| Linux - AArch64 | [![build status](https://github.com/OSSystems/compress-tools-rs/workflows/CI%20-%20Linux%20-%20AArch64/badge.svg)](https://github.com/OSSystems/compress-tools-rs/actions) |
| Linux - ARMv7 | [![build status](https://github.com/OSSystems/compress-tools-rs/workflows/CI%20-%20Linux%20-%20ARMv7/badge.svg)](https://github.com/OSSystems/compress-tools-rs/actions) |
| macOS - x86_64 | [![build status](https://github.com/OSSystems/compress-tools-rs/workflows/CI%20-%20macOS%20-%20x86_64/badge.svg)](https://github.com/OSSystems/compress-tools-rs/actions) |
| Windows - x86_64 | [![build status](https://github.com/OSSystems/compress-tools-rs/workflows/CI%20-%20Windows%20-%20x86_64/badge.svg)](https://github.com/OSSystems/compress-tools-rs/actions) |

---

## Dependencies

You must have `libarchive` properly installed on your system in order to use
this. If building on *nix systems, `pkg-config` is used to locate the
`libarchive`; on Windows `vcpkg` will be used to locating the `libarchive`.

The minimum supported Rust version is 1.40.

## Features

This crate is capable of extracting:

* compressed files
* archive files
* single file from an archive

For example, to extract an archive file it is as simple as:

```rust
use compress_tools::*;
use std::fs::File;
use std::path::Path;

let mut source = File::open("tree.tar.gz")?;
let dest = Path::new("/tmp/dest");

uncompress_archive(&mut source, &dest, Ownership::Preserve)?;
```

Also, this crate can be used in async code. It requires to enable the `async` feature.

As an example, this code extracts file into Vec from an archive in async environment:
```rust

use compress_tools::asynchronous::*;
use tokio::fs::File;

let mut source = File::open("test.tar.gz").await?;
let mut target = Vec::default();

uncompress_archive_file(&mut source, &mut target, "test.txt").await?;
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
