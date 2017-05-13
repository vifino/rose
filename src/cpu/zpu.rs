//! ZPU! Yay!

// Quick tips:
// - pop = sp + 4
// - push = sp - 4

extern crate mem;

// Handy aliases
type Byte = u8;

pub struct ZPU {
    sp: u32,
    pc: u32,
    last_im: bool,

    mem: Box<mem::MemoryBlock>,
}

// Helpers
fn op_flip_lookup(b: Byte) -> Byte {
    match b {
        0 => 0,
        1 => 2,
        2 => 1,
        3 => 3,
        _ => { panic!("wtf") }
    }
}
fn op_flip_byte(b: Byte) -> Byte {
    let a = op_flip_lookup((b & 0xC0) >> 6);
    let b = op_flip_lookup((b & 0x30) >> 4);
    let c = op_flip_lookup((b & 0x0C) >> 2);
    let d = op_flip_lookup(b & 0x03);
    (a | b << 2) | (c << 4 | d << 6)
}

/// converts u32 to i32.
// I hope this is not endianess dependant. :v
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
    fn set32(&mut self, addr: u32, val: u32) -> Result<(), mem::MemError> {
        let vals = super::dis32_be(val);
        println!("ZPU: set32:");
        for i in 0..4 {
            println!(" - {:#X}: {:#X}", addr + 1, vals[i as usize]);
            self.mem.set((addr + i) as usize, vals[i as usize])?;
        }
        Ok(())
    }
    /// Set a u16 in memory, big endian.
    fn set16(&mut self, addr: u32, val: u16) -> Result<(), mem::MemError> {
        let vals = super::dis16_be(val);
        //self.mem.set(addr as usize, 0)?;
        //self.mem.set((addr + 1) as usize, 0)?;
        self.mem.set(addr as usize, vals[0])?;
        self.mem.set((addr + 1) as usize, vals[1])?;
        //self.mem.set((addr + 2) as usize, vals[0])?;
        //self.mem.set((addr + 3) as usize, vals[1])?;
        Ok(())
    }

    /// Get a u32 in memory, big endian.
    fn get32(&self, addr: u32) -> Result<u32, mem::MemError> {
        let mut vals = [0 as Byte; 4];
        println!("ZPU: get32:");
        for i in 0..4 {
            println!(" - {:#X}", addr + i);
            vals[i as usize] = self.mem.get((addr.wrapping_add(i)) as usize)?;
        }
        let val = super::comb32_be(vals);
        println!("ZPU: get32: val is {:#X}", val);
        Ok(val)
    }

    /// Get a u16 in memory, big endian.
    fn get16(&self, addr: u32) -> Result<u16, mem::MemError> {
        let mut vals = [0 as Byte; 2];
        vals[0] = self.mem.get(addr as usize)?;
        vals[1] = self.mem.get((addr.wrapping_add(1)) as usize)?;
        Ok(super::comb16_be(vals))
    }

    /// Stack push.
    #[inline(always)]
    fn v_push(&mut self, val: u32) -> Result<(), mem::MemError> {
        let newsp = self.sp.wrapping_sub(4);
        self.set32(newsp, val)?;
        println!("ZPU: Pushed {:#X}", val);
        self.sp = newsp;
        Ok(())
    }

    /// Stack pop.
    #[inline(always)]
    fn v_pop(&mut self) -> Result<u32, mem::MemError> {
        let v = self.get32(self.sp)?;
        println!("ZPU: Popped {:#X}", v);
        self.sp = self.sp.wrapping_add(4) & 0xFFFFFFFC;
        Ok(v)
    }
}

impl ZPU {
    pub fn new(mem: Box<mem::MemoryBlock>) -> ZPU {
        ZPU {
            pc: 0,
            sp: 0x80000,

            last_im: false,
            mem: mem,
        }
    }

