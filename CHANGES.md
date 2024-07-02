# CHANGES

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.15.0] - 2024-07-02

* Raise MSRV to 1.65.0
* Add next_header() to ArchiveIterator [#122]
* Fix use slice::from_raw_parts only if size > 0 [#126]
* Add feature "static" to allow static linkage for unix/macos [#127]
[#122]: https://github.com/OSSystems/compress-tools-rs/pull/122
[#126]: https://github.com/OSSystems/compress-tools-rs/pull/126
[#127]: https://github.com/OSSystems/compress-tools-rs/pull/127

## [0.14.3] - 2023-05-26

* Allow passing a closure for `ArchiveIterator::filter` [#115]

[#115]: https://github.com/OSSystems/compress-tools-rs/pull/115

## [0.14.2] - 2023-05-23

* Fix call stack runtime error on filter from `ArchiveIterator` [#113]

[#113]: https://github.com/OSSystems/compress-tools-rs/pull/113

## [0.14.1] - 2023-03-21

* Add illumos compilation support [#99]
* Fix segmentation when failing to decode entry [#100]
* Wrap return value of `archive_write_data_block(3)` [#108]
* Allow to filter ArchiveIterator entries [#109]
* Add debug asserts to ArchiveIterator::next() [#110]
* examples: add example using ArchiveIterator [#111]
* tests: port from unmaintained crate encoding to encoding_rs [#112]

[#99]: https://github.com/OSSystems/compress-tools-rs/pull/99
[#100]: https://github.com/OSSystems/compress-tools-rs/issues/100
[#108]: https://github.com/OSSystems/compress-tools-rs/pull/108
[#109]: https://github.com/OSSystems/compress-tools-rs/pull/109
[#110]: https://github.com/OSSystems/compress-tools-rs/pull/110
[#111]: https://github.com/OSSystems/compress-tools-rs/pull/111
[#112]: https://github.com/OSSystems/compress-tools-rs/pull/112

## [0.14.0] - 2022-11-20

* Raise MSRV to 1.59.0
* Change to 2021 edition
* Drop lifetime annotations of reader parameter in `ArchiveIterator::from_read`
  and `ArchiveIterator::from_read_with_encoding` [#90]
* Forward name decode failures in `ArchiveIterator::from_read` and
  `ArchiveIterator::from_read_with_encoding` instead of panicking [#91]
* Increase internal used buffersize [#93], fixing sub-directories as file
  names. [#89]

[#89]: https://github.com/OSSystems/compress-tools-rs/issues/89
[#90]: https://github.com/OSSystems/compress-tools-rs/pull/90
[#91]: https://github.com/OSSystems/compress-tools-rs/pull/91
[#93]: https://github.com/OSSystems/compress-tools-rs/pull/93

## [0.13.0] - 2022-08-03

* Add `libc::stat` information to `ArchiveContents::StartOfEntry` [#88]

[#88]: https://github.com/OSSystems/compress-tools-rs/pull/88

## [0.12.4] - 2022-08-01

* Avoid failing uncompressing files in case of ARCHIVE_WARN returns [#85]
* Add `_with_encoding` suffix method. [#59]

[#59]: https://github.com/OSSystems/compress-tools-rs/pull/59
[#85]: https://github.com/OSSystems/compress-tools-rs/issues/85

## [0.12.3] - 2022-06-22

* ci: windows: Use pre-installed vcpkg and fix build [#81]
* Raise MSRV to 1.49.0
* Upgrade tokio-util to 0.7.0
* Fix absolute paths being extracted outside of destination directory [#83]

[#81]: https://github.com/OSSystems/compress-tools-rs/issues/81
[#83]: https://github.com/OSSystems/compress-tools-rs/issues/83

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
