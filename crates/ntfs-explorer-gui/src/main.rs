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

mod app;
mod components;
mod icons;
#[cfg(feature = "desktop")]
mod menus;
mod mft_only;
mod settings;
mod styles;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    #[cfg(feature = "desktop")]
    {
        use dioxus::LaunchBuilder;
        use dioxus::desktop::{Config, WindowBuilder};

        LaunchBuilder::desktop()
            .with_cfg(
                Config::new().with_menu(menus::build_menu()).with_window(
                    WindowBuilder::new()
                        .with_title("NTFS Explorer")
                        .with_inner_size(dioxus::desktop::LogicalSize::new(1200.0, 800.0)),
                ),
            )
            .launch(app::app);
    }

    // LiveView runs the app natively, but renders the UI in the browser.
    #[cfg(feature = "liveview")]
    {
        dioxus::launch(app::app);
    }

    #[cfg(feature = "web")]
    {
        dioxus::launch(app::app);
    }
}
