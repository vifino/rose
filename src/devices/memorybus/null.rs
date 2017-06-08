//! Null block.
//!
//! Consumes everything.
//! Contains nothing. Forever.

extern crate mem;

use self::mem::errors::*;

use std::usize;

pub struct NullBlock {}

impl mem::MemoryBlock for NullBlock {
    fn get_size(&self) -> mem::Addr {
        usize::MAX
    }

    fn get(&self, _addr: mem::Addr) -> Result<mem::Byte, Error> {
        Ok(0)
    }

    fn set(&mut self, _addr: mem::Addr, _value: mem::Byte) -> Result<(), Error> {
        Ok(())
    }
}