    /// Run one instruction.
    pub fn step(&mut self) -> Result<(), mem::MemError> {
        // Debug
        println!("");
        println!("ZPU: PC: {:#X} ({})", self.pc, self.pc);
        println!("ZPU: SP: {:#X} ({})", self.sp, self.sp);
        // Get op
        let op = self.mem.get((self.pc) as usize)?;
        println!("ZPU: OP: {:#X}", op);
        let lim = self.last_im;
        self.last_im = false;

        // basic ops
        let sp = self.sp;
        let pc = self.pc;
        let found = match op {
            0x02 => { // PUSHSP
                self.v_push(sp)?;
                self.pc = self.pc.wrapping_add(1);
                true
            },
            0x04 => { // POPPC
                self.pc = self.v_pop()?;
                true
            },
            0x05 => { // ADD
                let a = self.v_pop()?;
                let b = self.v_pop()?;
                self.v_push(a.wrapping_add(b))?;
                self.pc = self.pc.wrapping_add(1);
                true
            },
            0x06 => { // AND
                let a = self.v_pop()?;
                let b = self.v_pop()?;
                self.v_push(a & b)?;
                self.pc = self.pc.wrapping_add(1);
                true 
            },
            0x07 => { // OR
                let a = self.v_pop()?;
                let b = self.v_pop()?;
                self.v_push(a | b)?;
                self.pc = self.pc.wrapping_add(1);
                true 
            },
            0x08 => { // LOAD
                let addr = self.get32(sp)? & 0xFFFFFFFC;
                let val = self.get32(addr)?;
                self.set32(sp, val)?;
                self.pc = self.pc.wrapping_add(1);
                true 
            },
            0x09 => { // NOT
                let v = self.v_pop()?;
                self.v_push(!v)?;
                self.pc = self.pc.wrapping_add(1);
                true 
            },
            0x0A => { // FLIP
                for i in 0..4 {
                    let addr = (sp + i) as usize;
                    let v = self.mem.get(addr)?;
                    self.mem.set(addr, op_flip_byte(v))?;
                }
                self.pc = self.pc.wrapping_add(1);
                true 
            },
            0x0B => { // NOP
                self.pc = self.pc.wrapping_add(1);
                true
            },
            0x0C => { // STORE
                let addr = self.v_pop()? & 0xFFFFFFFC;
                let val = self.v_pop()?;
                self.set32(addr, val)?;
                self.pc = self.pc.wrapping_add(1);
                true 
            },
            0x0D => { // POPSP
                self.sp = self.v_pop()? & 0xFFFFFFFC;
                self.pc = self.pc.wrapping_add(1);
                true
            },
            _ => false,
        };

        if found {
            println!("ZPU: OP is basic.");
            return Ok(());
        }

        // bitfield ops
        if (op & 0x80) == 0x80 { // IM
            self.last_im = true;
            println!("ZPU: OP is IM.");
            self.pc = self.pc.wrapping_add(1);
            let i = (op & 0x7F) as u8;
            println!("ZPU: IM: {:#X}", i);
            if lim {
                println!("ZPU: IM: Last was IM.");
                let tmp = (self.v_pop()? & 0x1FFFFFFF).wrapping_shl(7);
                let val = tmp | (i as u32);
                println!("ZPU: IM: val is {:#X}", val);
                self.v_push(val)?;
                return Ok(());
            }
            println!("ZPU: IM: Last was not an IM.");
            if (i & 0x40) != 0 {
                let val = (i as u32) | 0xFFFFFF80;
                println!("ZPU: IM: val is {:#X}", val);
                self.v_push(val)?;
                return Ok(());
            }
            println!("ZPU: IM: val is {:#X}", i);
            self.v_push(i as u32)?;
            return Ok(());
        }

        let op_e0 = op & 0xE0;
        if op_e0 == 0x40 { // STORESP
            println!("ZPU: OP is STORESP.");
            let i = (op & 0x1F) as u32;
            let bsp = (sp + (i ^ 0x10).wrapping_shl(2)) & 0xFFFFFFFC;
            let val = self.v_pop()?;
            self.set32(bsp, val)?;
            self.pc = self.pc.wrapping_add(1);
            return Ok(());
        }
        if op_e0 == 0x60 { // LOADSP
            println!("ZPU: OP is LOADSP.");
            let i = (op & 0x1F) as u32;
            let addr = (sp + ((i ^ 0x10) << 2)) & 0xFFFFFFFC;
            let val = self.get32(addr)?;
            self.v_push(val)?;
            self.pc = self.pc.wrapping_add(1);
            return Ok(());
        }

        if op_e0 == 0x20 { // EMULATE
            let eop = op & 0x1F;
            //return (self.emulate)(self, op);
            println!("ZPU: OP is emulate.");
            let found = zpu_emulates(self, eop)?;
            if !found {
                self.v_push(pc + 1)?;
                self.pc = (eop << 5) as u32;
            }
            return Ok(());
        }

        if (op & 0xF0) == 0x10 { // ADDSP
            println!("ZPU: OP is ADDSP.");
            let i = (op & 0xF) as u32;
            let addr = (sp + (i << 2)) & 0xFFFFFFFC;
            let val = self.get32(addr)?;
            let pval = self.v_pop()?;
            self.v_push(val.wrapping_add(pval))?;
            self.pc = self.pc.wrapping_add(1);
            return Ok(());
        }

        // Should be error but it isn't right now.
        println!("ZPU: OP not found.");;
        Ok(())
    }
}

/// Emulates.
///
/// ZPU EMULATE operations are optional for the most part, but hardware implementations
/// are obviously quicker than software ones.
/// Apart from that, not all software implements those, relying on hardware implementations
/// instead.
pub fn zpu_emulates(zpu: &mut ZPU, op: Byte) -> Result<bool, mem::MemError> {
    let sp = zpu.sp;
    let pc = zpu.pc;
    println!("ZPU: EMULATE: {:#X}", op);
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
            zpu.v_push(bool2u32(tos < nos))?;
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
            Ok(true)
        },
        17 => { // SUB
            let tos = zpu.v_pop()?;
            let nos = zpu.v_pop()?;
            zpu.v_push(tos.wrapping_sub(nos))?;
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
