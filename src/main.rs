#![allow(overflowing_literals)]

use machine::*;

use crate::assemble::Program;

#[macro_use]
mod assemble;
mod machine;
mod isa;
mod constants;

fn boneless_chacha20() -> Program {
    program_from_asm! {
    const c_magic_val 0x8ab3ce;
    const c_init_msg 9408383;

    local ctr msg;
        frame_start;

        push 2;
        store ctr;

        push c_init_msg;
        store msg;
    inner loop;
        load msg;
        jal round;
        print;
        store msg;

        load ctr;
        subi 1;
        store ctr;

        load ctr;
        push 0;
        bne loop;

        frame_end;
        exit;

    label round;
    arg msg;
    local cnt;
        frame_start;

        push 4;
        store cnt;
    inner loop;
        load msg;
        jal qround;
        store msg;

        load cnt;
        subi 1;
        store cnt;
        load cnt;
        push 0;
        bne loop;

        frame_end;
        ret;

    label qround;
    arg msg;
    local x;
        frame_start;

        load msg;
        shl 8;
        store x;

        load msg;
        shr 24;
        load x;
        or;

        load msg;
        xor;

        addi c_magic_val;
        store msg;

        frame_end;
        ret;
    }
}

fn main() {
    let binary = {
        let mut program = boneless_chacha20();
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
    machine.copy_code(&binary);
    machine.run();
    println!();
    println!("Result: {:?}", machine);
    println!("{}", machine.stack_dump());
}
