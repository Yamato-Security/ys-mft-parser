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

#![forbid(unsafe_code)]
#![deny(unused_must_use)]
// Don't allow dbg! prints in release.
#![cfg_attr(not(debug_assertions), deny(clippy::dbg_macro))]

#[macro_use]
extern crate num_derive;

pub use attribute::MftAttribute;
pub use attribute::x10::StandardInfoAttr;
pub use attribute::x30::FileNameAttr;
pub use jiff::Timestamp;

pub use crate::mft::MftParser;
pub use entry::{EntryHeader, MftEntry};

pub mod attribute;
pub mod bodyfile;
pub mod csv;
pub mod entry;
pub mod err;
pub mod i30;
pub mod mft;
pub mod ntfs;
pub mod utf16;

pub(crate) mod macros;
pub(crate) mod utils;

// Public utilities (used by `ys-mft-parser` and helpful for library consumers).
pub use utils::{
    serialize_option_timestamp, serialize_option_windows_path, serialize_timestamp,
    serialize_windows_path, windows_filetime_to_timestamp, windows_path,
};

pub use utf16::Utf16LeStr;

#[cfg(test)]
pub(crate) mod tests;
