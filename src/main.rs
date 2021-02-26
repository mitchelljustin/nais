#[macro_use]
mod stack;

use stack::*;

fn program2() -> Program {
    return assemble! {
        push 15; // counter
        push 3; // acc
        print 1;
        print;

    label loop;
        print;

        // op
        muli 3;

        dup 1; // top = counter
        subi 1; // top -= 1
        dup; // sec, top = counter, counter
        put 2; // *counter = pop()
        push 0; // sec, top = counter, 0
        bne loop;

        exit;
    }
}

#[allow(dead_code)]
fn program1() -> Program { // TODO: fix
    assemble! {
        push 7;
        dup;

    label loop;
        shl 1;
        push 5_000;
        blt loop;

        pop;
        exit;
    }
}


fn main() {
    let mut vm = Machine::new();
    let program = program2();
    println!("Result: {:?}", vm.run(&program));
}
