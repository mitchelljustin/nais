use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};

use crate::machine::{MachineError, MachineStatus};
use crate::machine::MachineStatus::Stopped;

use super::Machine;
use crate::mem::addrs;

// --- START OP FUNCTIONS ---

pub fn push(m: &mut Machine, val: i32) {
    let sp = m.getsp();
    m.setsp(sp + 1);
    m.stack_store(sp, val);
}

pub fn pop(m: &mut Machine) -> Option<i32> {
    let newsp = m.getsp() - 1;
    let top = m.stack_load(newsp);
    m.setsp(newsp);
    top
}

pub fn loadi(m: &mut Machine, addr: i32) {
    if let Some(val) = m.stack_load(addr) {
        push(m, val);
    }
}

pub fn storei(m: &mut Machine, addr: i32) {
    if let Some(val) = pop(m) {
        m.stack_store(addr, val);
    }
}

pub fn addsp(m: &mut Machine, offset: i32) {
    let sp = m.getsp();
    m.setsp(sp + offset);
}

pub fn load(m: &mut Machine, offset: i32) {
    if let Some(addr) = pop(m) {
        if let Some(val) = m.stack_load(addr + offset) {
            push(m, val);
        }
    }
}

pub fn store(m: &mut Machine, offset: i32) {
    if let (Some(addr), Some(val)) = (pop(m), pop(m)) {
        m.stack_store(addr + offset, val);
    }
}

fn getfp(m: &mut Machine) -> i32 {
    m.stack_load(addrs::FP).expect("frame pointer invalid")
}

pub fn loadf(m: &mut Machine, offset: i32) {
    let fp = getfp(m);
    if let Some(val) = m.stack_load(fp + offset) {
        push(m, val);
    }
}

pub fn storef(m: &mut Machine, offset: i32) {
    let fp = getfp(m);
    if let Some(val) = pop(m) {
        m.stack_store(fp + offset, val);
    }
}

pub fn print(m: &mut Machine, _: i32) {
    if let Some(x) = pop(m) {
        m.print(x);
    }
}

pub const ENV_CALLS: &[&str] = &[
    "exit",
];

pub fn ecall(m: &mut Machine, callcode: i32) {
    if callcode < 0 || callcode >= ENV_CALLS.len() as i32 {
        m.set_status(MachineStatus::Error(MachineError::NoSuchEnvCall(callcode)));
        return;
    }
    let call_name = ENV_CALLS[callcode as usize];

    match call_name {
        "exit" => {
            match pop(m) {
                Some(0) =>
                    m.set_status(Stopped),
                Some(status) =>
                    m.set_status(MachineStatus::Error(MachineError::ProgramExit(status))),
                None => {}
            }
        }
        _ => {}
    }
}

pub fn ebreak(m: &mut Machine, _: i32) {
    m.breakpoint();
}

pub fn jump(m: &mut Machine, offset: i32) {
    let pc = m.getpc();
    m.setpc(pc + offset);
}

pub fn jal(m: &mut Machine, offset: i32) {
    let pc = m.getpc();
    push(m, pc);
    m.setpc(pc + offset);
}

pub fn ret(m: &mut Machine, _: i32) {
    if let Some(addr) = pop(m) {
        m.setpc(addr);
    }
}

pub fn invald(m: &mut Machine, _: i32) {
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
            pub fn $name(m: &mut Machine, imm: i32) {
                if let (Some(top), Some(sec)) = (pop(m), pop(m)) {
                    let mut res = with_overflow!(top $operator sec);
                    res = with_overflow!(res $operator imm);
                    push(m, res);
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
            pub fn $name(m: &mut Machine, imm: i32) {
                if let Some(top) = pop(m) {
                    push(m, with_overflow!(top $operator imm));
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
                    if let (Some(top), Some(sec)) = (pop(m), pop(m)) {
                        if sec $cmp top {
                            jump(m, offset);
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
    if let Some(top) = pop(m) {
        let top = top >> shamt;
        push(m, top);
    }
}

macro_rules! logical_shift_funcs {
        ( $($name:ident ($shop:tt));+; ) => {
            $(
                pub fn $name(m: &mut Machine, shamt: i32) {
                    if let Some(top) = pop(m) {
                        let top = top as u32;
                        let top = (top $shop shamt);
                        let top = top as i32;
                        push(m, top);
                    }
                }
            )+
        };
    }

logical_shift_funcs! {
    shl ( << );
    shr ( >> );
}

// --- END OP FUNCTIONS ---

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
        write!(f, "{} {:6} {:6x} [{:4}]",
               addr, self.op.name, arg_trunc, self.arg)
    }
}

impl Debug for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("OpFn")
            .field("name", &self.name)
            .finish()
    }
}

macro_rules! register_ops {
    ( $($name:ident)+ ) => {
        pub const OP_LIST: &'static [Op] = &[
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
    invald
    push addsp
    add sub mul div rem and or xor
    addi subi muli divi remi andi ori xori
    sar shl shr
    beq bne blt bge
    load store loadi storei loadf storef
    jump jal ret
    ecall ebreak
    print
);

pub const OP_INVALID: &'static Op = &OP_LIST[0];

#[derive(Clone)]
pub struct Encoder {
    pub name_to_op: HashMap<&'static str, &'static Op>,
    pub op_to_opcode: HashMap<&'static str, u8>,
    pub opcode_to_op: HashMap<u8, &'static Op>,
}

impl Encoder {
    pub fn new() -> Encoder {
        let mut enc = Encoder {
            name_to_op: HashMap::new(),
            op_to_opcode: HashMap::new(),
            opcode_to_op: HashMap::new(),
        };
        for (i, op) in OP_LIST.iter().enumerate() {
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
                Some(Inst {
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
        Some(Inst {
            addr: None,
            opcode,
            op,
            arg,
        })
    }
}