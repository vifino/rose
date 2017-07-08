//! ZPU! Yay!

// Quick tips:
// - pop = sp + 4
// - push = sp - 4

extern crate mem;

use super::super::errors::*;
use super::CPUState as State;

// Handy aliases
type Byte = u8;

pub struct ZPU {
    pub sp: u32,
    pub pc: u32,
    last_im: bool,

    state: State,

    pub mem: Box<mem::MemoryBlock32be>,
    hwemus: bool,
}

// Helpers
#[inline]
fn flipb(b: Byte) -> Byte {
    let mut v = b as u32;
    // from https://graphics.stanford.edu/~seander/bithacks.html#ReverseParallel
    // swap odd and even bits
    v = ((v >> 1) & 0x55555555) | ((v & 0x55555555) << 1);
    // swap consecutive pairs
    v = ((v >> 2) & 0x33333333) | ((v & 0x33333333) << 2);
    // swap nibbles ...
    (((v >> 4) & 0x0F0F0F0F) | ((v & 0x0F0F0F0F) << 4)) as Byte
}

fn flip32(val: u32) -> u32 {
    let mut alav = super::dis32_be(val);
    alav.reverse();
    for i in 0..4 {
        alav[i] = flipb(alav[i]);
    }
    super::comb32_be(alav)
}

/// converts u32 to i32.
// I hope this is not endianess dependant. :v
#[inline(always)]
fn u2i32(v: u32) -> i32 {
    if (v & 0x80000000) != 0 {
        return (v - 0x10000000) as i32;
    }
    v as i32
}

#[inline(always)]
fn bool2u32(v: bool) -> u32 {
    if v {
        return 1;
    }
    0
}

// L/R shifter based on signedness
fn gshift32(val: u32, shift: i32) -> u32 {
    if shift == 0 { return val; }
    if (shift >= 32) || (shift <= -32) { return 0; }
    if shift > 0 { // positive
        return val << shift;
    }
    val >> -shift
}
fn agshift32(val: u32, shift: i32) -> u32 {
    let tmp = gshift32(val, shift);
    if (val & 0x80000000) != 0 {
        return (gshift32(0xFFFFFFFF, shift) ^ 0xFFFFFFFF) | val;
    }
    tmp
}

impl ZPU {
    /// Set a u32 in memory, big endian.
    fn set32(&mut self, addr: u32, val: u32) -> Result<(), Error> {
        debug!("ZPU: set32: {:#X} to {:#X}", addr, val);
        self.mem.set32be(addr as usize, val).chain_err(|| "in ZPU internal set32")
    }
    /// Set a u16 in memory, big endian.
    fn set16(&mut self, addr: u32, val: u16) -> Result<(), Error> {
        let vals = super::dis16_be(val);
        //self.mem.set(addr as usize, 0)?        //self.mem.set((addr + 1) as usize, 0)?;
        self.mem.set(addr as usize, vals[0]).chain_err(|| "unable to complete set16, failure to set byte 1")?;
        self.mem.set((addr + 1) as usize, vals[1]).chain_err(|| "unable to complete set16, failure to set byte 2")?;
        //self.mem.set((addr + 2) as usize, vals[0])?;
        //self.mem.set((addr + 3) as usize, vals[1])?;
        Ok(())
    }

    /// Get a u32 in memory, big endian.
    fn get32(&self, addr: u32) -> Result<u32, Error> {
        let val = self.mem.get32be(addr as usize).chain_err(|| "in ZPU internal get32")?;
        debug!("ZPU: get32: val is {:#X}", val);
        Ok(val)
    }

    /// Get a u16 in memory, big endian.
    fn get16(&self, addr: u32) -> Result<u16, Error> {
        let mut vals = [0 as Byte; 2];
        vals[0] = self.mem.get(addr as usize).chain_err(|| "unable to complete get16, failure to get byte 1")?;
        vals[1] = self.mem.get((addr.wrapping_add(1)) as usize).chain_err(|| "unable to complete get16, failure to get byte 2")?;
        Ok(super::comb16_be(vals))
    }

