//! Common code for all CPU impls

pub mod zpu;

extern crate byteorder;
use cpu::byteorder::{ByteOrder, BigEndian};

use super::errors::*;

// Handy aliases
type Byte = u8;

/// States
/// Simply a state.
#[derive(PartialEq, Clone)]
pub enum CPUState {
    Running,
    Waiting,
    Sleeping,
    Stopped,
}

pub trait CPU {
    /// Run a single instruction.
    fn step(&mut self) -> Result<(), Error>;

    /// Get state.
    fn state(&self) -> CPUState;

    /// Start.
    fn start(&mut self) -> Result<(), Error>;
}

// Helpers
#[inline(always)]
fn dis32_be(val: u32) -> [Byte; 4] {
    let mut buf = [0; 4];
    BigEndian::write_u32(&mut buf, val);
    buf
}

#[inline(always)]
fn dis16_be(val: u16) -> [Byte; 2] {
    let mut buf = [0; 2];
    BigEndian::write_u16(&mut buf, val);
    buf
}

/// Combine 4 bytes into a big endian u32.
/*fn comb32_be(a: Byte, b: Byte, c: Byte, d: Byte) -> u32 {
    ((a << 24) | (b << 16)) | ((c << 8) | d)
}*/
#[inline(always)]
fn comb32_be(vals: [Byte; 4]) -> u32 {
    BigEndian::read_u32(&vals)
}

#[inline(always)]
fn comb16_be(vals: [Byte; 2]) -> u16 {
    BigEndian::read_u16(&vals)
}
