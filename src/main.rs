#![allow(overflowing_literals)]

use machine::*;

use crate::assemble::Program;

#[macro_use]
mod assemble;
mod machine;
mod isa;
mod constants;

#[allow(dead_code)]
fn calc152n() -> Program {
    program_from_asm! {
    label main;
    local loop_ctr;
    local n;
        extend 2;

        push 20;
        store loop_ctr;

        push 15;
        store n;
    inner loop;
        load n;
        print;
        muli 2;
        store n;

        load loop_ctr;
        subi 1;
        store loop_ctr;

        load loop_ctr;
        push 0;
        bne loop;

        exit;
    }
}
fn boneless_chacha20() -> Program {
    program_from_asm! {
    local loop_ctr a;
        frame_start;

        push 2;
        store loop_ctr;

        push 31;
        store a;
    inner loop;
        load a;
        jal round;
        print;
        store a;

        load loop_ctr;
        subi 1;
        store loop_ctr;

        load loop_ctr;
        push 0;
        bne loop;

        frame_end;
        exit;

    label round;
    arg a;
    local cnt;
        frame_start;

        push 4;
        store cnt;
    inner loop;
        load a;
        jal qround;
        store a;

        load cnt;
        subi 1;
        store cnt;
        load cnt;
        push 0;
        bne loop;

        frame_end;
        ret;

    label qround;
    arg a;
    local x;
         frame_start;

         load a;
         shl 8;
         store x;

         load a;
         shr 24;
         load x;
         or;

         load a;
         xor;

         addi 0xf389ab71;
         store a;

         frame_end;
         ret;
    }
}

fn main() {
    let mut program = boneless_chacha20();
    let binary = match program.assemble() {
        Err(err) => {
            panic!("assembly errors: {:?}", err);
        },
        Ok(bin) => bin
    };
    let mut machine = Machine::new();
    machine.load_code(&binary);
    machine.run();
    println!();
    println!("Result: {:?}", machine);
    println!("{}",machine.stack_dump());
}
