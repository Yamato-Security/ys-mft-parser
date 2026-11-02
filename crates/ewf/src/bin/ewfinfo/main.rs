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

//! `ewfinfo` – show meta data stored in EWF files.
//!
//! This binary is the CLI front-end for the `ewf` crate.
//!
//! - Image metadata reporting (`-f text|dfxml`, `-i`/`-m`/`-e`, `-A`, `-d`) is implemented here
//!   (binary-owned reporting/rendering), backed by spec-oriented metadata from `ewf::EwfReader`.
//! - Logical evidence outputs (`-F`, `-H`, `-B`) are **CLI-only** and intentionally live here (not
//!   in the library), mirroring libewf’s separation between `info_handle` printing routines and the
//!   core readers.
//!
//! References:
//! - `external/libewf/ewftools/ewfinfo.c`
//! - `external/libewf/ewftools/info_handle.c`
//! - `external/libewf/ewftools/bodyfile.c`
//! - `external/libewf/manuals/ewfinfo.1`

mod bodyfile;
mod cli;
mod ewfinfo;
mod logical;

use clap::Parser;

fn main() -> miette::Result<()> {
    cli::Cli::parse().run()
}
