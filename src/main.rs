#![allow(overflowing_literals)]

use machine::*;
use crate::assemble::Program;


#[macro_use]
mod assemble;
mod machine;
mod isa;
mod constants;

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
            stfi index;

            push 6;
            stfi x;

        inner loop;
            ldfi x;
            jal mangle;
            stfi x;
            ldfi x;

            ldfi array;
            ldfi index;
            add; // &arr[index]

            storer; // arr[index] = x

            ldfi index;
            addi 1;
            stfi index; // index += 1

            ldfi index;
            ldfi len;
            blt loop; // if index < len goto loop

            end_frame;
            ret;

        label mangle;
        arg x;
            start_frame;

            ldfi x;
            addi 78;
            stfi x;

            end_frame;
            ret;

        label print_array;
        arg array array_len;
        local index;
            start_frame;

            push 0;
            stfi index;

        inner print_loop;
            ldfi index;
            print;

            ldfi index;
            ldfi array;
            add;
            loadr;
            print;

            ldfi index;
            addi 1;
            stfi index;

            ldfi index;
            ldfi array_len;
            blt print_loop;

            end_frame;
            ret;
    }
}

fn main() {
    let binary = {
        let mut program = array_on_stack();
        match program.assemble() {
            Err(errors) => {
                panic!("assembly errors: \n{}\n", errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join("\n"));
            }
            Ok(bin) => {
                println!("Program:\n{}", program);
                bin
            }
        }
    };
    let mut machine = Machine::new();
    machine.verbose = true;
    machine.max_cycles = 1_000_000_000;
    machine.copy_code(&binary);
    machine.run();
    println!();
    println!("Result: {:?}", machine);
    println!("{}", machine.stack_dump());
}
