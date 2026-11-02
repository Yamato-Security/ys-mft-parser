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
use std::path::PathBuf;

use std::sync::Once;

static LOGGER_INIT: Once = Once::new();

// Rust runs the tests concurrently, so unless we synchronize logging access
// it will crash when attempting to run `cargo test` with some logging facilities.
pub fn ensure_env_logger_initialized() {
    LOGGER_INIT.call_once(env_logger::init);
}

pub fn samples_dir() -> PathBuf {
    PathBuf::from(file!())
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("samples")
        .canonicalize()
        .unwrap()
}

pub fn mft_sample() -> PathBuf {
    mft_sample_name("MFT")
}

pub fn mft_sample_name(filename: &str) -> PathBuf {
    samples_dir().join(filename)
}
