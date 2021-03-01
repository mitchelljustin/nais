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
    program_from_asm! {
        label _entry;
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
            addi 1;
            storef val;

            loadf val;
            loadf array;
            loadf index;
            add; // &arr[index]
            store; // arr[index] = val

            loadf index;
            jal increment;
            storef index; // index += 1

            loadf index;
            loadf array_len;
            blt loop; // if index < len goto loop

            end_frame;
            ret;

        label increment;
        arg val;
            start_frame;

            ebreak;

            loadf val;
            addi 1;
            storef val;

            end_frame;
            ret;

        label print_array;
        arg array array_len;
        local index;
            start_frame;

            push 0;
            storef index;

        inner print_loop;
            loadf index;
            print;

            // array[index]
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
    program_from_asm! {
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
    let mut program = program_with_name(&which);
    let binary = {
        match program.assemble() {
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
    machine.attach_debug_info(DebugInfo::from(program));
    machine.verbose = false;
    machine.enable_debugger = true;
    machine.max_cycles = 1_000_000_000;
    machine.copy_code(&binary);
    machine.run();
    println!("{:?}", machine);
}
