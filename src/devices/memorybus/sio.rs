// Rudimentary "serial" I/O.

extern crate mem;

use self::mem::errors::*;
use super::super::super::bus::memorybus::MemoryBusDevice;
use super::super::super::bus::BusDevice;

use std::io;
use std::io::prelude::*;
use std::cmp;

/// Basic Terminal I/O.
///
/// Mostly for the ZPU.
///
/// To read a char, `get` `addr_read`.
/// To print a char, `set` `addr_write`.
/// Plus a ton more, because ZPU reasons.
pub struct SIOTerm {
    addr_read: mem::Addr,
    addr_read_min: mem::Addr,
    addr_read_max: mem::Addr,
    addr_write: mem::Addr,
    addr_write_min: mem::Addr,
    addr_write_max: mem::Addr,
    min: mem::Addr,
    max: mem::Addr,
}

impl SIOTerm {
    pub fn new(read: mem::Addr, write: mem::Addr) -> SIOTerm {
        SIOTerm {
            addr_read: read,
            addr_read_min: read - 3,
            addr_read_max: read,
            addr_write: write,
            addr_write_min: write - 3,
            addr_write_max: write,
            min: cmp::min(read - 3, write - 3),
            max: cmp::max(read, write),
        }
    }
    pub fn new_zpu(base: mem::Addr) -> SIOTerm {
        let addr_read = base +  3 + 4;
        let addr_write = base + 3;
        SIOTerm {
            addr_read: addr_read,
            addr_read_min: addr_read - 3,
            addr_read_max: addr_read,
            addr_write: addr_write,
            addr_write_min: addr_write - 3,
            addr_write_max: addr_write,
            min: cmp::min(addr_read - 3, addr_write - 3),
            max: cmp::max(addr_read, addr_write),
        }
    }
}

impl mem::MemoryBlock for SIOTerm {
    fn get_size(&self) -> usize {
        self.max // highest address.
    }

    fn set(&mut self, addr: mem::Addr, val: mem::Byte) -> Result<(), Error> {
        if addr == self.addr_write {
            debug!("SIO: got write @ {:#X}: {}", addr, val as char);
            let mut buf = [0 as mem::Byte, 1];
            buf[0] = val;
            return match io::stdout().write(&buf) {
                Ok(_) => Ok(()),
                Err(_) => bail!(ErrorKind::HardwareFault(addr, "SIO device failed to read from stdin."))
            }
        }

        if addr >= self.addr_write_min && addr <= self.addr_write_max {
            return Ok(());
        }

        if addr < self.min {
            bail!(ErrorKind::TooSmall(addr, self.min));
        }
        if addr > self.max {
            bail!(ErrorKind::TooBig(addr, self.max));
        }

        bail!(ErrorKind::InvalidAddr(addr))
    }

    fn get(&self, addr: mem::Addr) -> Result<mem::Byte, Error> {
        if addr == self.addr_read {
            debug!("SIO: got read @ {:#X}", addr);
            let mut buf = [0 as mem::Byte; 1];
            return match io::stdin().read(&mut buf) {
                Ok(_) => {
                    debug!("SIO: read char {}", buf[0] as char);
                    Ok(buf[0])
                },
                Err(_) => {
                    debug!("SIO: hw fail");
                    bail!(ErrorKind::HardwareFault(addr, "SIO device failed to read from stdin."))
                },
            }
        } else if addr == self.addr_write - 1 {
            debug!("SIO: got read @ {:#X}, returning 1", addr);
            return Ok(1) // fake remaining bytes.
        } else if addr == self.addr_read - 1 {
            debug!("SIO: got read @ {:#X}, returning 1", addr);
            return Ok(1) // results in the same as being or'd with 0x100 for a LOAD
        }

        if addr >= self.addr_read_min && addr <= self.addr_read_max {
            debug!("SIO: got read @ {:#X}, returning 0", addr);
            return Ok(0 as mem::Byte);
        }
        if addr >= self.addr_write_min && addr <= self.addr_write_max {
            debug!("SIO: got read @ {:#X}, returning 0", addr);
            return Ok(0);
        }

        if addr < self.min {
            bail!(ErrorKind::TooSmall(addr, self.min));
        }
        if addr > self.max {
            bail!(ErrorKind::TooBig(addr, self.max));
        }

        bail!(ErrorKind::InvalidAddr(addr))
    }
}

impl BusDevice for SIOTerm {}
impl MemoryBusDevice for SIOTerm {}
