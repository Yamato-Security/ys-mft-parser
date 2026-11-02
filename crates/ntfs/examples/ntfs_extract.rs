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

//! Dev helper: extract an NTFS file's default stream to a local path.
//!
//! Usage:
//! - `cargo run -p ntfs --example ntfs_extract -- <image.E01> <ntfs-path> <out-path>`
//!
//! Example:
//! - `cargo run -p ntfs --example ntfs_extract -- testdata/ntfs/ntfs1-gen2.E01 \\Raw\\NIST_logo.jpg /tmp/NIST_logo.jpg`

#![forbid(unsafe_code)]

use ntfs::image::EwfImage;
use ntfs::ntfs::{FileSystem, Volume};
use std::path::PathBuf;
use std::sync::Arc;

fn main() {
    let img_path = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .expect("missing <image.E01>");
    let ntfs_path = std::env::args().nth(2).expect("missing <ntfs-path>");
    let out_path = std::env::args_os()
        .nth(3)
        .map(PathBuf::from)
        .expect("missing <out-path>");

    let img = EwfImage::open(img_path).expect("failed to open EWF image");
    let volume = Volume::open(Arc::new(img), 0).expect("failed to open NTFS volume");
    let fs = FileSystem::new(volume);

    let entry_id = fs.resolve_path(&ntfs_path).expect("failed to resolve path");
    let bytes = fs
        .read_file_default_stream(entry_id)
        .expect("failed to read default stream");

    std::fs::write(&out_path, bytes).expect("failed to write output");
    eprintln!("wrote {}", out_path.display());
}
