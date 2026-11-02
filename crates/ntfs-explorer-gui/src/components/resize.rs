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
static RESIZE_CSS: crate::styles::Stylesheet = crate::styles::Stylesheet {
    order: 250,
    name: "components/resize",
    href: asset!("/src/components/resize.css"),
};

#[component]
pub fn ResizeOverlay(on_move: Callback<MouseEvent>, on_end: Callback<MouseEvent>) -> Element {
    rsx! {
        div {
            class: "resize-overlay",
            onmousemove: move |e| on_move.call(e),
            onmouseup: move |e| on_end.call(e),
        }
    }
}

#[component]
pub fn ResizeHandle(active: bool, on_start: Callback<MouseEvent>) -> Element {
    let class = if active {
        "resize-handle is-active"
    } else {
        "resize-handle"
    };

    rsx! {
        div {
            class,
            onmousedown: move |e| on_start.call(e),
        }
    }
}
