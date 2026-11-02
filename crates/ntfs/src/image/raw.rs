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

use crate::image::ReadAt;
use std::fs::File;
use std::io;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::FileExt;
#[cfg(windows)]
use std::os::windows::fs::FileExt;

#[cfg(unix)]
fn file_read_at(file: &File, buf: &mut [u8], offset: u64) -> io::Result<usize> {
    file.read_at(buf, offset)
}

#[cfg(windows)]
fn file_read_at(file: &File, buf: &mut [u8], offset: u64) -> io::Result<usize> {
    file.seek_read(buf, offset)
}

#[derive(Debug)]
pub struct RawImage {
    file: File,
    len: u64,
}

impl RawImage {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file = File::open(path)?;
        let len = file.metadata()?.len();
        Ok(Self { file, len })
    }
}

impl ReadAt for RawImage {
    fn len(&self) -> u64 {
        self.len
    }

    fn read_exact_at(&self, offset: u64, mut buf: &mut [u8]) -> io::Result<()> {
        if offset.saturating_add(buf.len() as u64) > self.len {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        }

        let mut cur_offset = offset;
        while !buf.is_empty() {
            let read = file_read_at(&self.file, buf, cur_offset)?;
            if read == 0 {
                return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
            }
            cur_offset = cur_offset.saturating_add(read as u64);
            let tmp = buf;
            buf = &mut tmp[read..];
        }

        Ok(())
    }
}
