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

//! AFF crypto (read-side).
//!
//! This module provides a wrapper layer around a container backend which can:
//! - derive an AFF AES key from `affkey_aes256` using a passphrase
//! - unseal `affkey_evp%d` using an RSA private key (PEM)
//! - auto-decrypt `/aes256` segments on read (CBC with IV derived from segment name)
//!
//! Signature verification is implemented in [`crate::verify`].
//!
//! The implementation is modeled after AFFLIBv3 (`lib/crypto.cpp` + `lib/afflib.cpp`).

mod wrapper;

pub(crate) use wrapper::wrap_backend;
