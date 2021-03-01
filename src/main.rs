#![allow(overflowing_literals)]

use std::env;

use machine::*;

use crate::assemble::{Assembler, DebugInfo};

#[macro_use]
mod assemble;
mod machine;
mod isa;
mod constants;
mod util;

fn array_on_stack() -> Assembler {
    inline_assembler! {
        label main;
        array state 10;
            start_frame;

            push state_len;
            loadi fp;
            addi state;
            jal fill_array;
            addsp -2;

            push state_len;
            loadi fp;
            addi state;
            jal print_array;
            addsp -2;

            end_frame;
            push 0;
            ecall exit;

        label fill_array;
        arg array array_len;
        local index val;
            start_frame;

            push 0;
            storef index;

            push 0;
            storef val;

        inner loop;
            loadf val;
            loadf array;
            loadf index;
            add;
            store;

            loadf val;
            jal increment;
            storef val;

            loadf index;
            addi 1;
            storef index;

            loadf index;
            loadf array_len;
            blt loop; // if index < len goto loop

            end_frame;
            ret;

        label increment;
        arg val;
            start_frame;

            loadf val;
            addi 1;
            storef val;

            end_frame;
            ret;

        label print_array;
        arg array array_len;
        local index;
            start_frame;
            ebreak;

            push 0;
            storef index;

        inner print_loop;
            loadf index;
            loadf array;
            add;
            load;
            print;

            loadf index;
            addi 1;
            storef index;

            loadf index;
            loadf array_len;
            blt print_loop;

            end_frame;
            ret;
    }
}

fn test_debugger() -> Assembler {
    inline_assembler! {
        start_frame;

        push 0;
        jal nolocals;
        addsp -1;

        end_frame;
        push 0;
        ecall exit;

    label nolocals;
        arg val;

        start_frame;

        loadf val;
        addi 1;
        storef val;

        end_frame;
        ret;
    }
}

fn program_with_name(name: &str) -> Assembler {
    match name {
        "deb" => test_debugger(),
        "array" => array_on_stack(),
        _ => array_on_stack(),
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let which = args.get(1).cloned().unwrap_or("".to_string());
    let mut assembler = program_with_name(&which);
    let binary = {
        match assembler.assemble() {
            Err(errors) => {
                panic!("assembly errors: \n{}\n", errors
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
