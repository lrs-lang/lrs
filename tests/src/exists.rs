// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

extern crate linux;

use linux::{file};

fn main() {
    assert_eq!(file::exists("Makefile"), Ok(true));
    assert_eq!(file::exists("doesnotexist"), Ok(false));
}