#![allow(overflowing_literals)]

use std::{env, process};

use machine::*;

use crate::assembler::{AssemblyResult, assemble_file};

#[macro_use]
mod linker;
mod machine;
mod isa;
mod constants;
mod util;
mod mem;
mod assembler;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = match args.get(1) {
        Some(filename) => filename.clone(),
        None => {
            eprintln!("format: {} filename", args[0]);
            process::exit(1);
        }
    };
    let AssemblyResult { binary, debug_info } = match assemble_file(&filename) {
        Err(e) => {
            panic!("Error assembling file: \n{}\n", e);
        }
        Ok(res) => res,
    };
    let mut machine = Machine::new();
    machine.max_cycles = 1_000_000_000;

    machine.enable_debugger = true;
    machine.attach_debug_info(debug_info);

    machine.copy_code(&binary);
    machine.run();
    println!("{:?}", machine);
}
