//! ZPU test platform/binary.
//!
//! Loads bin at addr 0.
//! Write byte to 0x80000027 to write text to stdout,
//! read from 0x80000031 to read from stdin.

// Yes, we have a lot of uses.
extern crate mem;
extern crate rose;

#[macro_use]
extern crate clap;

use rose::cpu::*;
use rose::cpu::zpu::ZPU;
use rose::bus::BusDevice;
use rose::bus::memorybus::{MemoryBus32be, MemoryBusDevice32be};
use rose::devices::memorybus::sio::SIOTerm;

use rose::errors::*;

use mem::MemoryBlock;
use mem::MemoryCreator;
use mem::std_impls::MemVector;

use std::io::Read;
use std::io::Write;
use std::fs::File;

use clap::{App, Arg};

arg_enum!{
    #[derive(Debug)]
    enum Platforms {
        Phi,
        Zeta
    }
}

fn main() {
    // Arg parsing
    let matches = App::new("rose-zpu")
        .version("0.1")
        .author("Adrian Pistol <vifino@tty.sh>")
        .about("ZPU test binary for ROSE.")
        .arg(Arg::from_usage("<binary> 'The binary to load.'")
             .required(true)
             .index(1))
        .arg(Arg::from_usage("-p, --platform=[PLATFORM] 'The platform to emulate.'")
             .possible_values(&Platforms::variants())
             .takes_value(true))
        .arg(Arg::from_usage("-e, --emulates=[BOOL] 'Use hardware EMULATE implementations'"))
        .get_matches();

    let fname = matches.value_of("binary").unwrap();
    let platform = value_t!(matches.value_of("platform"), Platforms).unwrap_or(Platforms::Phi);
    let use_emulates = value_t!(matches.value_of("emulates"), bool).unwrap_or(true);

    // Platform variables
    let uart = match platform {
        Platforms::Phi =>  0x080a000c,
        Platforms::Zeta => 0x80000024,
    };

    // Device init
    let mut ram = Box::new(MemVector::new(0x80000));
    let sio = Box::new(SIOTerm::new_zpu(uart)); // UART of the ZPU.

    // Load rom.
    let f = File::open(fname).unwrap();
    let mut i = 0;
    for byte in f.bytes() {
        ram.set(i, byte.unwrap()).unwrap();
        i += 1;
    }

    // Set up bus
    let devices: Vec<Box<MemoryBusDevice32be>> = vec![sio, ram];
    let mut membus = Box::new(MemoryBus32be::new(devices));
    membus.init().unwrap();

    // CPU
    let mut cpu = ZPU::new(membus, use_emulates);
    cpu.sp = 0x80000;

    cpu.start().unwrap();

    while cpu.state() == CPUState::Running {
        if let Err(ref e) = cpu.step() {
            ehandle(e);
        }
    }
}

// Generic error_chain error handling
fn ehandle(e: &Error) {
    println!("");
    let stderr = &mut ::std::io::stderr();

    writeln!(stderr, "Error: {}", e).unwrap();

    for e in e.iter().skip(1) {
        writeln!(stderr, "caused by: {}", e).unwrap();
    }

    // The backtrace is not always generated. Try to run this example
    // with `RUST_BACKTRACE=1`.
    if let Some(backtrace) = e.backtrace() {
        writeln!(stderr, "backtrace: {:?}", backtrace).unwrap();
    }
    ::std::process::exit(1);
}
