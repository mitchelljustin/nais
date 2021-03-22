use std::{env, fs, process};

use machine::*;

use crate::assembler::{assemble_file, AssemblyResult};

mod linker;
mod machine;
mod isa;
mod util;
mod mem;
mod assembler;
mod encoder;
mod environment;

fn main() {
    let args: Vec<String> = env::args().collect();
    let filename = match args.get(1) {
        Some(filename) => filename,
        None => {
            eprintln!("format: {} filename", args[0]);
            process::exit(1);
        }
    };
    let mut machine = Machine::new();
    machine.max_cycles = 1_000_000_000;
    machine.debug_on_error = true;

    let extension = filename.split(".").last().unwrap_or("");
    match extension {
        "asm" => {
            let AssemblyResult { binary, debug_info } = match assemble_file(&filename) {
                Err(e) =>
                    panic!("Error assembling {}: \n{}\n", filename, e),
                Ok(r) => r,
            };
            machine.debug_info = debug_info;
            machine.load_code(&binary);

            let program_name = filename.strip_suffix(".asm").unwrap();
            let bin_name = format!("{}.bin", program_name);
            let (_, bin_u8, _) = unsafe { binary.align_to::<u8>() };
            fs::write(bin_name, bin_u8).unwrap();
        }
        "bin" => {
            let binary = fs::read(filename).unwrap();
            let (_, bin_i32, _) = unsafe { binary.align_to::<i32>() };
            machine.load_code(bin_i32);
        }
        _ => panic!("Can only read .bin or .asm files"),
    };

    machine.run();
}
