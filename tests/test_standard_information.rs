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

mod fixtures;

use fixtures::*;

use mft::attribute::MftAttributeType;
use mft::attribute::header::ResidentialHeader;
use mft::mft::MftParser;

/// Regression test for a subtle parsing bug:
///
/// `$STANDARD_INFORMATION` can be **48 bytes** (base layout) or **72 bytes** (extended layout).
/// Older code parsed it by reading a fixed 72-byte layout from the entry cursor, which meant that
/// for 48-byte values it would read 24 bytes from the *next attribute record header* and interpret
/// them as `owner_id/security_id/quota/usn` garbage.
#[test]
fn standard_information_len_48_does_not_leak_next_attribute_header_bytes() {
    // The sample MFT contains at least one entry where `$STANDARD_INFORMATION` is 48 bytes.
    // (In practice, the root directory entry 5 is one such case.)
    let sample = mft_sample();
    let mut parser = MftParser::from_path(sample).unwrap();

    let entry = parser.get_entry(5).unwrap();
    let attr = entry
        .iter_attributes_matching(Some(vec![MftAttributeType::StandardInformation]))
        .filter_map(Result::ok)
        .next()
        .expect("expected $STANDARD_INFORMATION");

    let data_size = match &attr.header.residential_header {
        ResidentialHeader::Resident(r) => r.data_size,
        ResidentialHeader::NonResident(_) => panic!("$STANDARD_INFORMATION must be resident"),
    };
    assert_eq!(data_size, 48, "fixture should exercise the 48-byte layout");

    let si = attr
        .data
        .as_standard_info()
        .expect("expected parsed standard info");

    // Extended fields are absent in the 48-byte layout => should be zero.
    assert_eq!(si.owner_id, 0);
    assert_eq!(si.security_id, 0);
    assert_eq!(si.quota, 0);
    assert_eq!(si.usn, 0);
}
