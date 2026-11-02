// Copyright (C) 2026 Shirofune Security Co., Ltd.
// Copyright (c) 2019 Omer Ben-Amram
//
// This file is changed from omerbenamram/mft (https://github.com/omerbenamram/mft), whose code is
// under the MIT License or the Apache License 2.0 (LICENSE-MIT, LICENSE-APACHE).
//
// Unless explicitly acquired and licensed from Licensor under another license, the contents of this
// file are subject to the Reciprocal Public License ("RPL") Version 1.5, or subsequent versions as
// allowed by the RPL, and You may not copy or use this file in either source code or executable
// form, except in compliance with the terms and conditions of the RPL.
//
// All software distributed under the RPL is provided strictly on an "AS IS" basis, WITHOUT WARRANTY
// OF ANY KIND, EITHER EXPRESS OR IMPLIED, AND LICENSOR HEREBY DISCLAIMS ALL SUCH WARRANTIES,
// INCLUDING WITHOUT LIMITATION, ANY WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
// PURPOSE, QUIET ENJOYMENT, OR NON-INFRINGEMENT. See the RPL for specific language governing rights
// and limitations under the RPL.

//! Snapshot tests for the serialized output of selected sample entries.
//!
//! These guard the JSON and CSV output formats against accidental changes. When a change to the
//! output is intentional, review and accept the new snapshots with `cargo insta review`.
mod fixtures;

use fixtures::*;
use mft::mft::MftParser;
use std::collections::BTreeSet;

/// Entries chosen to cover different record shapes in `samples/MFT`.
const JSON_ENTRIES: &[(u64, &str)] = &[
    (0, "mft"),
    (3, "volume"),
    (5, "root_directory"),
    (9, "secure"),
    (462, "object_id"),
    (3498, "attribute_list"),
    (8671, "alternate_data_stream"),
    (11600, "deleted"),
];

#[test]
fn test_json_output_snapshots() {
    ensure_env_logger_initialized();
    let mut parser = MftParser::from_path(mft_sample()).unwrap();

    for &(entry_id, name) in JSON_ENTRIES {
        let entry = parser.get_entry(entry_id).unwrap();
        let json = serde_json::to_string_pretty(&entry).unwrap();
        insta::assert_snapshot!(format!("json_entry_{entry_id}_{name}"), json);
    }
}

/// Formats an entry list as a `ys-mft-parser --ranges` argument.
fn ranges_arg(ids: impl IntoIterator<Item = u64>) -> String {
    ids.into_iter()
        .map(|id| id.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

/// `ys-mft-parser` uses a different JSON serializer for JSONL than the library's serde_json output, so
/// snapshot the CLI output too. CSV covers the first 40 records plus the JSON entries.
#[test]
#[cfg(feature = "ys-mft-parser")]
fn test_cli_output_snapshots() {
    use assert_cmd::cargo::cargo_bin_cmd;

    let json_ids = || JSON_ENTRIES.iter().map(|&(id, _)| id);
    let csv_ids: BTreeSet<u64> = (0..40).chain(json_ids()).collect();

    // Both formats carry full paths, written with `\` on every platform.
    for (format, ids) in [
        ("jsonl", ranges_arg(json_ids())),
        ("csv", ranges_arg(csv_ids)),
    ] {
        let mut cmd = cargo_bin_cmd!("ys-mft-parser");
        cmd.arg("-o")
            .arg(format)
            .arg("-r")
            .arg(ids)
            .arg(mft_sample());
        let output = cmd.output().unwrap();
        assert!(
            output.status.success(),
            "ys-mft-parser failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8(output.stdout).unwrap();
        insta::assert_snapshot!(format!("cli_{format}_selected_entries"), stdout);
    }
}
