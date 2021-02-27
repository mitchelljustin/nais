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

pub fn breakp(m: &mut Machine, _: i32) {
    println!("<<BREAKPOINT START>>");
    println!("{}", m.stack_dump());
    println!("<<BREAKPOINT END>>");
}

pub fn print(m: &mut Machine, _: i32) {
    if let Some(x) = m.pop() {
        println!("{:08x} [{}]", x, x);
        m.push(x);
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

macro_rules! register_ops {
    ( $($name:ident)+ ) => {
        pub const OPLIST: &'static [Op] = &[
            Op {
                name: "invalid",
                f: |_, _| panic!("INVALID OP")
            },
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
    push pop extend
    add sub mul div rem and or  xor
    addi subi muli divi remi andi ori xori
    sar shl shr
    beq bne blt bge
    load store
    aload astore setfp
    jal ret
    exit breakp
    print
);

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

    pub fn op_with_name(&self, name: &str) -> &'static Op {
        self.name_to_op.get(name).unwrap()
    }

    pub fn opcode_for_op(&self, op: &Op) -> u8 {
        *self.op_to_opcode.get(op.name).unwrap()
    }

    pub fn op_for_opcode(&self, opcode: u8) -> &'static Op {
        self.opcode_to_op.get(&opcode).unwrap().clone()
    }
}