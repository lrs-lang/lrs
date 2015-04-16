// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![crate_name = "linux_ty_two"]
#![crate_type = "lib"]
#![feature(plugin, no_std, optin_builtin_traits)]
#![plugin(linux_core_plugin)]
#![no_std]

#[macro_use]
extern crate linux_core as core;
extern crate linux_ty_one as ty_one;
extern crate linux_error as error;
extern crate linux_alloc as alloc;
extern crate linux_fmt as fmt;
extern crate linux_io as io;

pub mod linux {
    pub use ::fmt::linux::*;
    pub mod vec {
        pub use ::vec::{Vec};
    }
}

pub mod vec;
pub mod rc;