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

use ntfs::image::{EwfImage, RawImage};
use ntfs::ntfs::{FileSystem, Volume};
use std::sync::Arc;

mod common;

#[test]
fn test_filesystem_is_entry_allocated_matches_deleted_fixture_7() {
    let img_path = common::undelete7_fixture_path("7-ntfs-undel.dd");
    if !common::ensure_fixture(&img_path) {
        return;
    }
    let img = RawImage::open(img_path).unwrap();
    let volume = Volume::open(Arc::new(img), 0).unwrap();
    let fs = FileSystem::new(volume);

    // Root directory is allocated.
    assert!(fs.is_entry_allocated(5).unwrap());

    // Known deleted entries from DFTT image #7.
    for id in [29_u64, 30, 31, 32, 35, 36, 37, 38] {
        assert!(
            !fs.is_entry_allocated(id).unwrap(),
            "expected entry {id} to be not allocated (deleted)"
        );
    }
}

#[test]
fn test_filesystem_is_entry_efs_encrypted_detects_encrypted_fixture_gen2() {
    let img_path = common::ntfs_fixture_path("ntfs1-gen2.E01");
    if !common::ensure_fixture(&img_path) {
        return;
    }
    let img = EwfImage::open(img_path).unwrap();
    let volume = Volume::open(Arc::new(img), 0).unwrap();
    let fs = FileSystem::new(volume);

    let encrypted_id = fs.resolve_path("\\Encrypted\\logfile1.txt").unwrap();
    let raw_id = fs.resolve_path("\\Raw\\logfile1.txt").unwrap();

    assert!(fs.is_entry_efs_encrypted(encrypted_id).unwrap());
    assert!(!fs.is_entry_efs_encrypted(raw_id).unwrap());
}

#[test]
fn test_export_file_default_stream_to_path_matches_read_bytes() {
    let img_path = common::undelete7_fixture_path("7-ntfs-undel.dd");
    if !common::ensure_fixture(&img_path) {
        return;
    }
    let img = RawImage::open(img_path).unwrap();
    let volume = Volume::open(Arc::new(img), 0).unwrap();
    let fs = FileSystem::new(volume);

    // One resident + one non-resident case.
    for id in [37_u64, 32] {
        let expected = fs.read_file_default_stream(id).unwrap();

        let dir = tempfile::tempdir().unwrap();
        let out = dir.path().join(format!("entry-{id}.bin"));

        fs.export_file_default_stream_to_path(id, &out).unwrap();

        let got = std::fs::read(&out).unwrap();
        assert_eq!(got, expected, "export mismatch for entry {id}");
    }
}
