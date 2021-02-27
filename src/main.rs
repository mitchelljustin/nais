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
    assemble! {
    label main;
    local loop_ctr;
    local n;
        extend 2;

        push 20;
        store loop_ctr;

        push 15;
        store n;
    inner cnt_loop;
        load n;
        print;
        muli 2;
        store n;

        load loop_ctr;
        subi 1;
        store loop_ctr;

        load loop_ctr;
        push 0;
        bne cnt_loop;

        exit;
    }
}
fn boneless_chacha20() -> Program {
    assemble! {
    local loop_ctr a;
        frame_start;

        push 32;
        store loop_ctr;

        push 31;
        store a;
    inner cnt_loop;
        load a;
        load a;
        jal qround;
        print;
        store a;
        pop 1;

        load loop_ctr;
        subi 1;
        store loop_ctr;

        load loop_ctr;
        push 0;
        bne cnt_loop;

        frame_end;
        exit;

    label qround;
    arg a b;
    local x;
         frame_start;

         load a;
         shl 8;
         store x;

         load b;
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
    let program = boneless_chacha20();
    let binary = program.as_binary();
    let mut machine = Machine::new();
    machine.load_code(&binary);
    machine.run();
    println!();
    println!("Result: {:?}", machine);
    println!("{}",machine.stack_dump());
}
