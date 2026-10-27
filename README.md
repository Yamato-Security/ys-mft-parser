# ys-mft-parser

Yamato Security's maintained fork of Omer Ben-Amram's [mft](https://github.com/omerbenamram/mft)
crate, the Windows Master File Table parser used by
[Juntei](https://github.com/Yamato-Security/juntei). The complete upstream history and the
MIT / Apache-2.0 license are retained; everything below this section is upstream's README.

The fork applies upstream pull requests that were waiting for review and fixes found while building
Juntei, so that damaged or crafted `$MFT` files are parsed instead of crashing the process:

| Change | Source |
| --- | --- |
| A parent-reference loop (or a chain of thousands of directories) no longer overflows the stack | fork (replaces upstream #117) |
| An all-zero or garbage first header no longer divides by zero or sizes every read from it; the entry size comes from the first valid header (wiped first entries are recovered) | upstream #119 (Joel Uckelman) + bounds |
| Full paths use `/` on every OS (`get_full_path_for_entry` returns a `String`) | upstream #120 (Mason Bartle) |
| A file whose parent directory's record was reused is `[Orphaned]` instead of getting the new owner's path | upstream #150 (mcardonaes) |
| A deleted file in a deleted directory keeps its path (the parent's sequence number is one ahead) | fork |
| Data runs after a sparse run point at the right clusters (sparse and compressed files) | fork |
| Malformed data runs, huge entry numbers and `FlatMftEntryWithName` no longer panic | fork |
| The `ntfs` crate and the explorer GUI compile again (broken upstream since the UTF-16 refactor) | fork |
| `mft_dump -e` names resident stream files deterministically and sanitizes the stream name (an alternate data stream named `../../x` was written outside the output directory) | fork |
| The `log` crate's `release_max_level_debug` is enabled only for `mft_dump` | upstream #149 (Lej77) |

Use it from Cargo by commit:

```toml
mft = { git = "https://github.com/Yamato-Security/ys-mft-parser", rev = "<commit>", default-features = false }
```

The crate keeps the name `mft` and is not published to crates.io; its versions start at 0.8.0 and
are the fork's own. See [CHANGELOG.md](CHANGELOG.md).

---

[![Build Status](https://dev.azure.com/benamram/DFIR/_apis/build/status/omerbenamram.mft?branchName=master)](https://dev.azure.com/benamram/DFIR/_build/latest?definitionId=5&branchName=master)
![crates.io](https://img.shields.io/crates/v/mft.svg)

# MFT

This is a parser for the MFT (master file table) format.

MSRV is latest stable rust.

[Documentation](https://docs.rs/mft)

Python bindings are available as well at https://github.com/omerbenamram/pymft-rs (and at PyPi https://pypi.org/project/mft/)

## Features
 - Implemented using 100% safe rust - and works on all platforms supported by rust (that have stdlib).
 - Supports JSON and CSV outputs.
 - Supports extracting resident data streams.

## Installation (associated binary utility):
  - Download latest executable release from https://github.com/omerbenamram/mft/releases
    - Releases are automatically built for for Windows, macOS, and Linux. (64-bit executables only)
  - Build from sources using  `cargo install mft`

# `mft_dump` (Binary utility):
The main binary utility provided with this crate is `mft_dump`, and it provides a quick way to convert mft snapshots to different output formats.

Some examples
  - `mft_dump <input_file>` will dump contents of mft entries as JSON.
  - `mft_dump -o csv <input_file>` will dump contents of mft entries as CSV.
  - `mft_dump --extract-resident-streams <output_directory> -o json <input_file>` will extract all resident streams in MFT to files in <output_directory>.

# Library usage:
```rust,no_run
use mft::MftParser;
use mft::attribute::MftAttributeContent;
use std::path::PathBuf;

fn main() {
    // Change this to a path of your MFT sample.
    let fp = PathBuf::from(format!("{}/samples/MFT", std::env::var("CARGO_MANIFEST_DIR").unwrap()));

    let mut parser = MftParser::from_path(fp).unwrap();
    for entry in parser.iter_entries() {
        match entry {
            Ok(e) =>  {
                for attribute in e.iter_attributes().filter_map(|attr| attr.ok()) {
                    match attribute.data {
                        MftAttributeContent::AttrX10(standard_info) => {
                            println!("\tX10 attribute: {:#?}", standard_info)
                        },
                        MftAttributeContent::AttrX30(filename_attribute) => {
                            println!("\tX30 attribute: {:#?}", filename_attribute)
                        },
                        _ => {
                            println!("\tSome other attribute: {:#?}", attribute)
                        }
                    }

                }
            }
            Err(err) => eprintln!("{}", err),
        }
    }
}
```

## Performance profiling (samply)

The repo ships with a small sample MFT (`samples/MFT`, ~13MB) which makes a good fixed workload.

### Baseline timings (hyperfine)

```bash
cargo build --release --bin mft_dump

# End-to-end CLI throughput (write output to /dev/null to avoid terminal overhead).
hyperfine --warmup 3 --runs 20 \
  './target/release/mft_dump samples/MFT -o jsonl -f /dev/null --no-confirm-overwrite' \
  './target/release/mft_dump samples/MFT -o csv -f /dev/null --no-confirm-overwrite'
```

### CPU profiling (samply)

```bash
cargo build --release --bin mft_dump
mkdir -p target/samply

# End-to-end (parsing + serialization) profile.
samply record --save-only --unstable-presymbolicate \
  -o target/samply/mft_dump_jsonl.profile.json.gz \
  --iteration-count 200 -- \
  ./target/release/mft_dump samples/MFT -o jsonl -f /dev/null --no-confirm-overwrite

# Parser-only profile (no serialization/output), long single-process run.
# View in the Firefox Profiler UI.
samply load target/samply/mft_dump_jsonl.profile.json.gz
```

In the Firefox Profiler UI:
- Use the **Call Tree** tab with **Invert call stack** to identify top leaf frames.
- Keep **Invert call stack** off to see inclusive hot functions (top-down).
- Use **Filter stack** to focus on crate frames (for example, `mft::` or `mft_dump::`).

## Thanks/Resources:
 - https://docs.microsoft.com/en-us/windows/desktop/DevNotes/master-file-table
 - https://github.com/libyal/libfsntfs/blob/master/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc
 - https://github.com/forensicmatt/RustyMft
