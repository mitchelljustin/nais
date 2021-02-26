#![allow(overflowing_literals)]

use machine::*;

use crate::assemble::Program;

#[macro_use]
mod assemble;
mod machine;
mod isa;

fn program2() -> Program {
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

fn main() {
    let program = program2();
    println!("Program: \n{:}", program);
    let mut machine = Machine::new(&program);
    machine.run();
    println!("Result: {:?}", machine);
    println!("<<Stack dump>>");
    println!("{}",machine.stack_dump());
}
