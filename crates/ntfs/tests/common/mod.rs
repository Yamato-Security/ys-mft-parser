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

#![allow(dead_code)]

use std::env;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

pub fn ntfs_fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/ntfs")
}

pub fn ntfs_fixture_path(name: &str) -> PathBuf {
    ntfs_fixture_root().join(name)
}

pub fn undelete7_fixture_path(name: &str) -> PathBuf {
    ntfs_fixture_root().join("7-undel-ntfs").join(name)
}

fn is_git_lfs_pointer(path: &Path) -> bool {
    let Ok(mut f) = File::open(path) else {
        return false;
    };

    let mut buf = [0u8; 200];
    let Ok(n) = f.read(&mut buf) else {
        return false;
    };

    let Ok(s) = std::str::from_utf8(&buf[..n]) else {
        return false;
    };

    s.starts_with("version https://git-lfs.github.com/spec/v1")
}

fn fixture_missing_behavior(msg: &str) -> bool {
    if env::var_os("NTFS_TESTDATA_REQUIRED").is_some() {
        panic!("{msg}");
    }

    eprintln!("{msg}");
    false
}

/// Returns `true` iff `path` exists and is not a Git LFS pointer file.
///
/// If `NTFS_TESTDATA_REQUIRED=1` is set, missing/placeholder fixtures will **panic** (useful for CI).
/// Otherwise, the caller should `return` early from the test to skip it.
pub fn ensure_fixture(path: &Path) -> bool {
    if !path.exists() {
        return fixture_missing_behavior(&format!(
            "skipping: missing test fixture `{}` (did you forget to fetch testdata?)",
            path.display()
        ));
    }

    if is_git_lfs_pointer(path) {
        return fixture_missing_behavior(&format!(
            "skipping: `{}` looks like a Git LFS pointer file; run `git lfs install && git lfs pull`",
            path.display()
        ));
    }

    true
}
