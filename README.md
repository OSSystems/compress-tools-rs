[![Build Status](https://travis-ci.org/OSSystems/compress-tools-rs.svg?branch=master)](https://travis-ci.org/OSSystems/compress-tools-rs) [![Documentation](https://docs.rs/compress-tools/badge.svg)](https://docs.rs/compress-tools)

# compress-tools

The `compress-tools` crate aims to provide a convenient and easy to use set
of methods which builds on top of `libarchive` exposing a small set of it’s
functionalities.

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
