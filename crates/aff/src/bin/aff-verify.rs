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

use aff::{AffOpenOptions, SignatureStatus, Verifier};
use clap::Parser;
use std::path::PathBuf;

/// Verify `/sha256` signature segments in an AFF container.
#[derive(Debug, Parser)]
#[command(author, version, about)]
struct Cli {
    /// Path to an AFF1 file, an AFM file, or an AFD directory.
    path: PathBuf,

    /// Passphrase for decrypting `/aes256` segments (AFFLIB `affkey_aes256`).
    #[arg(long)]
    passphrase: Option<String>,

    /// PEM private key used to unseal `affkey_evp%d` segments.
    #[arg(long)]
    unseal_keyfile: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let mut opts = AffOpenOptions::new();
    opts.passphrase = cli.passphrase;
    opts.unseal_keyfile = cli.unseal_keyfile;

    let img = opts.open(&cli.path)?;
    let verifier = Verifier::new(&img);
    let results = verifier.verify_all()?;

    if results.is_empty() {
        println!("no signature segments found");
        return Ok(());
    }

    let mut bad = false;
    for (name, status) in results {
        println!("{status:?}\t{name}");
        if status != SignatureStatus::Good {
            bad = true;
        }
    }

    if bad {
        std::process::exit(1);
    }
    Ok(())
}
