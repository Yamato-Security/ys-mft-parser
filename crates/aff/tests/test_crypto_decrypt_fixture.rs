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

use aff::AffOpenOptions;
use forensic_image::ReadAt;

mod common;

#[test]
fn test_decrypt_afflib_encrypted_fixture_matches_raw() {
    // AFFLIBv3 fixture: encrypted.aff is encrypted with passphrase "password".
    let aff_path = common::afflib_fixture_path("encrypted.aff");
    let raw_path = common::afflib_fixture_path("encrypted.raw");
    if !common::ensure_fixture(&aff_path) || !common::ensure_fixture(&raw_path) {
        return;
    }

    let mut opts = AffOpenOptions::new();
    opts.passphrase = Some("password".to_string());
    opts.auto_decrypt = true;

    let img = opts.open(&aff_path).expect("open encrypted.aff");

    let raw = std::fs::read(&raw_path).expect("read encrypted.raw");
    assert_eq!(img.len(), raw.len() as u64);

    let mut out = vec![0u8; raw.len()];
    img.read_exact_at(0, &mut out)
        .expect("read decrypted bytes");
    assert_eq!(out, raw, "decrypted content mismatch");
}
