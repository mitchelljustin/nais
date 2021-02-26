#![allow(overflowing_literals)]

use machine::*;

use crate::assemble::Program;

#[macro_use]
mod assemble;
mod machine;
mod isa;

fn program2() -> Program {
    assemble! {
    local counter;
    local result;
        extend 2;

        push 1;
        store counter;

        push 5;
        store result;
    inner loop;
        load result;
        jal quarter_round;
        store result;

        load counter;
        subi 1;
        store counter;

        load counter;
        push 0;
        bne loop;

        load result;
        printx;
        pop;

        pop 2;
        exit;

    label quarter_round;
    arg a;
        aload fp;
        setfp;

        load a;
        addi 1;
        store a;

        astore fp;

        breakp;
        ret;
    }
}

fn main() {
    let program = program2();
    println!("Program: \n{:}", program);
    let mut machine = Machine::new(&program);
    machine.run();
    println!("Result: {:?}", machine);
}
