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

//! Parses one MFT record and walks everything a consumer would touch.
#![no_main]

use libfuzzer_sys::fuzz_target;
use mft::MftEntry;

fuzz_target!(|data: &[u8]| {
    for entry in [
        MftEntry::from_buffer(data.to_vec(), 0),
        MftEntry::from_buffer_skip_fixup(data.to_vec(), 0),
    ]
    .into_iter()
    .flatten()
    {
        let _ = entry.find_best_name_attribute();
        // Serializing walks `iter_attributes` and every attribute's content.
        let _ = serde_json::to_writer(std::io::sink(), &entry);
    }
});
