use std::fmt::{Debug, Display, Formatter, Result};

use crate::machine::{MachineError, MachineStatus};
use crate::machine::MachineStatus::Stopped;

use super::Machine;
use std::collections::HashMap;

pub struct Op {
    pub name: &'static str,
    pub f: fn(&mut Machine, i32),
}

#[derive(Debug, Clone, Copy)]
pub struct Inst {
    pub addr: Option<i32>,
    pub op: &'static Op,
    pub opcode: u8,
    pub arg: i32,
}

impl Display for Inst {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let addr = match self.addr {
            None => String::new(),
            Some(addr) => format!("{:x}", addr)
        };
        let arg_trunc = self.arg & 0xffffff;
        write!(f, "{} <{:02x}> {:6} {:6x} [{}]",
               addr, self.opcode, self.op.name, arg_trunc, self.arg)
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

pub fn drop(m: &mut Machine, amt: i32) {
    m.drop(amt);
}

pub fn swap(m: &mut Machine, _: i32) {
    if let (Some(top), Some(sec)) = (m.pop(), m.pop()) {
        m.push(top);
        m.push(sec);
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

pub fn breakp(m: &mut Machine, _: i32) {
    println!("<<BREAKPOINT>>\n{}", m.stack_dump());
}

pub fn print(m: &mut Machine, _: i32) {
    if let Some(x) = m.pop() {
        println!("\n>> {:08x} [{}]\n", x, x);
        m.push(x);
    }
}

pub fn exit(m: &mut Machine, code: i32) {
    if code == 0 {
        m.set_status(Stopped);
    } else {
        m.set_status(MachineStatus::Error(MachineError::ProgramExit(code)));
    }
}

pub fn jal(m: &mut Machine, offset: i32) {
    m.pushpc();
    m.jump(offset);
}

pub fn jump(m: &mut Machine, offset: i32) {
    m.jump(offset);
}

pub fn ret(m: &mut Machine, _: i32) {
    if let Some(loc) = m.pop() {
        m.setpc(loc);
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

pub fn extend(m: &mut Machine, amt: i32) {
    m.extend(amt);
}

pub fn invld(m: &mut Machine, _: i32) {
    m.set_status(MachineStatus::Error(MachineError::InvalidInstruction));
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

macro_rules! register_ops {
    ( $($name:ident)+ ) => {
        pub const OPLIST: &'static [Op] = &[
            $(
                Op {
                    name: stringify!($name),
                    f: $name,
                },
            )+
        ];
    }
}

register_ops!(
    invld
    push extend drop swap
    add sub mul div rem and or xor
    addi subi muli divi remi andi ori xori
    sar shl shr
    beq bne blt bge
    load store aload astore
    jump jal ret exit
    breakp print
);

pub const OP_INVALID: &'static Op = &OPLIST[0];

#[derive(Clone)]
pub struct Encoder {
    pub name_to_op: HashMap<&'static str, &'static Op>,
    pub op_to_opcode: HashMap<&'static str, u8>,
    pub opcode_to_op: HashMap<u8, &'static Op>,
}

impl Encoder {
    pub fn new() -> Encoder {
        let mut enc = Encoder{
            name_to_op: HashMap::new(),
            op_to_opcode: HashMap::new(),
            opcode_to_op: HashMap::new(),
        };
        for (i, op) in OPLIST.iter().enumerate() {
            let opcode = i as u8;
            enc.name_to_op.insert(op.name, op);
            enc.op_to_opcode.insert(op.name, opcode);
            enc.opcode_to_op.insert(opcode, op);
        }
        enc
    }

    pub fn make_inst(&self, opname: &str, arg: i32) -> Option<Inst> {
        match self.name_to_op.get(opname) {
            None => return None,
            Some(&op) => {
                let opcode = *self.op_to_opcode.get(opname).unwrap();
                Some(Inst{
                    addr: None,
                    op,
                    opcode,
                    arg,
                })
            }
        }
    }

    pub fn encode(&self, inst: &Inst) -> i32 {
        let opcode = inst.opcode as i32;
        let arg_part = inst.arg & 0xffffff;
        let bin_inst = (opcode << 24) | (arg_part);
        bin_inst
    }

    pub fn decode(&self, bin_inst: i32) -> Option<Inst> {
        let opcode = ((bin_inst >> 24) & 0xff) as u8;
        let mut arg = bin_inst & 0xffffff;
        if arg >> 23 != 0 {
            // sign extend
            arg |= 0xff000000;
        }
        let op = match self.opcode_to_op.get(&opcode) {
            None => return None,
            Some(&op) => op
        };
        Some(Inst{
            addr: None,
            opcode, op, arg
        })
    }
}