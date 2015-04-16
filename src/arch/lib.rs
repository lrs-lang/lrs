// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

#![crate_name = "linux_arch"]
#![crate_type = "lib"]
#![feature(plugin, no_std, asm)]
#![plugin(linux_core_plugin)]
#![no_std]

#[macro_use]
extern crate linux_core as core;
extern crate linux_cty;
extern crate linux_w_syscall;
extern crate linux_atomic;

pub use linux_cty as cty;
pub use linux_w_syscall as syscall;

pub mod atomic {
    pub use ::linux_atomic::*;
    pub type AtomicCInt = AtomicI32;
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
pub fn spin() {
    unsafe { asm!("pause" : : : "memory"); }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
pub fn spin() {
    atomic::fence_seqcst();
}