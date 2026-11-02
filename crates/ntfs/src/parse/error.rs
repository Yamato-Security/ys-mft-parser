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

use crate::parse::hexdump::hexdump_around;
use std::error::Error as StdError;
use std::fmt;

pub type Result<T> = std::result::Result<T, ParseError>;

/// A parsing error that captures the logical stream offset and a small hexdump window.
///
/// This is intentionally *not* an enum at the moment: in practice we want rich context (field
/// name + offset + bytes). Higher layers can categorize errors if/when needed.
#[derive(Debug)]
pub struct ParseError {
    offset: u64,
    field: &'static str,
    message: String,
    hexdump: String,
    source: Box<dyn StdError + Send + Sync + 'static>,
}

impl ParseError {
    pub fn new(
        offset: u64,
        field: &'static str,
        message: impl Into<String>,
        hexdump: impl Into<String>,
        source: Box<dyn StdError + Send + Sync + 'static>,
    ) -> Self {
        Self {
            offset,
            field,
            message: message.into(),
            hexdump: hexdump.into(),
            source,
        }
    }

    pub fn offset(&self) -> u64 {
        self.offset
    }

    pub fn field(&self) -> &'static str {
        self.field
    }

    pub fn hexdump(&self) -> &str {
        &self.hexdump
    }

    pub(crate) fn capture_from_slice(
        buf: &[u8],
        base_offset: u64,
        pos: usize,
        field: &'static str,
        message: impl Into<String>,
        source: Box<dyn StdError + Send + Sync + 'static>,
    ) -> Self {
        let offset = base_offset.saturating_add(pos as u64);
        let hexdump = hexdump_around(buf, base_offset, pos, 96, 64);
        Self::new(offset, field, message, hexdump, source)
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Offset `0x{offset:08x} ({offset})` - failed to parse `{field}`\n\
             {message}\n\n\
             Source: {source}\n\n\
             Hexdump:\n{hexdump}",
            offset = self.offset,
            field = self.field,
            message = self.message,
            source = self.source,
            hexdump = self.hexdump
        )
    }
}

impl StdError for ParseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.source.as_ref())
    }
}
