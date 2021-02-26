use crate::machine::{MachineError, MachineStatus};
use crate::machine::MachineStatus::Stopped;

use super::Machine;
use std::fmt::{Display, Formatter, Result, Debug};

pub type OpArgs = (i32, i32);

pub struct Op {
    pub name: &'static str,
    pub f: fn(&mut Machine, OpArgs),
}

#[derive(Debug, Clone, Copy)]
pub struct Inst {
    pub op: &'static Op,
    pub args: OpArgs,
}

impl Display for Inst {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:6} {:8x} {:8x}", self.op.name, self.args.0, self.args.1)
    }
}

impl Debug for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("OpFn")
            .field("name", &self.name)
            .finish()
    }
}


pub fn push(m: &mut Machine, (x, _): OpArgs) {
    m.push(x);
}

pub fn pop(m: &mut Machine, (n, _): OpArgs) {
    for _ in 0..n {
        m.pop();
    }
}

pub fn dup(m: &mut Machine, (offset, _): OpArgs) {
    if let Some(x) = m.peek(offset) {
        m.push(x);
    }
}

pub fn put(m: &mut Machine, (offset, _): OpArgs) {
    if let Some(top) = m.pop() {
        m.put(top, offset);
    }
}

pub fn swap(m: &mut Machine, _: OpArgs) {
    if let (Some(top), Some(sec)) = (m.pop(), m.pop()) {
        m.push(top);
        m.push(sec);
    }
}

pub fn breakp(m: &mut Machine, _: OpArgs) {
    println!("BREAKPOINT: {:?}", m);
}

pub fn print(m: &mut Machine, (offset, _): OpArgs) {
    if let Some(x) = m.peek(offset) {
        println!("{} ", x);
    }
}

pub fn printx(m: &mut Machine, (offset, _): OpArgs) {
    if let Some(x) = m.peek(offset) {
        println!("{:08x} ", x);
    }
}

pub fn exit(m: &mut Machine, (code, _): OpArgs) {
    if code == 0 {
        m.status = Stopped;
    } else {
        m.status = MachineStatus::Error(MachineError::ProgramExit(code))
    }
}

pub fn jal(m: &mut Machine, (offset, _): OpArgs) {
    m.push(m.pc);
    m.jump(offset);
}

pub fn ret(m: &mut Machine, _: OpArgs) {
    if let Some(loc) = m.pop() {
        m.setpc(loc);
    }
}

macro_rules! with_overflow {
        ($top:ident $op:tt $arg:ident) => {
            (($top as i64) $op ($arg as i64)) as i32
        };
    }

macro_rules! binary_op_funcs {
        ( $($name:ident ($operator:tt));+; ) => {
            $(
                pub fn $name(m: &mut Machine, _: OpArgs) {
                    if let (Some(top), Some(sec)) = (m.pop(), m.pop()) {
                        m.push(with_overflow!(top $operator sec));
                    }
                }
            )+
        }
    }

binary_op_funcs! {
        add ( + );
        sub ( - );
        mul ( * );
        div ( / );
        rem ( % );
        and ( & );
        or  ( | );
        xor ( ^ );
    }

macro_rules! binary_op_imm_funcs {
        ( $($name:ident ($operator:tt));+; ) => {
            $(
                pub fn $name(m: &mut Machine, (arg, _): OpArgs) {
                    if let Some(top) = m.pop() {
                        m.push(with_overflow!(top $operator arg));
                    }
                }
            )+
        }
    }

binary_op_imm_funcs! {
        addi ( + );
        subi ( - );
        muli ( * );
        divi ( / );
        remi ( % );
        andi ( & );
        ori  ( | );
        xori ( ^ );
    }

macro_rules! branch_cmp_funcs {
        ( $($name:ident ($cmp:tt));+; ) => {
            $(
                pub fn $name(m: &mut Machine, (offset, _): OpArgs) {
                    if let (Some(top), Some(sec)) = (m.pop(), m.pop()) {
                        if sec $cmp top {
                            m.jump(offset);
                        }
                    }
                }
            )+
        }
    }

branch_cmp_funcs! {
        beq ( == );
        bne ( != );
        blt ( < );
        bge ( >= );
    }

pub fn sar(m: &mut Machine, (shamt, _): OpArgs) {
    if let Some(top) = m.pop() {
        let top = top >> shamt;
        m.push(top);
    }
}

macro_rules! logical_shift_funcs {
        ( $($name:ident ($shop:tt));+; ) => {
            $(
                pub fn $name(m: &mut Machine, (shamt, _): OpArgs) {
                    if let Some(top) = m.pop() {
                        let top = top as u32;
                        let top = (top $shop shamt);
                        let top = top as i32;
                        m.push(top);
                    }
                }
            )+
        };
    }

logical_shift_funcs! {
        shl ( << );
        shr ( >> );
    }

pub mod ops {
    use super::Op;

    macro_rules! register_ops {
        ( $($name:ident)+ ) => {
            $(
                #[allow(unused, non_upper_case_globals)]
                pub const $name: &Op = &Op {
                    name: stringify!($name),
                    f: super::$name,
                };
            )+
        }
    }

    register_ops!(
        push pop dup swap put
        exit breakp print printx
        beq bne blt bge
        jal ret
        add sub mul div rem and or  xor
        addi subi muli divi remi andi ori xori
        sar shl shr
    );
}