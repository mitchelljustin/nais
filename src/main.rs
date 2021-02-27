#![allow(overflowing_literals)]

use machine::*;

use crate::assemble::Program;

#[macro_use]
mod assemble;
mod machine;
mod isa;

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
    label main;
    local loop_ctr;
    local a;
        extend 2;

        push 20;
        store loop_ctr;

        push 15;
        store a;
    inner cnt_loop;
        load a;
        jal qround;
        print;
        store a;

        load loop_ctr;
        subi 1;
        store loop_ctr;

        load loop_ctr;
        push 0;
        bne cnt_loop;

        exit;
    label qround;
    arg a; local x;
         aload fp;
         aload sp;
         astore fp;
         extend 1;

         load a;
         muli 2;
         store a;

         pop 1;
         astore fp;
         ret;
    }
}

fn main() {
    let program = boneless_chacha20();
    println!("Program: \n{:}", program);
    let mut machine = Machine::new(&program);
    machine.run();
    println!("Result: {:?}", machine);
    println!("<<Stack dump>>");
    println!("{}",machine.stack_dump());
}
