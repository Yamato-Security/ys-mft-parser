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

//! Encrypting File System (EFS) support.
//!
//! This module implements **offline EFS decryption** for NTFS volumes:
//!
//! - Parse the on-disk `$EFS` metadata stored in the `LoggedUtilityStream` attribute
//!   (`$LOGGED_UTILITY_STREAM`, attribute type `0x100`) named `"$EFS"`.
//! - Unwrap the **File Encryption Key (FEK)** from the DDF/DRF entries using an RSA private key
//!   (typically supplied as a PKCS#12 / `.pfx` file).
//! - Decrypt file `$DATA` **by 512-byte sectors**, using the FEK and the per-sector IV scheme used
//!   by Windows NTFS.
//!
//! ## References
//!
//! - `external/refs/specs/MS-EFSR.md` (MS-EFSR): structures used for the `$EFS` metadata and the
//!   "Encrypted FEK" structure.
//! - Reference implementations (vendored for offline reading only):
//!   - `external/refs/repos/ntfsprogs-plus__ntfsprogs-plus@*/src/deprecated/ntfsdecrypt.c`
//!   - `external/refs/repos/tuxera__ntfs-3g@*/libntfs-3g/efs.c`
//!
//! **Important**: this crate does not compile or link any code from `external/refs/`.

pub mod crypto;
pub mod metadata;
pub mod pfx;

pub use crypto::{EfsFek, EfsFekAlgorithm, EfsFekDecryptor};
pub use metadata::{EfsMetadataV1, KeyListEntry};
pub use pfx::EfsRsaKeyBag;
