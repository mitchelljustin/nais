use std::fmt::{Debug, Display, Formatter};

use crate::machine::{MachineError, MachineStatus};
use crate::machine::MachineStatus::Stopped;
use crate::mem::addrs;

use super::Machine;
use std::fmt;

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

pub mod env_call {
    use std::io;
    use std::io::Write;

    use crate::machine::MachineError::EnvCallErr;
    use crate::mem::segs;

    use super::*;

    pub enum RetCode {
        OK = 0,
        AddressOutOfBounds = 1,
        NotImplemented = 2,
        IOError = 3,
    }

    fn exit(m: &mut Machine) -> i32 {
        match pop(m) {
            Some(0) =>
                m.set_status(Stopped),
            Some(status) =>
                m.set_status(MachineStatus::Error(MachineError::ProgramExit(status))),
            None => {}
        };
        RetCode::OK as i32
    }


    fn write(m: &mut Machine) -> i32 {
        if let (Some(fd), Some(buf), Some(buf_len)) = (pop(m), pop(m), pop(m)) {
            let addr_range = buf..(buf + buf_len);
            if buf < segs::ADDR_SPACE.start || buf + buf_len > segs::ADDR_SPACE.end {
                return RetCode::AddressOutOfBounds as i32;
            }
            let data: Vec<u8> = addr_range
                .map(|addr| m.unsafe_load(addr) as u8)
                .collect();
            let result = match fd {
                1 => io::stdout().write(&data),
                2 => io::stderr().write(&data),
                _ => {
                    m.set_error(EnvCallErr(format!("cannot write to fd: {}", fd)));
                    return RetCode::NotImplemented as i32;
                },
            };
            match result {
                Err(err) => {
                    m.set_error(EnvCallErr(format!("IO error: {}", err)));
                    RetCode::IOError as i32
                }
                Ok(_) => RetCode::OK as i32,
            }
        } else {
            1
        }
    }

    macro_rules! def_env_call_list {
        ( $($name:ident)+ ) => {
            pub const LIST: &[(fn(&mut Machine) -> i32, &'static str)] = &[
                $(
                    ($name, stringify!($name)),
                )+
            ];
        }
    }

    def_env_call_list![
        exit
        write
    ];
}

pub fn ecall(m: &mut Machine, callcode: i32) {
    if callcode < 0 || callcode >= env_call::LIST.len() as i32 {
        m.set_status(MachineStatus::Error(MachineError::NoSuchEnvCall(callcode)));
        return;
    }
    let (env_call_func, _) = env_call::LIST[callcode as usize];
    let retval = env_call_func(m);
    push(m, retval);
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
            #[allow(unused)]
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
            #[allow(unused)]
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
            #[allow(unused)]
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
    ble ( <= );
    bgt ( > );
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
                    let top1 = top as u32;
                    let top2 = (top1 $shop shamt);
                    let top3 = top2 as i32;
                    push(m, top3);
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
    pub func: fn(&mut Machine, i32),
}

#[derive(Debug, Clone, Copy)]
pub struct Inst {
    pub addr: Option<i32>,
    pub op: &'static Op,
    pub opcode: u8,
    pub arg: i32,
}

impl Display for Inst {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpFn")
            .field("name", &self.name)
            .finish()
    }
}

macro_rules! def_op_list {
    ( $($name:ident)+ ) => {
        pub const OP_LIST: &'static [Op] = &[
            $(
                Op {
                    name: stringify!($name),
                    func: $name,
                },
            )+
        ];
    }
}

def_op_list![
    invald
    push addsp
    add sub mul div rem and or xor
    addi subi muli divi remi andi ori xori
    sar shl shr
    beq bne blt bge
    load store loadi storei loadf storef
    jump jal ret
    ecall ebreak
];

pub const OP_INVALID: &'static Op = &OP_LIST[0];