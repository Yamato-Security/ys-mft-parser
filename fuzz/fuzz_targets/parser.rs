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

//! Treats the input as a whole $MFT: reads every record and builds its CSV row, which resolves
//! the full path by following parent references between records.
#![no_main]

use libfuzzer_sys::fuzz_target;
use mft::MftParser;
use mft::csv::FlatMftEntryWithName;
use std::io::Cursor;

fuzz_target!(|data: &[u8]| {
    let Ok(mut parser) = MftParser::from_read_seek(Cursor::new(data), Some(data.len() as u64))
    else {
        return;
    };

    for i in 0..parser.get_entry_count() {
        if let Ok(entry) = parser.get_entry(i) {
            let _ = FlatMftEntryWithName::from_entry(&entry, &mut parser);
        }
    }
});
