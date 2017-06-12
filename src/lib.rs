//! rose
//! Emulator library.

// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

pub mod errors;

// Our modules
#[macro_use]
mod macros;

pub mod bus;
pub mod cpu;
pub mod devices;
