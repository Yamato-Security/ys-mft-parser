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

use std::sync::LazyLock;

use dioxus::prelude::*;

#[derive(Clone, Copy)]
pub struct Stylesheet {
    pub order: u16,
    pub name: &'static str,
    pub href: Asset,
}

#[linkme::distributed_slice]
pub static STYLESHEETS: [Stylesheet] = [..];

static SORTED_STYLESHEETS: LazyLock<Vec<Stylesheet>> = LazyLock::new(|| {
    let mut sheets: Vec<Stylesheet> = STYLESHEETS.iter().copied().collect();
    sheets.sort_unstable_by(|a, b| (a.order, a.name).cmp(&(b.order, b.name)));
    sheets
});

pub fn stylesheets() -> &'static [Stylesheet] {
    SORTED_STYLESHEETS.as_slice()
}

// ---------------------------------------------------------------------------
// LiveView
// ---------------------------------------------------------------------------
//
// LiveView's default Axum adapter does not serve static assets (it uses a catch-all route that
// returns the LiveView HTML shell). Instead of relying on `asset!()` + `<link rel="stylesheet">`,
// inline the CSS so the UI renders correctly.
#[cfg(feature = "liveview")]
pub const INLINE_CSS: &str = concat!(
    include_str!("styles/tokens.css"),
    "\n",
    include_str!("styles/base.css"),
    "\n",
    // Components
    include_str!("components/toolbar.css"),
    "\n",
    include_str!("components/address_bar.css"),
    "\n",
    include_str!("components/navigation_pane.css"),
    "\n",
    include_str!("components/details_view.css"),
    "\n",
    include_str!("components/resize.css"),
    "\n",
    include_str!("components/context_menu.css"),
    "\n",
    include_str!("components/status_bar.css"),
    "\n",
);

#[cfg(not(feature = "liveview"))]
pub const INLINE_CSS: &str = "";

// ---------------------------------------------------------------------------
// Core stylesheets (tokens → base → components)
// ---------------------------------------------------------------------------

#[linkme::distributed_slice(STYLESHEETS)]
static TOKENS_STYLESHEET: Stylesheet = Stylesheet {
    order: 0,
    name: "styles/tokens",
    href: asset!("/src/styles/tokens.css"),
};

#[linkme::distributed_slice(STYLESHEETS)]
static BASE_STYLESHEET: Stylesheet = Stylesheet {
    order: 100,
    name: "styles/base",
    href: asset!("/src/styles/base.css"),
};
