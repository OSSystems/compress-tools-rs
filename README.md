[![Coverage Status](https://coveralls.io/repos/github/OSSystems/compress-tools-rs/badge.svg?branch=master)](https://coveralls.io/github/OSSystems/compress-tools-rs?branch=master)
[![Crates.io](https://img.shields.io/crates/v/compress-tools.svg)](https://crates.io/crates/compress-tools)
[![Documentation](https://docs.rs/compress-tools/badge.svg)](https://docs.rs/compress-tools)

# compress-tools

The `compress-tools` crate aims to provide a convenient and easy to use set
of methods which builds on top of `libarchive` exposing a small set of its
functionalities.

| Platform | Build Status |
| -------- | ------------ |
| Linux - x86_64 | [![build status](https://github.com/OSSystems/compress-tools-rs/workflows/CI%20-%20Linux%20-%20x86_64/badge.svg)](https://github.com/OSSystems/compress-tools-rs/actions) |
| macOS - aarch64 | [![build status](https://github.com/OSSystems/compress-tools-rs/workflows/CI%20-%20macOS%20-%20aarch64/badge.svg)](https://github.com/OSSystems/compress-tools-rs/actions) |
| Windows - x86_64 | [![build status](https://github.com/OSSystems/compress-tools-rs/workflows/CI%20-%20Windows%20-%20x86_64/badge.svg)](https://github.com/OSSystems/compress-tools-rs/actions) |

---

## Dependencies

You must have `libarchive`, 3.2.0 or newer, properly installed on your
system in order to use this. If building on *nix and Windows GNU
systems, `pkg-config` is used to locate the `libarchive`; on Windows
MSVC, `vcpkg` will be used to locating the `libarchive`.

Typical install:

* Debian/Ubuntu: `apt install libarchive-dev pkg-config`
* macOS (Homebrew): `brew install libarchive pkg-config` (libarchive is
  keg-only; expose it via `PKG_CONFIG_PATH="$(brew --prefix libarchive)/lib/pkgconfig"`)
* Windows MSVC: `vcpkg install libarchive`

The minimum supported Rust version is 1.82.

## Install

```toml
[dependencies]
compress-tools = "0.16"
```

To enable async support backed by `tokio`:

```toml
[dependencies]
compress-tools = { version = "0.16", features = ["tokio_support"] }
```

See [Feature flags](#feature-flags) for the full list.

## Upgrading to 0.16

0.16.0 introduces a few breaking changes. See [CHANGES.md](CHANGES.md) for
the complete list; the highlights are:

- **Archive entry points now reject non-archive input.**
  `list_archive_files`, `list_archive_entries`, `uncompress_archive`,
  `uncompress_archive_file`, and `ArchiveIterator` (and their
  `_with_encoding` / async variants) no longer treat arbitrary byte
  streams as a single-entry archive named `data`. If you relied on that
  behavior with the iterator, opt back in with
  `ArchiveIteratorBuilder::raw_format(true)`. Raw compressed streams
  remain handled by `uncompress_data`.
- **Async entry points require `AsyncSeek`.** The async variants of
  `list_archive_files`, `uncompress_archive`, and `uncompress_archive_file`
  now bound the source on `AsyncRead + AsyncSeek`. Wrap `tokio::fs::File`
  via `tokio_util::compat` as needed.
- **`Error::Extraction` changed shape.** It now carries a `details`
  string and an optional `io::Error` reconstructed from `archive_errno`.
  Match arms using `Error::Extraction(msg)` should be rewritten as
  `Error::Extraction { details, .. }`.
- **New `Error::UnsupportedZipCompression` variant.** ZIP archives
  using Deflate64 (method 9) are now rejected up front instead of
  failing mid-extraction. Exhaustive `match` arms on `Error` need a
  new branch (or a catch-all).
- **MSRV raised to 1.82.0** (was 1.65.0). Older toolchains will fail
  to build.

## Features

This crate is capable of extracting:

* compressed files
* archive files
* a single file from an archive

### Extract an entire archive

```rust
use compress_tools::*;
use std::fs::File;
use std::path::Path;

let mut source = File::open("tree.tar.gz")?;
let dest = Path::new("/tmp/dest");

uncompress_archive(&mut source, &dest, Ownership::Preserve)?;
```

### Extract a single file from an archive

```rust
use compress_tools::uncompress_archive_file;
use std::fs::File;

let mut source = File::open("tree.tar.gz")?;
let mut target = File::create("/tmp/README.md")?;

uncompress_archive_file(&mut source, &mut target, "tree/README.md")?;
```

### Iterate over archive entries

```rust
use compress_tools::{ArchiveContents, ArchiveIteratorBuilder};
use std::fs::File;

let source = File::open("tree.tar.gz")?;
let iter = ArchiveIteratorBuilder::new(source).build()?;

for content in iter {
    match content {
        ArchiveContents::StartOfEntry(name, _stat) => println!("entry: {name}"),
        ArchiveContents::DataChunk(_bytes) => { /* stream the entry body */ }
        ArchiveContents::EndOfEntry => {}
        ArchiveContents::Err(e) => return Err(e.into()),
    }
}
```

### List entries with sizes

```rust
use compress_tools::list_archive_entries;
use std::fs::File;

let mut source = File::open("tree.tar")?;
for entry in list_archive_entries(&mut source)? {
    println!("{}: {} bytes", entry.path, entry.size);
}
```

### Asynchronous iteration (tokio)

Requires the `tokio_support` feature.

```rust
use compress_tools::tokio_support::ArchiveIteratorBuilder;
use compress_tools::ArchiveContents;
use futures_util::StreamExt;
use tokio::fs::File;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = File::open("tree.tar.gz").await?;
    let mut iter = ArchiveIteratorBuilder::new(source).build();

    while let Some(content) = iter.next().await {
        if let ArchiveContents::StartOfEntry(name, _stat) = content {
            println!("entry: {name}");
        }
    }
    Ok(())
}
```

### Password-protected ZIP archives

```rust
use compress_tools::{ArchiveIteratorBuilder, ArchivePassword};
use std::fs::File;

let source = File::open("secret.zip")?;
let iter = ArchiveIteratorBuilder::new(source)
    .with_password(ArchivePassword::new("<your password>")?)
    .build()?;

for _content in iter {
    // ...
}
```

## Feature flags

| Flag | Purpose |
| ---- | ------- |
| `async_support` | Base, executor-agnostic async primitives. |
| `futures_support` | `async_support` plus `blocking` integration for the `futures` ecosystem. |
| `tokio_support` | `async_support` plus `tokio` / `tokio-util` integration. |
| `static` | Statically link all bundled archive libraries and enable the default Windows imports. |
| `static_b2`, `static_lz4`, `static_zstd`, `static_lzma`, `static_bz2`, `static_z`, `static_xml2` | Selective static linking, one per bundled dependency. |
| `win_user32`, `win_crypt32`, `win_advapi32`, `win_xmllite` | Windows system import libraries (all enabled by default). |

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
