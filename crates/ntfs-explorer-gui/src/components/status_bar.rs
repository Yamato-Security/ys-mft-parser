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

use dioxus::prelude::*;

#[linkme::distributed_slice(crate::styles::STYLESHEETS)]
static STATUS_BAR_CSS: crate::styles::Stylesheet = crate::styles::Stylesheet {
    order: 200,
    name: "components/status_bar",
    href: asset!("/src/components/status_bar.css"),
};

#[component]
pub fn StatusBar(
    items_total: usize,
    items_deleted: usize,
    items_encrypted: usize,
    selected_dir_id: Option<u64>,
) -> Element {
    rsx! {
        div { class: "status-bar",
            div { class: "status-left",
                span { class: "status-item",
                    span { class: "status-count", "{items_total}" }
                    span { class: "status-label", "items" }
                }

                if items_deleted > 0 {
                    span { class: "status-separator" }
                    span { class: "status-item is-deleted",
                        span { class: "status-count", "{items_deleted}" }
                        span { class: "status-label", "deleted" }
                    }
                }

                if items_encrypted > 0 {
                    span { class: "status-separator" }
                    span { class: "status-item is-encrypted",
                        span { class: "status-count", "{items_encrypted}" }
                        span { class: "status-label", "encrypted" }
                    }
                }
            }

            div { class: "status-right",
                if let Some(id) = selected_dir_id {
                    span { class: "status-item is-muted",
                        "MFT #{id}"
                    }
                }
            }
        }
    }
}
