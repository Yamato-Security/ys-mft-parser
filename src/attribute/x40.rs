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

use std::io::{Read, Seek};

use crate::err::{Error, Result};
use serde::Serialize;
use winstructs::guid::Guid;

/// $Data Attribute
#[derive(Serialize, Clone, Debug)]
pub struct ObjectIdAttr {
    /// Unique Id assigned to file
    pub object_id: Guid,
    /// Volume where file was created
    pub birth_volume_id: Option<Guid>,
    /// Original Object Id of file
    pub birth_object_id: Option<Guid>,
    /// Domain in which object was created
    pub domain_id: Option<Guid>,
}

impl ObjectIdAttr {
    /// Data size should be either 16 or 64
    pub fn from_stream<S: Read + Seek>(stream: &mut S, data_size: usize) -> Result<ObjectIdAttr> {
        let object_id = Guid::from_reader(stream).map_err(Error::failed_to_read_guid)?;
        let (birth_volume_id, birth_object_id, domain_id) = if data_size == 64 {
            let g1 = Guid::from_reader(stream).map_err(Error::failed_to_read_guid)?;
            let g2 = Guid::from_reader(stream).map_err(Error::failed_to_read_guid)?;
            let g3 = Guid::from_reader(stream).map_err(Error::failed_to_read_guid)?;
            (Some(g1), Some(g2), Some(g3))
        } else {
            (None, None, None)
        };

        Ok(ObjectIdAttr {
            object_id,
            birth_volume_id,
            birth_object_id,
            domain_id,
        })
    }
}
