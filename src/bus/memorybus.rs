// mem integration
// memory bus is a bus containing memory device,
// which implement the MemoryBusDevice trait,
// which is just mem::MemoryBlock plus a little more.
// The bus itself also implements the same thing!
// You could chain it if you wanted to!
// Or, well, treat it like a normal mem::MemoryBlock!
// Yay!

use std::cmp;

//use errors::*;
use errors::Error as RError;
extern crate mem;
use self::mem::errors::{Error, ErrorKind};

pub trait MemoryBusDevice: mem::MemoryBlock + super::BusDevice {}

pub struct MemoryBus {
    devices: Vec<Box<MemoryBusDevice>>
}

impl MemoryBus {
    pub fn new(devs: Vec<Box<MemoryBusDevice>>) -> MemoryBus {
        MemoryBus {
            devices: devs,
        }
    }
}

impl super::BusDevice for MemoryBus {
    fn tick(&mut self) {
        for dev in self.devices.iter_mut() {
            dev.tick()
        }
    }

    fn init(&mut self) -> Result<(), RError> {
        for dev in self.devices.iter_mut() {
            match dev.init() {
                Ok(()) => (),
                Err(err) => bail!(err),
            }
        }
        Ok(())
    }
}

impl mem::MemoryBlock for MemoryBus {
    fn get_size(&self) -> usize {
        let mut acc: usize = 0;
        for dev in self.devices.iter() {
            acc = cmp::max(acc, dev.get_size());
        }
        acc
    }

    fn set(&mut self, addr: mem::Addr, val: mem::Byte) -> Result<(), Error> {
        let mut pos = 1;
        for dev in self.devices.iter_mut() {
            let res = dev.set(addr, val);
            match res {
                Ok(()) => (),
                Err(Error(err, _)) => {
                    match err {
                        ErrorKind::ReadOnly(_, _) => false,
                        ErrorKind::TooBig(_, _) => false,
                        ErrorKind::TooSmall(_, _) => false,
                        ErrorKind::NotImplemented => false,
                        ErrorKind::NotApplicable(_) => false,
                        ErrorKind::InvalidAddr(_) => false,
                        _ => bail!("non-ignorable error in bus at pos {}", pos),
                    };
                },
            }
            pos += 1;
        }
        Ok(())
    }

    fn get(&self, addr: mem::Addr) -> Result<mem::Byte, Error> {
        let mut pos = 1;
        for dev in self.devices.iter() {
            let res = dev.get(addr);
            match res {
                Ok(val) => { return Ok(val); },
                Err(Error(err, _)) => {
                    match err {
                        ErrorKind::NoData(_) => false,
                        ErrorKind::TooBig(_, _) => false,
                        ErrorKind::TooSmall(_, _) => false,
                        ErrorKind::NotImplemented => false,
                        ErrorKind::NotApplicable(_)=> false,
                        ErrorKind::InvalidAddr(_) => false,
                        _ => bail!("non-ignorable error in bus at pos {}", pos),
                    };
                },
            }
            pos += 1;
        }
        // Err(mem::MemError::HardwareFault { at: addr, reason: "All devices returned faults." })
        bail!("all devices returned faults in bus, tried {} devices", pos-1)
        
    }

    fn delete(&mut self, from: mem::Addr, to: mem::Addr) -> Result<(), Error> {
        let mut pos = 1;
        for dev in self.devices.iter_mut() {
            let res = dev.delete(from, to);
            match res {
                Ok(()) => (),
                Err(Error(err, _)) => {
                    match err {
                        ErrorKind::ReadOnly(_, _) => false,
                        ErrorKind::TooBig(_, _) => false,
                        ErrorKind::TooSmall(_, _) => false,
                        ErrorKind::NotImplemented => false,
                        ErrorKind::NotApplicable(_) => false,
                        ErrorKind::InvalidAddr(_) => false,
                        _ => bail!("non-ignorable error in bus at pos {}", pos),
                    };
                },
            }
            pos += 1;
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), Error> {
        let mut pos = 1;
        for dev in self.devices.iter_mut() {
            let res = dev.flush();
            match res {
                Ok(()) => (),
                Err(Error(err, _)) => {
                    match err {
                        ErrorKind::NotImplemented => false,
                        ErrorKind::NotApplicable(_) => false,
                        _ => bail!("non-ignorable error in bus at pos {}", pos),
                    };
                },
            }
            pos += 1;
        }
        Ok(())
    }
}

// Impls
impl super::BusDevice for mem::MemoryBlock {}
impl MemoryBusDevice for mem::MemoryBlock {}

impl super::BusDevice for mem::std_impls::MemVector {}
impl MemoryBusDevice for mem::std_impls::MemVector {}
