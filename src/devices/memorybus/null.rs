//! Null block.
//!
//! Consumes everything.
//! Contains nothing. Forever.

extern crate mem;

use std::usize;

pub struct NullBlock {}

impl mem::MemoryBlock for NullBlock {
    fn get_size(&self) -> mem::Addr {
        usize::MAX
    }

    fn get(&self, _addr: mem::Addr) -> Result<mem::Byte, mem::MemError> {
        Ok(0)
    }

    fn set(&mut self, _addr: mem::Addr, _value: mem::Byte) -> Result<(), mem::MemError> {
        Ok(())
    }
}

