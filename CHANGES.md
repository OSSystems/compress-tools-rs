# CHANGES

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.12.2] - 2021-09-23

* Fix locale drop causing crash on a system without locale [#71]

[#71]: https://github.com/OSSystems/compress-tools-rs/issues/71

## [0.12.1] - 2021-09-03

## [0.12.0] - 2021-08-03

* Use "lossy" strings for invalid filenames. [#59]
* Fix zip-slip vulnerability. [#63]
* Fix memory leak when dropping locale guard. [#64]
* Add `ArchiveIterator` type. [#65]

[#59]: https://github.com/OSSystems/compress-tools-rs/issues/59
[#63]: https://github.com/OSSystems/compress-tools-rs/issues/63
[#64]: https://github.com/OSSystems/compress-tools-rs/issues/64
[#65]: https://github.com/OSSystems/compress-tools-rs/issues/65

## [0.11.2] - 2021-05-29

* Bump MSRV to 1.46. [#54]
* Install VcPkg/Pkg-Config depending on target env. [#56]
* Fix invalid display attribute causing build error [#58]

[#54]: https://github.com/OSSystems/compress-tools-rs/issues/54
[#56]: https://github.com/OSSystems/compress-tools-rs/pull/56
[#58]: https://github.com/OSSystems/compress-tools-rs/pull/58

## [0.11.1] - 2021-03-07

* Fix when uncompressing 7z archive to a directory. [#53]

[#53]: https://github.com/OSSystems/compress-tools-rs/issues/53

## [0.11.0] - 2021-03-03

### Fixed

* Fix unpacking of filenames with contains UTF-8 characters. [#52]
* Fixed the build script so it enforce the use of `libarchive` 3.2.0 or newer.

[#52]: https://github.com/OSSystems/compress-tools-rs/pull/52

## [0.10.0] - 2021-02-11

### Changed

* Update MSRV to 1.44.0.

### Fixed

* Fix error when uncompressing specific files from 7z archives. [#48]

[#48]: https://github.com/OSSystems/compress-tools-rs/pull/48

## [0.9.0] - 2020-12-25

* Upgrade `tokio` to 1.0.0 release.

## [0.8.0] - 2020-10-19

### Changed

* Upgrade `tokio` to 0.3.0 release.

## [0.7.1] - 2020-09-15

### Fixed

* Fix two memory leaks related to entry pathname and hardlink handling. [#33]
* Fix a memory leak found in the error handling code path. [#33]

[#33]: https://github.com/OSSystems/compress-tools-rs/pull/33

## [0.7.0] - 2020-09-05

### Added

* Optional async support
* Uncompress service example and its async-std and Tokio counterparts

### Removed

* Removed `Error::NullEntry` as it is unused.

### Changed

* Replaced `Error::FileNotFound` with `std::io::Error` using the
  `std::io::ErrorKind::NotFound`.

* Change error enum names to more meaninful ones. The following errors were
  renamed as:

  - `ExtractionError` to `Extraction`
  - `ArchiveNull` to `NullArchive`
  - `EntryNull` to `NullEntry`

* Change MSRV to 1.42.0

## [0.6.0] - 2020-06-28

### Added

* `list_archive_file` allow for getting the list of files included in an archive. [#22]

### Changed

* Change MSRV to 1.40.0

[#22]: https://github.com/OSSystems/compress-tools-rs/issues/22

## [0.5.1] - 2020-05-12

### Changed

* Lower required version of libarchive to 3 instead of 3.2.2 [#21]

[#21]: https://github.com/OSSystems/compress-tools-rs/pull/21

## [0.5.0] - 2020-04-30

### Added

* Support for windows build through `vcpkg` [#19]

[#19]: https://github.com/OSSystems/compress-tools-rs/pull/19

## [0.4.0] - 2020-04-17

### Added

* `uncompress_data` (previously `uncompress_file`) and `uncompress_archive_file`, on success, now return the ammount of bytes they have uncompressed [#16]

[#16]: https://github.com/OSSystems/compress-tools-rs/pull/16

### Changed

* More generic read/write api (should not be a breaking change) [#14]
  * `Read` and `Write` arguments are no longer required to be a mutable reference,
    which allows for more tyes to be used, as `&mut [u8]`

* Renamed `uncompress_file` function to `uncompress_data` [#17]

[#14]: https://github.com/OSSystems/compress-tools-rs/pull/14
[#17]: https://github.com/OSSystems/compress-tools-rs/pull/17

## [0.3.1] - 2020-04-14

### Fixed

* Fixed outdated README

## [0.3.0] - 2020-04-14

### Added

* Add crate level error type [#4]

### Changed

* API fully Reworked [#6]

* Archive and uncompression is now handled with ffi calls to libarchive [#6]

* Improved documentation, tests and examples

[#4]: https://github.com/OSSystems/compress-tools-rs/pull/4
[#6]: https://github.com/OSSystems/compress-tools-rs/pull/6

## [0.2.0] - 2019-04-29

### Added

* Add support for Zip compressed archives [#3]

[#3]: https://github.com/OSSystems/compress-tools-rs/pull/3

## [0.1.2] - 2019-04-29

### Changed

* Add flags to tar command to perserve file permissions

* Use BusyBox compatible commands for uncompression

## [0.1.0] - 2019-04-25

* First release
