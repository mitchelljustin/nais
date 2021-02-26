use crate::machine::{MachineError, MachineStatus};
use crate::machine::MachineStatus::Stopped;

use super::Machine;
use std::fmt::{Display, Formatter, Result, Debug};

pub type OpArg = i32;

pub struct Op {
    pub name: &'static str,
    pub f: fn(&mut Machine, OpArg),
}

#[derive(Debug, Clone, Copy)]
pub struct Inst {
    pub op: &'static Op,
    pub arg: OpArg,
}

impl Display for Inst {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:6} {:8x}", self.op.name, self.arg)
    }
}

impl Debug for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("OpFn")
            .field("name", &self.name)
            .finish()
    }
}


pub fn push(m: &mut Machine, x: OpArg) {
    m.push(x);
}

pub fn pop(m: &mut Machine, n: OpArg) {
    for _ in 0..n {
        m.pop();
    }
}

pub fn dup(m: &mut Machine, offset: OpArg) {
    if let Some(x) = m.peek(offset) {
        m.push(x);
    }
}

pub fn put(m: &mut Machine, offset: OpArg) {
    if let Some(top) = m.pop() {
        m.put(top, offset);
    }
}

pub fn swap(m: &mut Machine, _: OpArg) {
    if let (Some(top), Some(sec)) = (m.pop(), m.pop()) {
        m.push(top);
        m.push(sec);
    }
}

pub fn breakp(m: &mut Machine, _: OpArg) {
    println!("BREAKPOINT: {:?}", m);
}

pub fn print(m: &mut Machine, offset: OpArg) {
    if let Some(x) = m.peek(offset) {
        println!("{} ", x);
    }
}

pub fn printx(m: &mut Machine, offset: OpArg) {
    if let Some(x) = m.peek(offset) {
        println!("{:08x} ", x);
    }
}

pub fn exit(m: &mut Machine, code: OpArg) {
    if code == 0 {
        m.status = Stopped;
    } else {
        m.status = MachineStatus::Error(MachineError::ProgramExit(code))
    }
}

pub fn jal(m: &mut Machine, offset: OpArg) {
    m.push(m.pc);
    m.jump(offset);
}

pub fn ret(m: &mut Machine, _: OpArg) {
    if let Some(loc) = m.pop() {
        m.setpc(loc);
    }
}

pub fn load(m: &mut Machine, loc: OpArg) {
    if let Some(x) = m.load(loc) {
        m.push(x);
    }
}

pub fn store(m: &mut Machine, loc: OpArg) {
    if let Some(x) = m.pop() {
        m.store(loc, x);
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
                pub fn $name(m: &mut Machine, _: OpArg) {
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
                pub fn $name(m: &mut Machine, arg: OpArg) {
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
                pub fn $name(m: &mut Machine, offset: OpArg) {
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

pub fn sar(m: &mut Machine, shamt: OpArg) {
    if let Some(top) = m.pop() {
        let top = top >> shamt;
        m.push(top);
    }
}

macro_rules! logical_shift_funcs {
        ( $($name:ident ($shop:tt));+; ) => {
            $(
                pub fn $name(m: &mut Machine, shamt: OpArg) {
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
        load store
        add sub mul div rem and or  xor
        addi subi muli divi remi andi ori xori
        sar shl shr
    );
}