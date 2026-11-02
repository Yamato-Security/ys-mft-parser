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

//! Expert Witness Compression Format (EWF) reader/writer.
//!
//! This crate is a Rust implementation of the EWF family of formats (e.g. EnCase `.E01`, SMART
//! `.S01`, EnCase EWF2 `.Ex01`, and the logical evidence variants `.L01`/`.Lx01`).
//!
//! The implementation is intended to be **spec-driven** and **compatible with libewf** (see the
//! pinned libewf reference commit in `external/refs/repos/libyal__libewf.commit`).
//!
//! Note: The public API is stabilized as part of the crate extraction work. Expect additions as we
//! expand format coverage (EWF2, delta/shadow files, write resume, etc.).

mod error;
mod ewf1;
mod ewf2;
mod info;
mod util;

pub mod delta;
pub mod metadata;
pub mod reader;
pub mod writer;

pub use delta::EwfDelta;
pub use error::{Error, Result};
pub use info::{EwfCompression, EwfFileFormat, EwfFormat, EwfInfo};
pub use reader::{EwfReader, LefEntry, LefExtent, LefReader, VerifyOptions};
pub use writer::{Ewf2CompressionMethod, Ewf2Writer, Ewf2WriterOptions, EwfWriter};
