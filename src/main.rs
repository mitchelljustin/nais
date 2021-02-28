#![allow(overflowing_literals)]

use machine::*;
use crate::assemble::Program;


#[macro_use]
mod assemble;
mod machine;
mod isa;
mod constants;
mod util;

fn array_on_stack() -> Program {
    program_from_asm! {
        local index;
        array state 10;
            start_frame;

            push state_len;
            loadi fp;
            addi state;
            jal fill_array;
            drop 2;

            push state_len;
            loadi fp;
            addi state;
            jal print_array;
            drop 2;

            end_frame;
            push 0;
            ecall exit;

        label fill_array;
        arg array len;
        local index x;
            start_frame;

            push 0;
            storef index;

            push 6;
            storef x;

        inner loop;
            loadf x;
            jal mangle;
            storef x;

            loadf x;
            loadf array;
            loadf index;
            add; // &arr[index]
            store; // arr[index] = x

            loadf index;
            addi 1;
            storef index; // index += 1

            loadf index;
            loadf len;
            blt loop; // if index < len goto loop

            end_frame;
            ret;

        label mangle;
        arg x;
            start_frame;

            ebreak;

            loadf x;
            addi 78;
            storef x;

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

fn main() {
    let mut program = array_on_stack();
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
    machine.attach_debug_info(&program);
    machine.verbose = false;
    machine.max_cycles = 1_000_000_000;
    machine.copy_code(&binary);
    machine.run();
}
