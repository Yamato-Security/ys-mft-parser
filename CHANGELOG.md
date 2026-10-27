# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.8.0] - 2026-10-27 (ys-mft-parser)

First version of Yamato Security's fork, based on upstream `master` 18b6c05 (after 0.7.0).

### Changed
- `MftParser::get_full_path_for_entry` returns `Option<String>` and `FlatMftEntryWithName::full_path`
  is a `String`: paths are joined with `/` on every OS (upstream #120, Mason Bartle).
- `mft_dump --extract-resident-streams` names files `<path>__<entry>-<sequence>_<n>_<stream>.dontrun`
  (no random part), sanitizes the stream name and the characters Windows does not allow, shortens
  only the file name, and does not overwrite an existing file.
- The entry size comes from the first valid FILE header (upstream #119, Joel Uckelman), accepted
  only as a power of two from 512 bytes to 64 KiB within the first 4 MiB.
- `log`'s `release_max_level_debug` is a `mft_dump` feature only (upstream #149, Lej77).
- The `rand` dependency is gone; the crate is not published (`publish = false`).

### Fixed
- Two directories naming each other as parent (or a chain deeper than 256) overflowed the stack;
  such paths now start with `[Unknown]` (instead of upstream #117's API change).
- An all-zero first header divided by zero in `get_entry_count`.
- A parent directory whose record was reused gave its children the new owner's path; they are
  `[Orphaned]` (upstream #150, mcardonaes). A deleted directory, whose sequence number NTFS raised
  when freeing it, is still the parent of its deleted files.
- Data runs after a sparse run were placed relative to the sparse run (LCN 0), so the clusters of
  sparse and compressed files after the first gap were wrong.
- A data run without a length, an LCN out of range, and an entry number whose offset overflows
  panicked (debug builds) or wrapped; they are errors.
- `FlatMftEntryWithName::from_entry` used `expect()` for the signature and the path.
- The `ntfs` crate (with `ntfs-info`) and `ntfs-explorer-gui` did not compile since upstream's
  borrowed UTF-16 refactor (#147); CI now also checks the workspace crates (except the GUI).
- The `ntfs` crate's `fuse` feature did not compile (a derived `Debug` over `dyn ReadAt`).

### Tests
- Damaged-input tests: seeded random damage and every byte of real entries at 0x00 / 0xFF.
- `test_runs` expected a run after a sparse run at the bare offset; it is relative to the last LCN.

### CI
- The NTFS / EWF / AFF disk images are upstream's Git LFS objects and are not stored in this fork;
  CI no longer requires them (the tests of those crates skip without them).
- `actions/checkout` is pinned to a commit.

## [Unreleased]

### Added
- Expand `FileAttributeFlags` with the latest Windows-defined bits for NTFS parsing.

### Changed
- Upgrade the crate to the Rust 2024 edition and require Rust 1.90 for builds.
- Forbid usage of `unsafe` code across the workspace and satisfy new Clippy lints.
- Keep formatting and lint configuration aligned with Rust 2024 expectations.

### Fixed
- Correct decoding of sparse non-resident data runs when reading MFT entries.
- Address the `clippy::collapsible_if` warning emitted for the `mft_dump` CLI.

### CI
- Refresh CI workflows, add Apple Silicon release artifacts, and scope push triggers to `main` and `release/**`.

## [0.6.1] - 2023-02-18

### Changed
- Dependency updates

## [0.6.0] - 2022-01-05

### Changed
- Happy new year! lots of dependencies have been updated.
- update to rust edition 2021

## [0.5.2] - 2020-03-10

### Fixed
- Attribute list parsing

### Changed
- Warn on fixup mismatch instead of fail

### Added
- Additional File Attribute flags
- `valid_fixup` field to MftEntry

## [0.5.1] - 2020-02-06

### Fixed
- Added support for additional MFT attributes (parsed as raw streams)

## [0.5.0] - 2020-01-06

### Changed
- Bumped dependencies.
- `mft_dump` is now an optional features so consumers of the library can enjoy faster compilation time.
- Changed error handling to `anyhow` + `thiserror` from `snafu`.

## [0.4.4] - 2019-06-19

### Fixed
- Fixed a bug where `HasAlternateDataStreams` will miss entries with only a single named stream.

## [0.4.3] - 2019-06-06

### Added
- `mft_dump` can now dump only a specific range of entries with `-r`.
- CSV output now shows `FileSize`, `IsDeleted` as separate columns.

### Fixed
- Fixed an issue with debug-logs

## [0.4.2] - 2019-06-04

### Fixed
- Ignore zeroed entries in `mft_dump`

## [0.4.1] - 2019-06-04

### Fixed
- Files which are not an MFT now cause the program to terminate with a nice message.
- Nicely ignore zeroed files in `mft_dump`

## [0.4.0] - 2019-06-04

### Added
- `mft_dump` can now export resident streams to a directory.

## [0.3.0] - 2019-06-02

Fixed parsing of entries which are spread over multiple sectors.

## [0.2.0] - 2019-05-23

Initial release which I consider a usable beta.