    /// Stack push.
    #[inline(always)]
    fn v_push(&mut self, val: u32) -> Result<(), Error> {
        let newsp = self.sp.wrapping_sub(4);
        self.set32(newsp, val).chain_err(|| "unable to complete v_push")?;
        debug!("ZPU: Pushed {:#X}", val);
        self.sp = newsp;
        Ok(())
    }

    /// Stack pop.
    #[inline(always)]
    fn v_pop(&mut self) -> Result<u32, Error> {
        let v = self.get32(self.sp).chain_err(|| "unable to complete v_pop")?;
        debug!("ZPU: Popped {:#X}", v);
        self.sp = self.sp.wrapping_add(4) & 0xFFFFFFFC;
        Ok(v)
    }
}

impl ZPU {
    pub fn new(mem: Box<mem::MemoryBlock32be>, emus: bool) -> ZPU {
        ZPU {
            pc: 0,
            sp: 0,
            last_im: false,

            state: State::Stopped,

            mem: mem,
            hwemus: emus,
        }
    }
}

impl super::CPU for ZPU {
    /// Run one instruction.
    fn step(&mut self) -> Result<(), Error> { // TODO: make it use a custom error type or something.
        // Bail out if not running
        if self.state != State::Running {
            bail!(ErrorKind::CPUNotRunning);
        }

        // Debug
        debug!("");
        debugf!("{} ({:x}/{:x}) :", self.pc, self.sp, match self.get32(self.sp) { Ok(val) => val, Err(_) => 0});

        // Get op
        let op = self.mem.get((self.pc) as usize).chain_err(|| "ZPU failed to fetch OP")?;
        let lim = self.last_im;
        self.last_im = false;

        // basic ops
        let sp = self.sp;
        let pc = self.pc;
        let found = match op {
            0x00 => { // breakpoint
                debug!("BREAKPOINT");
                self.pc = self.pc.wrapping_add(1);
                self.state = State::Sleeping;
                true
            },
            // 0x01 = SHIFTLEFT?
            0x02 => { // PUSHSP
                debug!("PUSHSP");
                self.v_push(sp)?;
                self.pc = self.pc.wrapping_add(1);
                true
            },
            // 0x03 = POPINT?
            0x04 => { // POPPC
                debug!("POPPC");
                self.pc = self.v_pop()?;
                true
            },
            0x05 => { // ADD
                debug!("ADD");
                let a = self.v_pop()?;
                let b = self.v_pop()?;
                self.v_push(a.wrapping_add(b))?;
                self.pc = self.pc.wrapping_add(1);
                true
            },
            0x06 => { // AND
                debug!("AND");
                let a = self.v_pop()?;
                let b = self.v_pop()?;
                self.v_push(a & b)?;
                self.pc = self.pc.wrapping_add(1);
                true
            },
            0x07 => { // OR
                debug!("OR");
                let a = self.v_pop()?;
                let b = self.v_pop()?;
                self.v_push(a | b)?;
                self.pc = self.pc.wrapping_add(1);
                true
            },
            0x08 => { // LOAD
                debug!("LOAD");
                let addr = self.get32(sp)? & 0xFFFFFFFC;
                let val = self.get32(addr)?;
                self.set32(sp, val)?;
                self.pc = self.pc.wrapping_add(1);
                true
            },
            0x09 => { // NOT
                debug!("NOT");
                let v = self.v_pop()?;
                self.v_push(!v)?;
                self.pc = self.pc.wrapping_add(1);
                true
            },
            0x0A => { // FLIP
                debug!("FLIP");
                let val = self.get32(sp)?;
                self.set32(sp, flip32(val))?;
                self.pc = self.pc.wrapping_add(1);
                true
            },
            0x0B => { // NOP
                debug!("NOP");
                self.pc = self.pc.wrapping_add(1);
                true
            },
            0x0C => { // STORE
                debug!("STORE");
                let addr = self.v_pop()? & 0xFFFFFFFC;
                let val = self.v_pop()?;
                self.set32(addr, val)?;
                self.pc = self.pc.wrapping_add(1);
                true
            },
            0x0D => { // POPSP
                debug!("POPSP");
                self.sp = self.v_pop()? & 0xFFFFFFFC;
                self.pc = self.pc.wrapping_add(1);
                true
            },
            // 0x0E = IPSUM?
            // 0x0F = SNCPY?
            _ => false,
        };

        if found {
            return Ok(());
        }

        // bitfield ops
        if (op & 0x80) == 0x80 { // IM
            self.last_im = true;
            self.pc = self.pc.wrapping_add(1);
            let i = (op & 0x7F) as u8;
            debug!("IM {}", i);
            if lim {
                debug!("ZPU: IM: Last was IM.");
                let tmp = (self.v_pop()? & 0x1FFFFFFF).wrapping_shl(7);
                let val = tmp | (i as u32);
                debug!("ZPU: IM: val is {:#X}", val);
                self.v_push(val)?;
                return Ok(());
            }
            debug!("ZPU: IM: Last was not an IM.");
            if (i & 0x40) != 0 {
                let val = (i as u32) | 0xFFFFFF80;
                debug!("ZPU: IM: val is {:#X}", val);
                self.v_push(val)?;
                return Ok(());
            }
            debug!("ZPU: IM: val is {:#X}", i);
            self.v_push(i as u32)?;
            return Ok(());
        }

        let op_e0 = op & 0xE0;
        if op_e0 == 0x40 { // STORESP
            let i = (((op ^ 0x10) & 0x1F) as u32).wrapping_shl(2);
            debug!("STORESP {}", i);
            let bsp = sp.wrapping_add(i) & 0xFFFFFFFC;
            let val = self.v_pop()?;
            self.set32(bsp, val)?;
            self.pc = self.pc.wrapping_add(1);
            return Ok(());
        }
        if op_e0 == 0x60 { // LOADSP
            let i = (((op ^ 0x10) & 0x1F) as u32).wrapping_shl(2);
            debug!("LOADSP {}", i);
            let addr = sp.wrapping_add(i) & 0xFFFFFFFC;
            let val = self.get32(addr)?;
            self.v_push(val)?;
            self.pc = self.pc.wrapping_add(1);
            return Ok(());
        }

        if op_e0 == 0x20 { // EMULATE
            let eop = op & 0x1F;
            //return (self.emulate)(self, op);
            debug!("EMULATE {}/{}", eop, eop | 0x20);
            let found = match self.hwemus {
                true => zpu_emulates(self, eop)?,
                false => false,
            };
            if !found {
                self.v_push(pc + 1)?;
                self.pc = (eop as u32) << 5
            }
            return Ok(());
        }

        if (op & 0xF0) == 0x10 { // ADDSP
            let i = (op & 0x0F) as u32;
            debug!("ADDSP {}", i);
            let addr = sp.wrapping_add(i.wrapping_shl(2)) & 0xFFFFFFFC;
            let val = self.get32(addr)?;
            let pval = self.v_pop()?;
            self.v_push(val.wrapping_add(pval))?;
            self.pc = self.pc.wrapping_add(1);
            return Ok(());
        }

        self.state = State::Stopped;
        bail!("ZPU OP not implemented: {:#X}", op)
    }

