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

use crate::icons;

#[linkme::distributed_slice(crate::styles::STYLESHEETS)]
static ADDRESS_BAR_CSS: crate::styles::Stylesheet = crate::styles::Stylesheet {
    order: 200,
    name: "components/address_bar",
    href: asset!("/src/components/address_bar.css"),
};

#[component]
pub fn AddressBar(
    path: String,
    on_navigate_up: Callback<()>,
    on_navigate_to_path: Callback<String>,
    can_go_up: bool,
) -> Element {
    // Split path into breadcrumb segments
    let segments: Vec<&str> = path.split('\\').filter(|s| !s.is_empty()).collect();
    let mut crumbs: Vec<(String, String, bool)> = Vec::with_capacity(segments.len());
    let mut cur = String::from("\\");
    for (i, seg) in segments.iter().enumerate() {
        if cur != "\\" {
            cur.push('\\');
        }
        cur.push_str(seg);
        crumbs.push((
            seg.to_string(),
            cur.clone(),
            i == segments.len().saturating_sub(1),
        ));
    }

    rsx! {
        div { class: "address-bar",
            // Navigation buttons
            div { class: "address-nav-group",
                button {
                    class: "address-nav-btn",
                    disabled: !can_go_up,
                    onclick: move |_| on_navigate_up.call(()),
                    title: "Up one level (Alt+Up)",
                    span { class: "address-nav-icon", {icons::arrow_up()} }
                }
            }

            // Breadcrumb path
            div { class: "address-path",
                // Root / Volume
                button {
                    r#type: "button",
                    class: "address-segment is-root",
                    onclick: move |_| on_navigate_to_path.call("\\".to_string()),
                    span { class: "address-segment-icon", {icons::this_pc()} }
                    span { class: "address-segment-text", "Volume" }
                }

                // Path segments
                for (label, crumb_path, is_current) in crumbs {
                    span { class: "address-separator", {icons::breadcrumb_sep()} }
                    button {
                        r#type: "button",
                        class: if is_current { "address-segment is-current" } else { "address-segment" },
                        onclick: move |_| on_navigate_to_path.call(crumb_path.clone()),
                        span { class: "address-segment-icon", {icons::folder_closed()} }
                        span { class: "address-segment-text", "{label}" }
                    }
                }
            }
        }
    }
}
