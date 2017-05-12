//! Bus stuff

use std::cmp;

extern crate mem;

// Common things
pub trait BusDevice {
    /// Do whatever in a clock cycle.
    fn tick(&mut self) {}

    /// Initialize.
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }
}

// mem integration
// memory bus is a bus containing memory device,
// which implement the MemoryBusDevice trait,
// which is just mem::MemoryBlock plus a little more.
// The bus itself also implements the same thing!
// You could chain it if you wanted to!
// Or, well, treat it like a normal mem::MemoryBlock!
// Yay!
pub trait MemoryBusDevice: BusDevice + mem::MemoryBlock {}

pub struct MemoryBus {
    devices: Vec<Box<mem::MemoryBlock>>
}

impl MemoryBus {
    pub fn new(devs: Vec<Box<mem::MemoryBlock>>) -> MemoryBus {
        MemoryBus {
            devices: devs,
        }
    }
}

/*impl BusDevice for MemoryBus {
    fn tick(&mut self) {
        for dev in self.devices.iter_mut() {
            dev.tick()
        }
    }

    fn init(&mut self) -> Result<(), String> {
        for dev in self.devices.iter_mut() {
            match dev.init() {
                Ok(()) => (),
                Err(str) => return Err(str),
            }
        }
        Ok(())
    }
}*/

impl mem::MemoryBlock for MemoryBus {
    fn get_size(&self) -> usize {
        let mut acc: usize = 0;
        for dev in self.devices.iter() {
            acc = cmp::max(acc, dev.get_size());
        }
        acc
    }

    fn set(&mut self, addr: mem::Addr, val: mem::Byte) -> Result<(), mem::MemError> {
        for dev in self.devices.iter_mut() {
            match dev.set(addr, val) {
                Ok(()) => (),
                Err(err) => {
                    let doerror = match err {
                        mem::MemError::ReadOnly { at: _, globally: _ } => false,
                        mem::MemError::TooBig { given: _, max: _ } => false,
                        mem::MemError::TooSmall { given: _, min: _ } => false,
                        mem::MemError::NotImplemented => false,
                        mem::MemError::NotApplicable { at: _ }=> false,
                        mem::MemError::InvalidAddr { addr: _ }=> false,
                        _ => true,
                    };
                    if doerror {
                        return Err(err);
                    }
                },
            }
        }
        Ok(())
    }

    fn get(&self, addr: mem::Addr) -> Result<mem::Byte, mem::MemError> {
        for dev in self.devices.iter() {
            match dev.get(addr) {
                Ok(val) => { return Ok(val); },
                Err(err) => {
                    let doerror = match err {
                        mem::MemError::NoData { at: _ } => false,
                        mem::MemError::TooBig { given: _, max: _ } => false,
                        mem::MemError::TooSmall { given: _, min: _ } => false,
                        mem::MemError::NotImplemented => false,
                        mem::MemError::NotApplicable { at: _ }=> false,
                        mem::MemError::InvalidAddr { addr: _ }=> false,
                        _ => true,
                    };
                    if doerror {
                        return Err(err);
                    }
                },
            }
        }
        Err(mem::MemError::HardwareFault { at: addr, reason: "All devices returned faults." })
    }

    fn delete(&mut self, from: mem::Addr, to: mem::Addr) -> Result<(), mem::MemError> {
        for dev in self.devices.iter_mut() {
            match dev.delete(from, to) {
                Ok(()) => (),
                Err(err) => {
                    let doerror = match err {
                        mem::MemError::ReadOnly { at: _, globally: _ } => false,
                        mem::MemError::TooBig { given: _, max: _ } => false,
                        mem::MemError::TooSmall { given: _, min: _ } => false,
                        mem::MemError::NotImplemented => false,
                        mem::MemError::NotApplicable { at: _ }=> false,
                        mem::MemError::InvalidAddr { addr: _ }=> false,
                        _ => { true }
                    };
                    if doerror {
                        return Err(err);
                    }
                },
            }
        }
        Ok(())
    }

    fn flush(&mut self) -> Result<(), mem::MemError> {
        for dev in self.devices.iter_mut() {
            match dev.flush() {
                Ok(()) => (),
                Err(err) => {
                    let doerror = match err {
                        mem::MemError::NotImplemented => false,
                        mem::MemError::NotApplicable { at: _ } => false,
                        _ => true,
                    };
                    if doerror {
                        return Err(err);
                    }
                },
            }
        }
        Ok(())
    }
}