    // State stuff

    // Get state
    fn state(&self) -> State {
        self.state.clone()
    }

    // Start
    fn start(&mut self) -> Result<(), Error> {
        self.state = State::Running;
        Ok(())
    }
}

/// Emulates.
///
/// ZPU EMULATE operations are optional for the most part, but hardware implementations
/// are obviously quicker than software ones.
/// Apart from that, not all software implements those, relying on hardware implementations
/// instead.
pub fn zpu_emulates(zpu: &mut ZPU, op: Byte) -> Result<bool, Error> {
    let sp = zpu.sp;
    let pc = zpu.pc;
    debug!("ZPU: EMULATE: {:#X}", op);
    match op {
        2 => { // LOADH
            let addr = zpu.get32(sp)?;
            let val = zpu.get16(addr)?;
            zpu.set32(sp, val as u32)?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        3 => { // STOREH
            let addr = zpu.v_pop()?;
            let val = zpu.v_pop()? as u16;
            zpu.set16(addr, val)?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        4 => { // LESSTHAN
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push(bool2u32(u2i32(tos) < u2i32(nos)))?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        5 => { // LESSTHANEQUAL
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push(bool2u32(u2i32(tos) <= u2i32(nos)))?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        6 => { // ULESSTHAN
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push(bool2u32(tos < nos))?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        7 => { // ULESSTHANEQUAL
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push(bool2u32(tos <= nos))?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },

        8 => { // SWAP, maaaybe???
            let tos = zpu.get32(sp)?;
            let nosp = sp.wrapping_add(4) & 0xFFFFFFFC;
            let nos = zpu.get32(nosp)?;
            zpu.set32(sp, nos)?;
            zpu.set32(nosp, tos)?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },

        9 => { // SLOWMULT
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push(tos * nos)?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        10 => { // LSHIFTRIGHT
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push(gshift32(nos, -u2i32(tos)))?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        11 => { // ASHIFTLEFT
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push(agshift32(nos, u2i32(tos)))?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        12 => { // ASHIFTRIGHT
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push(agshift32(nos, -u2i32(tos)))?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        // 13: CALL
        14 => { // EQ
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push(bool2u32(tos == nos))?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        15 => { // NEQ
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push(bool2u32(tos != nos))?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        16 => { // NEG
            let tos = zpu.get32(sp)?;
            zpu.set32(sp, -u2i32(tos) as u32)?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        17 => { // SUB
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push(nos.wrapping_sub(tos))?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        18 => { // XOR
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push(tos ^ nos)?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        19 => { // LOADB
            let addr = zpu.get32(sp)?;
            let val = zpu.mem.get(addr as usize)?;
            zpu.set32(sp, val as u32)?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        20 => { // STOREB
            let addr = zpu.v_pop()?;
            let val = zpu.v_pop()? as Byte;
            zpu.mem.set(addr as usize, val)?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        21 => { // DIV
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push((u2i32(tos) / u2i32(nos)) as u32)?; // might be incorrect due to rounding
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        22 => { // MOD
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push((u2i32(tos) % u2i32(nos)) as u32)?; // might be incorrect
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        23 => { // EQBRANCH
            let branch = zpu.pc.wrapping_add(zpu.v_pop()?);
            let v = zpu.v_pop()?;
            if v == 0 {
                zpu.pc = branch;
                return Ok(true);
            }
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        24 => { // NEQBRANCH
            let branch = zpu.pc.wrapping_add(zpu.v_pop()?);
            let v = zpu.v_pop()?;
            if v != 0 {
                zpu.pc = branch;
                return Ok(true);
            }
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        25 => { // POPPCREL
            let tos = zpu.v_pop()?;
            zpu.pc = pc.wrapping_add(tos);
            Ok(true)
        },
        // 26: CONFIG
        // 27: PUSHPC
        // 28: SYSCALL
        29 => { // PUSHSPADD
            let tos = zpu.get32(sp)?;
            zpu.set32(sp, (tos.wrapping_shl(2).wrapping_add(sp)) & 0xFFFFFFFC)?;
            zpu.pc = zpu.pc.wrapping_add(1);
            Ok(true)
        },
        // 30: HALFMULT, multiply halves? NYI.
        31 => { // CALLPCREL
            let tos = zpu.get32(sp)?;
            let routinep = zpu.pc.wrapping_add(tos);
            zpu.set32(sp, pc.wrapping_add(1))?;
            zpu.pc = routinep;
            Ok(true)
        },
        _ => Ok(false),
    }
}
