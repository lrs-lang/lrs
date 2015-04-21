// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[prelude_import] use base::prelude::*;
use core::{mem};
use base::rmo::{AsRef, AsMut};
use str_one::c_str::{CStr};
use fmt::{Debug, Write};
use vec::{Vec};

pub struct CString {
    data: Vec<u8>,
}

impl CString {
    /// Casts the byte vector directly to a `CString` without checking it for validity.
    pub unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> CString {
        CString { data: bytes }
    }
}

impl Deref for CString {
    type Target = CStr;
    fn deref(&self) -> &CStr {
        unsafe { mem::cast(self.data.deref()) }
    }
}

impl Debug for CString {
    fn fmt<W: Write>(&self, w: &mut W) -> Result {
        Debug::fmt(self.deref(), w)
    }
}

impl AsRef<CStr> for CString {
    fn as_ref(&self) -> &CStr {
        unsafe { CStr::from_bytes_unchecked(&self.data[..]) }
    }
}

impl AsMut<CStr> for CString {
    fn as_mut(&mut self) -> &mut CStr {
        unsafe { CStr::from_bytes_unchecked_mut(&mut self.data[..]) }
    }
}
