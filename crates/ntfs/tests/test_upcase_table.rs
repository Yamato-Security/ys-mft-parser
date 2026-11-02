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

use ntfs::image::AffImage;
use ntfs::ntfs::name::UpcaseTable;
use ntfs::ntfs::name::upcase::UPCASE_TABLE_SIZE_BYTES;
use ntfs::ntfs::{FileSystem, Volume};
use std::sync::Arc;

mod common;

#[test]
fn test_read_upcase_table_from_fixture() {
    let path = common::ntfs_fixture_path("ntfs1-gen0.aff");
    if !common::ensure_fixture(&path) {
        return;
    }
    let img = AffImage::open(path).unwrap();
    let volume = Volume::open(Arc::new(img), 0).unwrap();
    let fs = FileSystem::new(volume);

    let raw = fs.read_file_default_stream(10).unwrap();
    assert_eq!(raw.len(), UPCASE_TABLE_SIZE_BYTES);

    let table = UpcaseTable::from_bytes(&raw).unwrap();
    assert_eq!(table.map_u16(b'a' as u16), b'A' as u16);
    assert_eq!(table.map_u16(b'Z' as u16), b'Z' as u16);
}
