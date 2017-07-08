// mem integration
// memory bus is a bus containing memory device,
// which implement the MemoryBusDevice trait,
// which is just mem::MemoryBlock plus a little more.
// The bus itself also implements the same thing!
// You could chain it if you wanted to!
// Or, well, treat it like a normal mem::MemoryBlock!
// This time, it's a special 32 bit variant.
// Yay!

use std::cmp;

//use errors::*;
use errors::Error as RError;
extern crate mem;
use self::mem::errors::{Error, ErrorKind};

pub trait MemoryBusDevice: mem::MemoryBlock + super::BusDevice {}
pub trait MemoryBusDevice32be: mem::MemoryBlock32be + super::BusDevice {}
pub trait MemoryBusDevice32le: mem::MemoryBlock32le + super::BusDevice {}

pub struct MemoryBus32be {
    devices: Vec<Box<MemoryBusDevice32be>>
}
pub struct MemoryBus32le {
    devices: Vec<Box<MemoryBusDevice32le>>
}

impl MemoryBus32be {
    pub fn new(devs: Vec<Box<MemoryBusDevice32be>>) -> MemoryBus32be {
        MemoryBus32be {
            devices: devs,
        }
    }
}
impl MemoryBus32le {
    pub fn new(devs: Vec<Box<MemoryBusDevice32le>>) -> MemoryBus32le {
        MemoryBus32le {
            devices: devs,
        }
    }
}

impl super::BusDevice for MemoryBus32be {
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
impl super::BusDevice for MemoryBus32le {
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

fn ehandlefatal(err: Error, pos: usize) -> Result<(), Error> {
    match err {
        Error(ErrorKind::ReadOnly(_, _), _) => false,
        Error(ErrorKind::NoData(_), _) => false,
        Error(ErrorKind::TooBig(_, _), _) => false,
        Error(ErrorKind::TooSmall(_, _), _) => false,
        Error(ErrorKind::NotImplemented, _) => false,
        Error(ErrorKind::NotApplicable(_), _) => false,
        Error(ErrorKind::InvalidAddr(_), _) => false,
        // TODO: make this work:
        //Error(ErrorKind::EndianessHelperFail(_, _, _), _) => { return ehandlefatal(err.1.next_error.unwrap(), pos) },
        //Error(ErrorKind::EndianessHelperFail(_, _, _), _) => false,
        _ => bail!("non-ignorable error in bus from device {}: {:?}", pos, err),
    };
    Ok(())
}

impl mem::MemoryBlock for MemoryBus32be {
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
                Err(err) => {
                    ehandlefatal(err, pos)?;
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
                Err(err) => {
                    ehandlefatal(err, pos)?;
                },
            }
            pos += 1;
        }
        // Err(mem::MemError::HardwareFault { at: addr, reason: "All devices returned faults." })
        bail!("receiving hard failures from all bus devices, tried {} devices in total", pos-1)

    }

    fn delete(&mut self, from: mem::Addr, to: mem::Addr) -> Result<(), Error> {
        let mut pos = 1;
        for dev in self.devices.iter_mut() {
            let res = dev.delete(from, to);
            match res {
                Ok(()) => (),
                Err(err) => {
                    ehandlefatal(err, pos)?;
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
                        _ => bail!("non-ignorable error in bus from device {}", pos),
                    };
                },
            }
            pos += 1;
        }
        Ok(())
    }
}

impl mem::MemoryBlock32be for MemoryBus32be {
    fn set32be(&mut self, addr: mem::Addr, val: u32) -> Result<(), Error> {
        let mut pos = 1;
        for dev in self.devices.iter_mut() {
            let res = dev.set32be(addr, val);
            match res {
                Ok(()) => (),
                Err(err) => {
                    ehandlefatal(err, pos)?;
                },
            }
            pos += 1;
        }
        Ok(())
    }

    fn get32be(&self, addr: mem::Addr) -> Result<u32, Error> {
        let mut pos = 1;
        for dev in self.devices.iter() {
            let res = dev.get32be(addr);
            match res {
                Ok(val) => { return Ok(val); },
                Err(err) => {
                    ehandlefatal(err, pos)?;
                },
            }
            pos += 1;
        }
        // Err(mem::MemError::HardwareFault { at: addr, reason: "All devices returned faults." })
        bail!("receiving hard failures from all bus devices, tried {} devices in total", pos-1)

    }
}
impl mem::MemoryBlock for MemoryBus32le {
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
                Err(err) => {
                    ehandlefatal(err, pos)?;
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
                Err(err) => {
                    ehandlefatal(err, pos)?;
                },
            }
            pos += 1;
        }
        // Err(mem::MemError::HardwareFault { at: addr, reason: "All devices returned faults." })
        bail!("receiving hard failures from all bus devices, tried {} devices in total", pos-1)

    }

    fn delete(&mut self, from: mem::Addr, to: mem::Addr) -> Result<(), Error> {
        let mut pos = 1;
        for dev in self.devices.iter_mut() {
            let res = dev.delete(from, to);
            match res {
                Ok(()) => (),
                Err(err) => {
                    ehandlefatal(err, pos)?;
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
                        _ => bail!("non-ignorable error in bus from device {}", pos),
                    };
                },
            }
            pos += 1;
        }
        Ok(())
    }
}

impl mem::MemoryBlock32le for MemoryBus32le {
    fn set32le(&mut self, addr: mem::Addr, val: u32) -> Result<(), Error> {
        let mut pos = 1;
        for dev in self.devices.iter_mut() {
            let res = dev.set32le(addr, val);
            match res {
                Ok(()) => (),
                Err(err) => {
                    ehandlefatal(err, pos)?;
                },
            }
            pos += 1;
        }
        Ok(())
    }

    fn get32le(&self, addr: mem::Addr) -> Result<u32, Error> {
        let mut pos = 1;
        for dev in self.devices.iter() {
            let res = dev.get32le(addr);
            match res {
                Ok(val) => { return Ok(val); },
                Err(err) => {
                    ehandlefatal(err, pos)?;
                },
            }
            pos += 1;
        }
        // Err(mem::MemError::HardwareFault { at: addr, reason: "All devices returned faults." })
        bail!("receiving hard failures from all bus devices, tried {} devices in total", pos-1)

    }
}

// Impls
impl super::BusDevice for mem::MemoryBlock {}
impl super::BusDevice for mem::MemoryBlock32be {}
impl super::BusDevice for mem::MemoryBlock32le {}

impl MemoryBusDevice for mem::MemoryBlock {}
impl MemoryBusDevice32be for mem::MemoryBlock32be {}
impl MemoryBusDevice32le for mem::MemoryBlock32le {}

impl super::BusDevice for mem::std_impls::MemVector {}
impl MemoryBusDevice for mem::std_impls::MemVector {}
impl MemoryBusDevice32be for mem::std_impls::MemVector {}
impl MemoryBusDevice32le for mem::std_impls::MemVector {}
