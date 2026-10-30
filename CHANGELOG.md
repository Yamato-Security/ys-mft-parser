# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - Unreleased (ys-mft-parser)

Includes upstream 0.8.0 (ed20ab7, everything in its section below). Where upstream now fixes the
same problem as this fork, upstream's code is used.

### Added
- `i30`: directory indexes (`$I30`): `index_records` reads the INDX records of an `$INDEX_ALLOCATION:$I30` stream (or any data holding them: they are looked for at every 512-byte boundary), with their update sequence arrays applied (a record written in part is read as it is and said so; one saved with the array already applied is taken as it is), their entries in use (each file's `$FILE_NAME`: name, namespace, parent, times, sizes, flags) and the entries left in their slack by files deleted or renamed: whole entries, `$FILE_NAME`s whose entry header was written over, and one whose parent the end of the entries in use wrote over. What is taken from slack must look like what Windows writes; copies of entries in use are marked. (@YamatoSecurity)

### Changed
- The crate is named after its repository, `ys-mft-parser` (the library is still `mft`: depend on
  it as `mft = { package = "ys-mft-parser", … }`), version 1.0.0, by Omer Ben-Amram and Zach
  Mathis (@yamatosecurity), and requires Rust 1.99. (@YamatoSecurity)
- The command is named after the repository: `ys-mft-parser`, not `mft_dump` (and so is the
  feature that builds it, `ys-mft-parser`, on by default). The entries below keep the old name.
- `MftParser::get_full_path_for_entry` returns `Option<PathBuf>` and `FlatMftEntryWithName::full_path`
  is a `PathBuf` again; CSV and JSON write paths with `\` on every OS, as upstream does.
  This replaces the fork's port of upstream #120 (`/` everywhere), which upstream closed.
- A cycle of parent references is cut at its lowest entry and becomes `[Orphaned]/...` (upstream's
  resolver); it was `[Unknown]/...`. Chains of any depth resolve without recursion.
- Paths are cached per entry and sequence number, so the stale-parent check (upstream #150) also
  applies to names read from extension records.
- `mft_dump -e` keeps the fork's file names, and takes upstream's export of streams stored in
  extension records and its skipping of empty streams.
- The version is 0.9.0, and the root crate doesn't depend on `rand`.

### Fixed
- A file whose parent directory was deleted and its base record reused is placed under the
  parent's old name when the parent left an extension record behind that still points at its old
  version (Harel Segev's "orphaned attributes"); it was `[Orphaned]`.
- A data run that lands before LCN 0 is rejected instead of wrapping to a huge cluster number,
  and the first run's offset is read as signed like the others (upstream reads it unsigned, so
  `11 01 ff` was LCN 255 instead of a damaged -1).
- The `mft_dump -e` help still described random bytes in the file names.

### Tests
- `tests/test_path_pitfalls.rs`: an MFT built for each pitfall of resolving paths (reused and
  deleted parents, short names and hard links, names only in extension records behind a resident
  or non-resident `$ATTRIBUTE_LIST`, files without a `$FILE_NAME`, leftover extension records,
  cycles, unusable parents). The README defines `[Orphaned]` and `[Unknown]`.

### Kept from the fork
- The 4 MiB entry-size probe with the update sequence check: upstream's 64 KiB probe can't open
  dfir_ntfs's `ntfs_frag_mft.bin`, which this fork reads (3,007 entries).
- The stale-parent check (upstream #150 with the deleted-parent correction), the error for
  overflowing entry offsets, `FlatMftEntryWithName::from_entry` without `expect()`, and the
  `mft_dump -e` path traversal fix.

### CI
- Upstream's lint job (clippy on every crate except the GUI) and MSRV job, with actions pinned by
  commit.
- Every crate's tests run on Linux, macOS and Windows with upstream's disk images:
  `scripts/fetch_lfs_testdata.py` downloads the LFS objects that `testdata/` points to from
  upstream's LFS store, checks each one's size and SHA-256 against its pointer, and CI caches them
  until the pointers change. A missing image fails the run (`NTFS_TESTDATA_REQUIRED`), and the
  AFFLIBv3 fixtures are fetched as upstream's CI does.
- The explorer GUI is built, linted and tested with GTK and WebKitGTK installed.

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

## [0.8.0] - 2026-10-27

### Added
- CSV output has `StandardInfoLastMftModified` and `FileNameLastMftModified` columns (the "MFT entry changed" times), appended after `FullPath` so existing column positions don't move (#123).
- `windows_path`, `serialize_windows_path` and `serialize_option_windows_path` format a resolved path with `\` separators.
- `MftEntry::iter_attribute_records_matching` yields each attribute's header and raw record without parsing its content, and `ResidentHeader::value` slices a resident attribute's value out of its record.
- Parse `$VOLUME_NAME` (0x60) and `$VOLUME_INFORMATION` (0x70) into `VolumeNameAttr` and `VolumeInformationAttr` (label, NTFS version, volume flags) instead of raw hex.
- `mft_dump -o bodyfile` writes Sleuth Kit bodyfile lines in the `fls -m` layout, for timelining with `mactime`: one line per `$DATA` stream and `$INDEX_ROOT` with the `$STANDARD_INFORMATION` times, and a `($FILE_NAME)` line with the `$FILE_NAME` times.
- Parse `$REPARSE_POINT` (0xC0) into `ReparsePointAttr`: the tag, its documented name, the GUID for third-party tags, and decoded symlink, junction/mount point, AppExecLink and WSL symlink targets. Other tags keep their data as hex.
- `mft_dump` JSON and JSONL output include each entry's `full_path` (previously only CSV had it).
- `FlatMftEntryWithName::from_entry_with_path` builds a CSV row from an already-resolved path and the entry's extension records.
- `MftParser::get_extension_entries` reads the extension records that hold attributes an `$ATTRIBUTE_LIST` moved out of a base record, `MftEntry::has_attribute_list` says whether there are any, and `entry::find_best_name_attribute_in` picks a file's name across its base and extension records (#148).
- JSON output lists attributes that failed to parse in a per-entry `errors` array (only present when non-empty). Previously they were dropped silently.
- Add `MftAttributeType::PropertySet` (`0xF0`, NT4-era `$PROPERTY_SET`).
- JSON output includes `data_size` for resident attributes (#125).

### Changed
- **Breaking:** attributes borrow from the entry's record buffer instead of copying it. `MftAttribute`, `MftAttributeContent` and the name-carrying attribute types (`FileNameAttr`, `DataAttr`, `AttributeListAttr`, `IndexRootAttr`, ...) take a lifetime, names are borrowed UTF-16LE that decode on demand, and the `into_*` accessors are replaced by borrowing `as_*` ones (`as_file_name`, `as_data_runs`, ...). This makes parsing much faster but needs callers to keep the `MftEntry` alive while they use its attributes.
- Faster `mft_dump`: release builds use thin LTO with one codegen unit and native release binaries are profile-guided (PGO), timestamps and flags are formatted without `core::fmt`, directory paths are cached, and I/O buffers are larger (see PERF.md).
- CSV and JSON output write `full_path` with `\` separators on every platform, the way Windows shows NTFS paths. Linux and macOS builds used `/`. `get_full_path_for_entry` still returns a `PathBuf`, and bodyfile output keeps `fls`'s `/`.
- JSON output of `$VOLUME_NAME` and `$VOLUME_INFORMATION` attributes changes from `{attribute_type, data}` raw hex to the parsed fields (a `$VOLUME_INFORMATION` value too short to parse stays raw hex); `MftAttributeContent` gains the `AttrX60` and `AttrX70` variants, which breaks exhaustive matches.
- `MftAttributeContent` has a new `AttrXC0` variant, and `$REPARSE_POINT` JSON output changes from raw hex to the parsed structure.
- Timestamps keep NTFS's full 100ns resolution and are always written with 7 fractional digits (`2007-06-30T12:50:52.2523952Z`). They were truncated to microseconds and printed with 0, 3, 6 or 9 digits, chrono style. This changes every timestamp in JSON and CSV output. `serialize_timestamp_chrono_compat` and `serialize_option_timestamp_chrono_compat` are renamed to `serialize_timestamp` and `serialize_option_timestamp`.
- `FileNameAttr::logical_size` now holds the real size and `physical_size` the allocated size (see Fixed). Code that worked around the old swap needs to drop the workaround.
- Update all workspace dependencies to their latest releases, including `rand` 0.10, `lru` 0.19,
  `itertools` 0.15, `num-derive` 0.5, `criterion` 0.8, `quick-xml` 0.42, RustCrypto `md-5`/`sha1` 0.11
  and `aes`/`des` 0.9, and `fuser` 0.18 (the FUSE frontend now uses fuser's `&self` callbacks).

### Fixed
- Full paths use the long name of a file or directory whose base record only has room for its 8.3 short name, by looking in its extension records (#148). Paths like `Windows/servicing/LCU/PACKAG~1.7/...` become `Windows/servicing/LCU/Package_for_RollupFix~.../...`; on a Windows 11 sample, about a third of all paths change this way.
- CSV `FileSize`, `HasAlternateDataStreams` and the bodyfile's stream lines include `$DATA` attributes stored in extension records, instead of reporting such files as empty.
- `$FILE_NAME` `logical_size` and `physical_size` were swapped: the allocated size (offset 0x28) was reported as the logical size and the real size (0x30) as the physical size. JSON output and index entries change accordingly.
- `iter_attributes` keeps returning later attributes after one with an unknown type code, and
  `iter_attributes_matching` skips unrequested attributes without parsing their headers.
- Resolve the LCN of a data run that follows a sparse run relative to the last allocated run (#152).
- Serialize flags as `A | B` again instead of bitflags 2's `TypeName(A | B)` debug format.
- Break cycles in parent references instead of overflowing the stack while resolving paths.
- Recover the entry size from later records when the first entry is zeroed, instead of dividing by zero.
- `mft_dump -e` no longer writes empty files for zero-length resident streams (#23).
- The `ntfs` crate, its tools and the NTFS explorer GUI build again against the borrowed attribute API.

### CI
- Push triggers name `master` (they said `main`, so pushes to master didn't run CI).
- Lint and test every workspace crate (previously only the root `mft` crate), build the GUI, and check
  the 1.90 MSRV.

## [0.7.0] - 2025-12-23

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
