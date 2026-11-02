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

//! EWF2 chunk-table primitives.
//!
//! EWF2 stores per-chunk metadata in the *sector table* section. Each table entry contains a flags
//! bitmask describing how the corresponding chunk is stored (compressed, checksumed, pattern fill,
//! etc.).

use bitflags::bitflags;

bitflags! {
    /// EWF2 sector-table entry flags.
    ///
    /// Unknown bits are preserved (see [`Ewf2ChunkDataFlags::from_raw`]).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(crate) struct Ewf2ChunkDataFlags: u32 {
        const COMPRESSED  = 0x0000_0001;
        const CHECKSUMED  = 0x0000_0002;
        const PATTERNFILL = 0x0000_0004;
    }
}

impl Ewf2ChunkDataFlags {
    pub(crate) fn from_raw(raw: u32) -> Self {
        Self::from_bits_retain(raw)
    }

    pub(crate) fn raw(self) -> u32 {
        self.bits()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_data_flags_preserves_unknown_bits() {
        let raw = 0x4000_0000u32 | Ewf2ChunkDataFlags::CHECKSUMED.bits();
        let flags = Ewf2ChunkDataFlags::from_raw(raw);
        assert_eq!(flags.raw(), raw);
        assert!(flags.contains(Ewf2ChunkDataFlags::CHECKSUMED));
        assert!(!flags.contains(Ewf2ChunkDataFlags::COMPRESSED));
    }
}
