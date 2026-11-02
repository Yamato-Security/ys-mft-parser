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

//! EWF1 (E01/S01/L01) format primitives.
//!
//! This module contains small, **spec-driven** building blocks shared by the reader and writer
//! implementations for the original EWF container formats (EWF1).
//!
//! Reference material:
//! - `external/libewf/documentation/Expert Witness Compression Format (EWF).asciidoc`
//! - libewf reference implementation: `external/libewf/`

/// EWF1 EVF segment file signature (`EVF\t\r\n\xff\x00`).
pub(crate) const EWF1_EVF_SIGNATURE: [u8; 8] = [0x45, 0x56, 0x46, 0x09, 0x0d, 0x0a, 0xff, 0x00];

/// EWF1 LVF segment file signature (`LVF\t\r\n\xff\x00`) used by logical evidence (`.L01`).
pub(crate) const EWF1_LVF_SIGNATURE: [u8; 8] = [0x4c, 0x56, 0x46, 0x09, 0x0d, 0x0a, 0xff, 0x00];

/// EWF1 file header size (13 bytes).
pub(crate) const EWF1_FILE_HEADER_SIZE: usize = 8 + 1 + 2 + 2;

/// EWF1 section descriptor size (76 bytes).
pub(crate) const EWF1_SECTION_DESCRIPTOR_SIZE: usize = 16 + 8 + 8 + 40 + 4;

/// EWF1 chunk table header size (24 bytes).
pub(crate) const EWF1_TABLE_HEADER_SIZE: usize = 4 + 4 + 8 + 4 + 4;

pub(crate) mod file_header;
pub(crate) mod header;
pub(crate) mod runs;
pub(crate) mod section;
pub(crate) mod volume;
