#[macro_use]
mod stack;

use stack::*;

fn main() {
    let mut vm = stack::Machine::new();
    let program = assemble! {
        push 10;
        dup;

    label loop;
        pop;
        push 26;
        mul;
        push 1_000_000;
        blt loop;

        pop;
        exit;
    };
    println!("Result: {:?}", vm.run(&program));
}
