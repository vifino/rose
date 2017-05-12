//! ZPU test platform/binary.
//!
//! Loads bin at addr 0.
//! Write byte to 0x80000027 to write text to stdout,
//! read from 0x80000031 to read from stdin.

extern crate mem;
extern crate rose;

use rose::cpu::zpu::ZPU;
use rose::bus::MemoryBus;
use rose::devices::memorybus::sio::SIOTerm;
use rose::devices::memorybus::null::NullBlock;

use mem::MemoryBlock;
use mem::MemoryCreator;
use mem::std_impls::MemVector;

use std::io;
use std::io::Read;
use std::env;
use std::fs::File;

fn main() {
    // Arg parsing
    let args: Vec<_> = env::args().collect();
    if args.len() <= 1 {
        println!("Usage: emu filename.rom");
        ::std::process::exit(1);
    }
    let ref fname = args[1]; // second arg, first is prog.

    // Device init
    let mut ram = Box::new(MemVector::new(0x80000));
    let mut sio = Box::new(SIOTerm::new(0x80000028 + 3, 0x80000024 + 3)); // Serial I/O.
    let mut null = Box::new(NullBlock {}); // Null, so more or less invalid writes don't error. For now.


    // Load rom.
    let mut f = File::open(fname).unwrap();
    let mut i = 0;
    for byte in f.bytes() {
        ram.set(i, byte.unwrap()).unwrap();
        i += 1;
    }

    // Set up bus
    let membus = Box::new(MemoryBus::new(vec![sio, ram]));

    // CPU
    let mut cpu = ZPU::new(membus);

    loop {
        cpu.step().unwrap();
    }
}