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

//! Backend abstraction shared by all container implementations.

use forensic_image::ReadAt;
use std::io;

/// Concrete container kind backing an [`crate::backends::AffImage`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerKind {
    /// AFF1 single-file (`.aff`).
    Aff1,
    /// AFM metadata file (`.afm`) + split-raw payload.
    Afm,
    /// AFD directory container.
    Afd,
}

/// Raw segment bytes as stored by the underlying container (no decryption, no signature checks).
///
/// Higher layers (crypto wrapper) may expose **decrypted** views in the future.
#[derive(Debug, Clone)]
pub struct Segment {
    /// Segment name (e.g. `pagesize`, `imagesize`, `page0`).
    pub name: String,
    /// Segment `arg` field (AFFLIB calls this `flag` in `af_segment_head`).
    pub arg: u32,
    /// Segment data bytes.
    pub data: Vec<u8>,
}

pub(crate) trait Backend: ReadAt {
    fn kind(&self) -> ContainerKind;
    fn page_size(&self) -> usize;

    /// Lists segment names present in the container.
    fn segment_names(&self) -> Vec<String>;

    /// Reads a segment by name.
    ///
    /// Returns `Ok(None)` if the segment does not exist.
    fn read_segment(&self, name: &str) -> io::Result<Option<Segment>>;
}
