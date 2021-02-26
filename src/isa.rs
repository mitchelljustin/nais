use std::fmt::{Debug, Display, Formatter, Result};

use crate::machine::{MachineError, MachineStatus};
use crate::machine::MachineStatus::Stopped;

use super::Machine;

pub struct Op {
    pub name: &'static str,
    pub f: fn(&mut Machine, i32),
}

#[derive(Debug, Clone, Copy)]
pub struct Inst {
    pub op: &'static Op,
    pub arg: i32,
}

impl Display for Inst {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{:6} {:8x} [{}]", self.op.name, self.arg, self.arg)
    }
}

impl Debug for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("OpFn")
            .field("name", &self.name)
            .finish()
    }
}


pub fn push(m: &mut Machine, x: i32) {
    m.push(x);
}

pub fn pop(m: &mut Machine, n: i32) {
    for _ in 0..n {
        m.pop();
    }
}

pub fn load(m: &mut Machine, offset: i32) {
    if let Some(x) = m.load(offset) {
        m.push(x);
    }
}

pub fn store(m: &mut Machine, offset: i32) {
    if let Some(top) = m.pop() {
        m.store(top, offset);
    }
}

pub fn swap(m: &mut Machine, _: i32) {
    if let (Some(top), Some(sec)) = (m.pop(), m.pop()) {
        m.push(top);
        m.push(sec);
    }
}

pub fn breakp(m: &mut Machine, _: i32) {
    println!("<<BREAKPOINT>>");
    for (i, x) in m.stack.iter().enumerate() {
        let extra = match i {
            0 => "pc",
            1 => "sp",
            2 => "fp",
            3 => "boundary",
            _ => ""
        };
        println!("{:02x}. {:8x} [{:8}] {}", i, x, x, extra);
    }
    println!("<<BREAKPOINT END>>");
}

pub fn print(m: &mut Machine, offset: i32) {
    if let Some(x) = m.load(offset) {
        println!("{}", x);
    }
}

pub fn printx(m: &mut Machine, offset: i32) {
    if let Some(x) = m.load(offset) {
        println!("{:08x} ", x);
    }
}

pub fn exit(m: &mut Machine, code: i32) {
    if code == 0 {
        m.status = Stopped;
    } else {
        m.status = MachineStatus::Error(MachineError::ProgramExit(code))
    }
}

pub fn jal(m: &mut Machine, offset: i32) {
    m.pushpc();
    m.jump(offset);
}

pub fn ret(m: &mut Machine, _: i32) {
    if let Some(loc) = m.pop() {
        m.setpc(loc - 1);
    }
}

pub fn aload(m: &mut Machine, loc: i32) {
    if let Some(x) = m.load_abs(loc) {
        m.push(x);
    }
}

pub fn astore(m: &mut Machine, loc: i32) {
    if let Some(x) = m.pop() {
        m.store_abs(loc, x);
    }
}

pub fn setfp(m: &mut Machine, _: i32) {
    m.setfp();
}

pub fn extend(m: &mut Machine, amt: i32) {
    m.extend(amt);
}

macro_rules! with_overflow {
        ($top:ident $op:tt $arg:ident) => {
            (($top as i64) $op ($arg as i64)) as i32
        };
    }

macro_rules! binary_op_funcs {
        ( $($name:ident ($operator:tt));+; ) => {
            $(
                pub fn $name(m: &mut Machine, _: i32) {
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
                pub fn $name(m: &mut Machine, arg: i32) {
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
                pub fn $name(m: &mut Machine, offset: i32) {
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

pub fn sar(m: &mut Machine, shamt: i32) {
    if let Some(top) = m.pop() {
        let top = top >> shamt;
        m.push(top);
    }
}

macro_rules! logical_shift_funcs {
        ( $($name:ident ($shop:tt));+; ) => {
            $(
                pub fn $name(m: &mut Machine, shamt: i32) {
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

    macro_rules! register_ops {
        ( $($name:ident)+ ) => {
            $(
                #[allow(unused, non_upper_case_globals)]
                pub const $name: &super::Op = &super::Op {
                    name: stringify!($name),
                    f: super::$name,
                };
            )+
        }
    }

    register_ops!(
        push pop swap extend
        add sub mul div rem and or  xor
        addi subi muli divi remi andi ori xori
        sar shl shr
        beq bne blt bge
        load store
        aload astore setfp
        jal ret
        exit breakp
        print printx
    );
}