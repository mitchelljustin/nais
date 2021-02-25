#[macro_use]
mod stack;

use stack::Op;

fn main() {
    let mut vm = stack::Machine::<i32>::new(64 * 1024);
    let program = assemble! {
        (word i32)
        push 4;
        push 2;
        push 1234123;
        dup; dup;
        push 499213;
        add;
        xor;
        sub;
    };
    println!("Result: {:?}", vm.run(&program));
}
