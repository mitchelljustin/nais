#![allow(unused)]

use std::{env, process};

use machine::*;

use crate::assembler::{assemble_file, AssemblyResult};

mod linker;
mod machine;
mod isa;
mod util;
mod mem;
mod assembler;
mod encoder;
mod tokenizer;
mod parser;
mod ast;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = match args.get(1) {
        Some(filename) => filename,
        None => {
            eprintln!("format: {} filename", args[0]);
            process::exit(1);
        }
    };
    let AssemblyResult { binary, debug_info } = match assemble_file(&filename) {
        Err(e) => {
            eprintln!("Error assembling file: \n{}\n", e);
            process::exit(1);
        }
        Ok(res) => res,
    };
    let mut machine = Machine::new();
    machine.max_cycles = 1_000_000_000;

    machine.debug_info = debug_info;
    machine.debug_on_error = true;

    machine.load_code(&binary);
    machine.run();
}
