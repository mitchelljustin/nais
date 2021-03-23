use std::fs;

use clap::Clap;

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

#[derive(Clap)]
#[clap(version = "1.0", author = "Mitchell Justin")]
struct Opts {
    filename: String,

    #[clap(short, long)]
    debug_on_err: bool,

    #[clap(short, default_value="1000000")]
    max_cycles: usize,
}

fn main() {
    let opts: Opts = Opts::parse();

    let mut machine = Machine::new();
    machine.max_cycles = opts.max_cycles;
    machine.debug_on_error = opts.debug_on_err;

    let filename = opts.filename;
    let extension = filename.split(".").last().unwrap_or("");
    match extension {
        "asm" => assemble_and_load_file(&mut machine, filename),
        "bin" => load_binary(&mut machine, filename),
        _ => panic!("Can only read .bin or .asm files"),
    };

    machine.run();
    if !machine.debug_on_error && machine.status != MachineStatus::Stopped {
        eprintln!("{:?}", machine);
    }
}

fn assemble_and_load_file(machine: &mut Machine, filename: String) {
    let AssemblyResult {
        binary,
        debug_info,
        expanded_source,
    } = match assemble_file(&filename) {
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

    let expanded_name = format!("{}.expanded.asm", program_name);
    fs::write(expanded_name, expanded_source).unwrap();
}

fn load_binary(machine: &mut Machine, filename: String) {
    let binary = fs::read(filename).unwrap();
    let (_, bin_i32, _) = unsafe { binary.align_to::<i32>() };
    machine.load_code(bin_i32);
}
