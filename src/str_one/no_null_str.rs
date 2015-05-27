// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#[prelude_import] use base::prelude::*;
use core::{mem};
use core::ops::{Index, IndexMut, Range, RangeFrom, RangeTo, RangeFull};
use base::rmo::{AsRef};
use base::{error};
use arch_fns::{memrchr, memchr};
use fmt::{Debug, Write};

use byte_str::{AsByteStr};
use c_str::{CStr, ToCStr};

/// A byte slice with no null bytes.
pub struct NoNullStr {
    data: [u8],
}

/// Objects that can be borrowed as a `NoNullStr`.
pub trait AsNoNullStr {
    /// Tries to borrow the object as a `NoNullStr`.
    fn as_no_null_str(&self) -> Result<&NoNullStr>;
}

/// Objects that can be borrowed as a mutable `NoNullStr`.
pub trait AsMutNoNullStr {
    /// Tries to borrow the object as a `NoNullStr`.
    fn as_mut_no_null_str(&mut self) -> Result<&mut NoNullStr>;
}

impl NoNullStr {
    /// Sets a byte in the slice to a value.
    ///
    /// [argument, idx]
    /// The index of the byte to be set.
    ///
    /// [argument, val]
    /// The value of the byte.
    ///
    /// = Remarks
    ///
    /// If `val == 0`, the process is aborted.
    pub fn set(&mut self, idx: usize, val: u8) {
        assert!(val != 0);
        self.data[idx] = val;
    }

    /// Returns a `NoNullStr` that consists of the segment after the last '/'.
    pub fn file(&self) -> &NoNullStr {
        let bytes = &self.data;
        match memrchr(bytes, b'/') {
            Some(idx) => unsafe { NoNullStr::from_bytes_unchecked(&bytes[idx+1..]) },
            _ => self,
        }
    }

    /// Returns a mutable `NoNullStr` that consists of the segment after the last '/'.
    pub fn file_mut(&mut self) -> &mut NoNullStr {
        unsafe { &mut *(self.file() as *const _ as *mut _) }
    }

    /// Returns a `NoNullStr` that consists of the segment before the last '/'.
    pub fn dir(&self) -> &NoNullStr {
        let bytes = &self.data;
        match memrchr(bytes, b'/') {
            Some(idx) => unsafe { NoNullStr::from_bytes_unchecked(&bytes[..idx]) },
            _ => unsafe { NoNullStr::from_bytes_unchecked(&[]) },
        }
    }

    /// Returns a mutable `NoNullStr` that consists of the segment before the last '/'.
    pub fn dir_mut(&mut self) -> &mut NoNullStr {
        unsafe { &mut *(self.dir() as *const _ as *mut _) }
    }

    /// Returns the length of the string.
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Returns whether the tring starts with a byte slice.
    ///
    /// [argument, arg]
    /// The byte slice to be checked.
    pub fn starts_with<A>(&self, arg: A) -> bool
        where A: AsRef<[u8]>,
    {
        let bytes: &[u8] = self.as_ref();
        bytes.starts_with(arg.as_ref())
    }

    /// Casts a byte slice to a `NoNullStr` without checking it for validity.
    ///
    /// [argument, bytes]
    /// The slice to be interpreted as a `NoNullStr`.
    ///
    /// = Remarks
    ///
    /// If the slice contains null bytes, the behavior is undefined.
    pub unsafe fn from_bytes_unchecked(bytes: &[u8]) -> &NoNullStr {
        mem::cast(bytes)
    }

    /// Casts a byte slice to a mutable `NoNullStr` without checking it for validity.
    ///
    /// [argument, bytes]
    /// The slice to be interpreted as a `NoNullStr`.
    ///
    /// = Remarks
    ///
    /// If the slice contains null bytes, the behavior is undefined.
    pub unsafe fn from_mut_bytes_unchecked(bytes: &mut [u8]) -> &mut NoNullStr {
        mem::cast(bytes)
    }
}

impl AsRef<[u8]> for NoNullStr {
    fn as_ref(&self) -> &[u8] { &self.data }
}

impl Index<RangeFull> for NoNullStr {
    type Output = NoNullStr;
    fn index(&self, _: RangeFull) -> &NoNullStr { self }
}

