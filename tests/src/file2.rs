// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![feature(plugin, no_std)]
#![plugin(linux_core_plugin)]
#![no_std]

#[macro_use] extern crate linux;

use linux::file::{info, File};

fn main() {
    let info1 = File::current_dir().info().unwrap();
    let info2 = File::open_read(".").unwrap().info().unwrap();
    let info3 = info(".").unwrap();

    assert!(info1 == info2);
    assert!(info1 == info3);
}
