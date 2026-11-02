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

use crate::attribute::MftAttributeType;
use crate::utils;
use serde::{Serialize, ser};

/// Placeholder attribute for currently unparsed attributes.
#[derive(Serialize, Clone, Debug)]
pub struct RawAttribute<'a> {
    pub attribute_type: MftAttributeType,
    #[serde(serialize_with = "data_as_hex")]
    pub data: &'a [u8],
}

impl<'a> RawAttribute<'a> {
    pub fn from_slice(attribute_type: MftAttributeType, data: &'a [u8]) -> Self {
        RawAttribute {
            attribute_type,
            data,
        }
    }
}

pub(crate) fn data_as_hex<S>(x: &[u8], s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: ser::Serializer,
{
    s.serialize_str(&utils::to_hex_string(x))
}
