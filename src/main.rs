#![allow(overflowing_literals)]

use machine::*;

use crate::assemble::Program;

#[macro_use]
mod assemble;
mod machine;
mod isa;

fn program2() -> Program {
    assemble! {
    label start;
    var ctr;
        push 25; // ctr

    var acc;
        push 4; // acc

    label loop;
        jal f;

        load ctr;
        subi 1;
        store ctr;

        load ctr;
        push 0;
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
