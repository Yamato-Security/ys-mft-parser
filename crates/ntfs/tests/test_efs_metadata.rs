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

use mft::attribute::MftAttributeType;
use mft::attribute::header::ResidentialHeader;
use ntfs::image::EwfImage;
use ntfs::ntfs::efs::EfsMetadataV1;
use ntfs::ntfs::{FileSystem, Volume};
use std::sync::Arc;

mod common;

#[test]
fn test_parse_efs_metadata_v1_has_ddf_entries() {
    let img_path = common::ntfs_fixture_path("ntfs1-gen2.E01");
    if !common::ensure_fixture(&img_path) {
        return;
    }
    let img = EwfImage::open(img_path).unwrap();
    let volume = Volume::open(Arc::new(img), 0).unwrap();
    let fs = FileSystem::new(volume);

    let encrypted_id = fs.resolve_path("\\Encrypted\\NIST_logo.jpg").unwrap();
    let entry = fs.volume().read_mft_entry(encrypted_id).unwrap();

    let attr = entry
        .iter_attributes_matching(Some(vec![MftAttributeType::LoggedUtilityStream]))
        .filter_map(std::result::Result::ok)
        .find(|a| a.header.name == "$EFS")
        .expect("missing $EFS attribute");

    let efs_blob = match &attr.header.residential_header {
        ResidentialHeader::Resident(rh) => {
            let start = attr.header.start_offset as usize + rh.data_offset as usize;
            let end = start + rh.data_size as usize;
            entry.data[start..end].to_vec()
        }
        ResidentialHeader::NonResident(nr) => {
            let runs = attr.data.as_data_runs().unwrap().data_runs.clone();
            let mut buf = vec![0u8; nr.file_size as usize];
            ntfs::ntfs::data_stream::read_from_data_runs(fs.volume(), &runs, 0, &mut buf).unwrap();
            buf
        }
    };

    let meta = EfsMetadataV1::parse(&efs_blob, 0).unwrap();
    assert!(!meta.ddf.is_empty(), "expected at least one DDF entry");

    // The fixture should include an RSA-wrapped FEK (flags=0) in at least one entry.
    assert!(
        meta.ddf.iter().any(|e| e.flags == 0),
        "expected at least one RSA-wrapped DDF entry"
    );
}
