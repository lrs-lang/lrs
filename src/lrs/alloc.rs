// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Memory allocation

pub use lrs_alloc::{
    MAX_SIZE, empty_ptr, Allocator, Heap, FbHeap, Libc, NoMem, Bda,
};

#[cfg(jemalloc)]
pub use lrs_alloc::{
    JeMalloc,
};