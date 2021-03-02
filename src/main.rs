#![allow(overflowing_literals)]

use std::{env, process};

use machine::*;

use crate::assembler::DebugInfo;
use crate::parse_asm::load_asm_file;

#[macro_use]
mod assembler;
mod machine;
mod isa;
mod constants;
mod util;
mod mem;
mod parse_asm;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = match args.get(1) {
        Some(filename) => filename.clone(),
        None => {
            eprintln!("format: {} filename", args[0]);
            process::exit(1);
        }
    };
    let mut assembler = match load_asm_file(&filename) {
        Err(e) => {
            panic!("Error parsing ASM file: \n{}\n", e);
        },
        Ok(assem) => assem,
    };
    let binary = {
        match assembler.assemble() {
            Err(errors) => {
                panic!("Errors assembling program: \n{}\n", util::dump_errors(&errors));
            }
            Ok(bin) => bin
        }
    };
    let mut machine = Machine::new();
    machine.max_cycles = 1_000_000_000;

    machine.enable_debugger = true;
    machine.attach_debug_info(DebugInfo::from(assembler));

    machine.copy_code(&binary);
    machine.run();
    println!("{:?}", machine);
}
