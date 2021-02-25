#[macro_use]
mod riscv;

fn main() {
    let mut vm = riscv::Machine::new(64 * 1024);
    run_asm! { in vm
        addi t0 zero 44;
        add t0 t0 t0;
        addi t1 zero 900;
        or t0 t0 t1;
    }
    println!("VM: {:?}", vm)
}
