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

#[macro_export]
macro_rules! impl_serialize_for_bitflags {
    ($flags: ident) => {
        impl ::std::fmt::Display for $flags {
            /// Formats the flags as `A | B`, the same way `mft` did with bitflags 1.x.
            /// Empty flags are formatted as the name of a zero-valued flag if there is one,
            /// otherwise as `(empty)`.
            fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                $crate::utils::write_bitflags(self, f)
            }
        }

        impl serde::ser::Serialize for $flags {
            fn serialize<S>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error>
            where
                S: serde::ser::Serializer,
            {
                // Same text as `Display`, but built in a stack buffer without going through
                // `core::fmt` (which dominated serialization time).
                $crate::utils::serialize_bitflags(self, serializer)
            }
        }
    };
}
