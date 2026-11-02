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

use ewf::EwfReader;

mod common;

fn assert_ntfs_boot_sector(img: &EwfReader) {
    let mut boot = [0u8; 512];
    img.read_exact_at(0, &mut boot).unwrap();
    assert_eq!(&boot[3..11], b"NTFS    ");
    assert_eq!(&boot[510..512], &[0x55, 0xAA]);
}

#[test]
fn test_ewf_ntfs1_gen0_reads_boot_sector() {
    let path = common::ntfs_fixture_path("ntfs1-gen0.E01");
    if !common::ensure_fixture(&path) {
        return;
    }
    let img = EwfReader::open(&path).unwrap();
    assert_ntfs_boot_sector(&img);
}

#[test]
fn test_ewf_ntfs1_gen1_reads_boot_sector() {
    let path = common::ntfs_fixture_path("ntfs1-gen1.E01");
    if !common::ensure_fixture(&path) {
        return;
    }
    let img = EwfReader::open(&path).unwrap();
    assert_ntfs_boot_sector(&img);
}

#[test]
fn test_ewf_ntfs1_gen2_reads_boot_sector() {
    let path = common::ntfs_fixture_path("ntfs1-gen2.E01");
    if !common::ensure_fixture(&path) {
        return;
    }
    let img = EwfReader::open(&path).unwrap();
    assert_ntfs_boot_sector(&img);
}
