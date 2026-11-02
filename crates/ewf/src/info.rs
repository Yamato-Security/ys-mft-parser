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

//! Public “shape”/metadata types.
//!
//! The underlying formats expose a large metadata surface area (case data, device info, hashes,
//! etc.). This module provides a small, stable summary for callers who mostly care about
//! “what kind of image is this?” and “how big is it?”.

use std::fmt;

/// High-level EWF format identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EwfFormat {
    /// EnCase EWF1 (EVF) disk image.
    E01,
    /// SMART EWF1 (EVF) disk image.
    S01,
    /// EnCase EWF2 (EVF2) disk image.
    Ex01,
    /// EWF1 logical evidence.
    L01,
    /// EWF2 logical evidence.
    Lx01,
}

/// libewf-compatible EWF *file format* classification.
///
/// This corresponds to what libewf exposes via `libewf_handle_get_format()` and what
/// `ewfinfo` prints under "EWF information" → "File format".
///
/// References (libewf):
/// - `external/libewf/ewftools/info_handle.c` (`info_handle_ewf_information_fprint`)
/// - `external/libewf/libewf/libewf_io_handle.c` (default format initialization)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EwfFileFormat {
    /// "original EWF"
    OriginalEwf,
    /// "SMART"
    Smart,
    /// "FTK Imager"
    FtkImager,
    EnCase1,
    EnCase2,
    EnCase3,
    EnCase4,
    EnCase5,
    EnCase6,
    EnCase7,
    Linen5,
    Linen6,
    Linen7,
    /// "EWFX (extended EWF)"
    Ewfx,
    /// "Logical Evidence File (LEF) EnCase 5"
    LogicalEnCase5,
    /// "Logical Evidence File (LEF) EnCase 6"
    LogicalEnCase6,
    /// "Logical Evidence File (LEF) EnCase 7"
    LogicalEnCase7,
    /// "EnCase 7 (version 2)"
    EnCase7V2,
    /// "Logical Evidence File (LEF) EnCase 7 (version 2)"
    LogicalEnCase7V2,
    /// Unknown / not classified.
    Unknown,
}

impl EwfFileFormat {
    /// Returns the exact libewf `ewfinfo` display string for this format.
    pub fn as_ewfinfo_str(self) -> &'static str {
        match self {
            Self::OriginalEwf => "original EWF",
            Self::Smart => "SMART",
            Self::FtkImager => "FTK Imager",
            Self::EnCase1 => "EnCase 1",
            Self::EnCase2 => "EnCase 2",
            Self::EnCase3 => "EnCase 3",
            Self::EnCase4 => "EnCase 4",
            Self::EnCase5 => "EnCase 5",
            Self::EnCase6 => "EnCase 6",
            Self::EnCase7 => "EnCase 7",
            Self::Linen5 => "linen 5",
            Self::Linen6 => "linen 6",
            Self::Linen7 => "linen 7",
            Self::Ewfx => "EWFX (extended EWF)",
            Self::LogicalEnCase5 => "Logical Evidence File (LEF) EnCase 5",
            Self::LogicalEnCase6 => "Logical Evidence File (LEF) EnCase 6",
            Self::LogicalEnCase7 => "Logical Evidence File (LEF) EnCase 7",
            Self::EnCase7V2 => "EnCase 7 (version 2)",
            Self::LogicalEnCase7V2 => "Logical Evidence File (LEF) EnCase 7 (version 2)",
            Self::Unknown => "unknown",
        }
    }
}

impl fmt::Display for EwfFileFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_ewfinfo_str())
    }
}

/// Compression algorithm identifier (not a guarantee every chunk is compressed).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EwfCompression {
    None,
    Zlib,
    Bzip2,
    Unknown(u16),
}

/// Summary metadata for an EWF image set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EwfInfo {
    pub format: EwfFormat,
    /// libewf-compatible file format classification.
    pub file_format: EwfFileFormat,
    pub media_size: u64,
    pub chunk_size: usize,
    pub chunk_count: u64,
    pub segment_count: usize,
    pub compression: EwfCompression,
}