impl IndexMut<RangeFull> for NoNullStr {
    fn index_mut(&mut self, _: RangeFull) -> &mut NoNullStr { self }
}

impl Index<RangeTo<usize>> for NoNullStr {
    type Output = NoNullStr;
    fn index(&self, idx: RangeTo<usize>) -> &NoNullStr {
        unsafe { NoNullStr::from_bytes_unchecked(&self.data[idx]) }
    }
}

impl IndexMut<RangeTo<usize>> for NoNullStr {
    fn index_mut(&mut self, idx: RangeTo<usize>) -> &mut NoNullStr {
        unsafe { NoNullStr::from_mut_bytes_unchecked(&mut self.data[idx]) }
    }
}

impl Index<RangeFrom<usize>> for NoNullStr {
    type Output = NoNullStr;
    fn index(&self, idx: RangeFrom<usize>) -> &NoNullStr {
        unsafe { NoNullStr::from_bytes_unchecked(&self.data[idx]) }
    }
}

impl IndexMut<RangeFrom<usize>> for NoNullStr {
    fn index_mut(&mut self, idx: RangeFrom<usize>) -> &mut NoNullStr {
        unsafe { NoNullStr::from_mut_bytes_unchecked(&mut self.data[idx]) }
    }
}

impl Index<Range<usize>> for NoNullStr {
    type Output = NoNullStr;
    fn index(&self, idx: Range<usize>) -> &NoNullStr {
        unsafe { NoNullStr::from_bytes_unchecked(&self.data[idx]) }
    }
}

impl IndexMut<Range<usize>> for NoNullStr {
    fn index_mut(&mut self, idx: Range<usize>) -> &mut NoNullStr {
        unsafe { NoNullStr::from_mut_bytes_unchecked(&mut self.data[idx]) }
    }
}

impl ToCStr for NoNullStr {
    fn to_cstr<'a>(&self, buf: &'a mut [u8]) -> Result<&'a mut CStr> {
        let bytes = &self.data;
        if bytes.len() < buf.len() {
            mem::copy(buf, bytes);
            buf[bytes.len()] = 0;
            unsafe { Ok(CStr::from_mut_bytes_unchecked(&mut buf[..bytes.len()+1])) }
        } else {
            Err(error::NoMemory)
        }
    }
}

impl Debug for NoNullStr {
    fn fmt<W: Write>(&self, w: &mut W) -> Result {
        self.as_byte_str().fmt(w)
    }
}

///////////////////////////////

impl<'a, T: AsNoNullStr+?Sized> AsNoNullStr for &'a T {
    fn as_no_null_str(&self) -> Result<&NoNullStr> { (**self).as_no_null_str() }
}

impl AsNoNullStr for [u8] {
    fn as_no_null_str(&self) -> Result<&NoNullStr> {
        match memchr(self, 0) {
            Some(_) => Err(error::InvalidArgument),
            _ => Ok(unsafe { NoNullStr::from_bytes_unchecked(self) })
        }
    }
}

impl AsNoNullStr for NoNullStr {
    fn as_no_null_str(&self) -> Result<&NoNullStr> {
        Ok(self)
    }
}

impl AsNoNullStr for [i8] {
    fn as_no_null_str(&self) -> Result<&NoNullStr> {
        let bytes: &[u8] = self.as_ref();
        bytes.as_no_null_str()
    }
}

impl AsNoNullStr for str {
    fn as_no_null_str(&self) -> Result<&NoNullStr> {
        let bytes: &[u8] = self.as_ref();
        bytes.as_no_null_str()
    }
}

impl AsMutNoNullStr for [u8] {
    fn as_mut_no_null_str(&mut self) -> Result<&mut NoNullStr> {
        match memchr(self, 0) {
            Some(_) => Err(error::InvalidArgument),
            _ => Ok(unsafe { NoNullStr::from_mut_bytes_unchecked(self) })
        }
    }
}

impl AsMutNoNullStr for [i8] {
    fn as_mut_no_null_str(&mut self) -> Result<&mut NoNullStr> {
        let bytes: &mut [u8] = self.as_mut();
        bytes.as_mut_no_null_str()
    }
}

impl AsMutNoNullStr for NoNullStr {
    fn as_mut_no_null_str(&mut self) -> Result<&mut NoNullStr> {
        Ok(self)
    }
}
