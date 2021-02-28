#![allow(overflowing_literals)]

use machine::*;

use crate::assemble::Program;

#[macro_use]
mod assemble;
mod machine;
mod isa;
mod constants;

#[allow(dead_code)]
fn boneless_chacha20() -> Program {
    program_from_asm! {
    const c_magic_val 0x8ab3ce;
    const c_init_msg 9408383;

    local ctr msg;
        frame_start;

        push 2;
        store ctr;

        push c_init_msg;
        store msg;
    inner loop;
        load msg;
        jal round;
        print;
        store msg;

        load ctr;
        subi 1;
        store ctr;

        load ctr;
        push 0;
        bne loop;

        frame_end;
        exit;

    label round;
    arg msg;
    local cnt;
        frame_start;

        push 4;
        store cnt;
    inner loop;
        load msg;
        jal qround;
        store msg;

        load cnt;
        subi 1;
        store cnt;
        load cnt;
        push 0;
        bne loop;

        frame_end;
        ret;

    label qround;
    arg msg;
    local x;
        frame_start;

        load msg;
        shl 8;
        store x;

        load msg;
        shr 24;
        load x;
        or;

        load msg;
        xor;

        addi c_magic_val;
        store msg;

        frame_end;
        ret;
    }
}

fn array_on_stack() -> Program {
    program_from_asm! {
        local dummy;
        array state 16;
            frame_start;

            push 16; // len
            ldgi fp;
            push state;
            add;     // &arr
            jal fill_array;
            drop 2;

            frame_end;

            extend 20; // for debugging

            exit;

        label fill_array;
        arg arr len;
        local index x;
            frame_start;

            push 0;
            stfi index;

            push 1;
            stfi x;

        inner loop;
            ldfi arr;
            ldfi index;
            add; // &arr[index]

            ldfi x;
            shl 1;
            addi 1;
            xori 0xf1f1f1;
            stfi x;

            ldfi x;
            stgt; // arr[index] = x

            ldfi index;
            addi 1;
            stfi index; // index += 1

            ldfi index;
            ldfi len;
            blt loop; // if index < len goto loop

            frame_end;
            ret;
    }
}

fn main() {
    let binary = {
        let mut program = array_on_stack();
        match program.assemble() {
            Err(errors) => {
                panic!("assembly errors: \n{}\n", errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join("\n"));
            }
            Ok(bin) => {
                println!("Program:\n{}", program);
                bin
            }
        }
    };
    println!("Binary:\n{}", binary
        .iter()
        .enumerate()
        .map(|(i, inst)| format!("{:04x} {:08x}", i, inst))
        .collect::<Vec<_>>()
        .join("\n"));
    let mut machine = Machine::new();
    machine.copy_code(&binary);
    machine.run();
    println!();
    println!("Result: {:?}", machine);
    println!("{}", machine.stack_dump());
}
