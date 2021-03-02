#![allow(overflowing_literals)]

use std::{env, process};

use machine::*;

use crate::assemble::DebugInfo;
use crate::parse::load_asm_file;

#[macro_use]
mod assemble;
mod machine;
mod isa;
mod constants;
mod util;
mod mem;
mod parse;

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
        Err(e) => panic!("Error reading ASM file: {:?}", e),
        Ok(assem) => assem,
    };
    let binary = {
        match assembler.assemble() {
            Err(errors) => {
                panic!("Errors assembling program: \n{}\n", errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join("\n"));
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
