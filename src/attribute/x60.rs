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

use crate::Utf16LeStr;
use serde::Serialize;

/// $VOLUME_NAME Attribute. Present in the `$Volume` file (entry 3).
#[derive(Serialize, Clone, Debug)]
pub struct VolumeNameAttr<'a> {
    /// The volume label. Empty when the volume has no label.
    pub name: Utf16LeStr<'a>,
}

impl<'a> VolumeNameAttr<'a> {
    /// The attribute value is the label as UTF-16LE, without a terminator.
    ///
    /// # Example
    ///
    /// ```
    /// use mft::attribute::x60::VolumeNameAttr;
    ///
    /// let value: &[u8] = &[0x44, 0x00, 0x61, 0x00, 0x74, 0x00, 0x61, 0x00];
    /// let attribute = VolumeNameAttr::from_slice(value);
    ///
    /// assert_eq!(attribute.name.to_utf8_string(), "Data");
    /// ```
    pub fn from_slice(value: &'a [u8]) -> VolumeNameAttr<'a> {
        VolumeNameAttr {
            name: Utf16LeStr::from_utf16le_bytes(value),
        }
    }
}
