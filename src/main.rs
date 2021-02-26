#![allow(overflowing_literals)]

#[macro_use]
mod assemble;
mod machine;
mod isa;

use machine::*;
use crate::assemble::Program;

#[allow(dead_code)]
fn program1() -> Program {
assemble! {
        push 7;
        dup;

    label loop;
        shl 1;
        dup;
        push 5_000;
        blt loop;

        pop;
        exit;
    }
}

fn program2() -> Program {
    assemble! {
    label start;
        push 25; // ctr
        push 4; // acc

    label loop;
        call f;

        dup 1; // top = ctr
        subi 1; // ctr -= 1
        dup; // sec, top = ctr, ctr
        put 2; // *ctr = pop()
        push 0; // sec, top = ctr, 0
        bne loop;

        printx;
        print;
        pop 2;
        exit;

    label f;
        // [arg retaddr]
        dup 1; // [arg retaddr acc]

        printx;
        dup;        // [a a]
        dup;        // [a a a]
        shl 8;      // [a a a<<]
        swap;       // [a a<< a]
        shr 24;     // [a a<< a>>]
        or;         // [a a<<|a>>]
        xor;        // [a^(a<<|a>>)]
        addi 0xfff82913; // [arg retaddr acc']
        put 1; // [acc' retaddr]

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
