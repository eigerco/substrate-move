// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

#![cfg_attr(not(feature = "std"), no_std)]

#[macro_use]
extern crate alloc;

#[cfg(test)]
mod tests;

#[cfg(feature = "std")]
pub mod utils;

pub mod natives;

#[cfg(feature = "std")]
pub mod doc;

/// Provides a precompiled bundle of bytecode modules.
pub fn move_stdlib_bundle() -> &'static [u8] {
    include_bytes!("../build/MoveStdlib/bundles/MoveStdlib.mvb")
}
