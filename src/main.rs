#[macro_use]
mod stack;

use stack::*;

fn main() {
    let mut vm = stack::Machine::new();
    let program = assemble! {
        push 7;
        dup;

    label loop;
        pop;
        shl 1;
        push 5_000;
        blt loop;

        pop;
        exit;
    };
    println!("Result: {:?}", vm.run(&program));
}
