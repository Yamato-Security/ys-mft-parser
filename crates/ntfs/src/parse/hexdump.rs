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

/// Creates a compact hexdump around `pos` in `buf`.
///
/// `base_offset` is used for labeling offsets in the dump (useful when `buf` is a window into a
/// larger stream).
pub(crate) fn hexdump_around(
    buf: &[u8],
    base_offset: u64,
    pos: usize,
    lookbehind: usize,
    lookahead: usize,
) -> String {
    if buf.is_empty() {
        return "<empty buffer>".to_string();
    }

    let start = pos.saturating_sub(lookbehind);
    let end = (pos + lookahead).min(buf.len());

    // 16 bytes per line.
    let mut out = String::new();
    let mut i = start - (start % 16);
    while i < end {
        let line_start = i;
        let line_end = (i + 16).min(buf.len());

        // Offset.
        out.push_str(&format!(
            "0x{off:08x}  ",
            off = base_offset.saturating_add(line_start as u64)
        ));

        // Hex bytes.
        for b in buf[line_start..line_end]
            .iter()
            .copied()
            .map(Some)
            .chain(std::iter::repeat(None))
            .take(16)
        {
            match b {
                Some(b) => out.push_str(&format!("{b:02x} ")),
                None => out.push_str("   "),
            }
        }

        // ASCII.
        out.push(' ');
        for b in buf[line_start..line_end]
            .iter()
            .copied()
            .map(Some)
            .chain(std::iter::repeat(None))
            .take(16)
        {
            match b {
                Some(b) => {
                    let c = if b.is_ascii_graphic() || b == b' ' {
                        b as char
                    } else {
                        '.'
                    };
                    out.push(c);
                }
                None => out.push(' '),
            }
        }

        // Mark the cursor line/position.
        if pos >= line_start && pos < line_start + 16 {
            let caret_pos = 10 + 2 + (pos - line_start) * 3; // after "0x........  "
            out.push('\n');
            out.push_str(&" ".repeat(caret_pos));
            out.push('^');
        }

        out.push('\n');
        i += 16;
    }

    out
}
