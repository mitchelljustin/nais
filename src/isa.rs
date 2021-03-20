use std::fmt::{Debug, Display, Formatter};
use std::fmt;

use crate::machine::{MachineError, MachineStatus};
use crate::machine::MachineStatus::Stopped;
use crate::mem::addrs;

use super::Machine;

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
    let val = m.unsafe_load(addr);
    push(m, val);
}

pub fn storei(m: &mut Machine, addr: i32) {
    if let Some(val) = pop(m) {
        m.unsafe_store(addr, val);
    }
}

pub fn addsp(m: &mut Machine, delta: i32) {
    let sp = m.getsp();
    m.setsp(sp + delta);
}

pub fn load(m: &mut Machine, offset: i32) {
    if let Some(addr) = pop(m) {
        let val = m.unsafe_load(addr + offset);
        push(m, val);
    }
}

pub fn store(m: &mut Machine, offset: i32) {
    if let (Some(addr), Some(val)) = (pop(m), pop(m)) {
        m.unsafe_store(addr + offset, val);
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

pub fn loadr(m: &mut Machine, offset: i32) {
    let pc = m.getpc();
    loadi(m, pc + offset);
}

pub fn storer(m: &mut Machine, offset: i32) {
    let pc = m.getpc();
    storei(m, pc + offset);
}

pub mod env_call {
    use std::io;
    use std::io::{Read, Write};

    use crate::mem::segs;

    use super::*;

    pub enum RetCode {
        UnknownError = -1,
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
            if let Err(code) = bounds_check(buf, buf_len) {
                return code as i32;
            }
            let data: Vec<u8> = (buf..(buf + buf_len))
                .map(|addr| m.unsafe_load(addr) as u8)
                .collect();
            let mut out: Box<dyn io::Write> = match fd {
                1 => Box::new(io::stdout()),
                2 => Box::new(io::stderr()),
                _ => return RetCode::NotImplemented as i32,
            };
            let result = out.write(&data);
            out.flush().unwrap();
            match result {
                Err(_) => RetCode::IOError as i32,
                Ok(_) => RetCode::OK as i32,
            }
        } else {
            RetCode::UnknownError as i32
        }
    }

    fn bounds_check(buf: i32, buf_len: i32) -> Result<(), RetCode> {
        if buf < segs::ADDR_SPACE.start || buf + buf_len > segs::ADDR_SPACE.end {
            return Err(RetCode::AddressOutOfBounds);
        }
        Ok(())
    }

    fn read(m: &mut Machine) -> i32 {
        if let (Some(fd), Some(buf), Some(buf_len)) = (pop(m), pop(m), pop(m)) {
            if let Err(code) = bounds_check(buf, buf_len) {
                return code as i32;
            }
            let mut data = vec![0; buf_len as usize];
            let result = match fd {
                1 => io::stdin().read(&mut data),
                _ => return RetCode::NotImplemented as i32,
            };
            if let Err(_) = result {
                return RetCode::IOError as i32;
            }
            for (addr, val) in (buf..(buf + buf_len)).zip(data) {
                m.stack_store(addr, val as i32);
            }
            RetCode::OK as i32
        } else {
            RetCode::UnknownError as i32
        }
    }

    macro_rules! def_env_call_list {
        ( $($name:ident)+ ) => {
            pub const CALL_LIST: &[(fn(&mut Machine) -> i32, &'static str)] = &[
                $(
                    ($name, stringify!($name)),
                )+
            ];
        }
    }

    def_env_call_list![
        exit
        write
        read
    ];
}

pub fn ecall(m: &mut Machine, callcode: i32) {
    if callcode < 0 || callcode >= env_call::CALL_LIST.len() as i32 {
        m.set_status(MachineStatus::Error(MachineError::NoSuchEnvCall(callcode)));
        return;
    }
    let (env_call_func, _) = env_call::CALL_LIST[callcode as usize];
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

pub fn sar(m: &mut Machine, _: i32) {
    if let (Some(shamt), Some(val)) = (pop(m), pop(m)) {
        let val = val >> shamt;
        push(m, val);
    }
}

pub fn sari(m: &mut Machine, shamt: i32) {
    if let Some(top) = pop(m) {
        let top = top >> shamt;
        push(m, top);
    }
}

macro_rules! logical_shift_imm_funcs {
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

logical_shift_imm_funcs! {
    shli ( << );
    shri ( >> );
}

macro_rules! logical_shift_funcs {
    ( $($name:ident ($shop:tt));+; ) => {
        $(
            pub fn $name(m: &mut Machine, _: i32) {
                if let (Some(shamt), Some(val)) = (pop(m), pop(m)) {
                    let val1 = val as u32;
                    let val2 = val1 $shop shamt;
                    let val3 = val2 as i32;
                    push(m, val3);
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
            Some(addr) => format!("{:x} ", addr)
        };
        let arg_trunc = self.arg & 0xffffff;
        write!(f, "{}{:6} {:6x} [{:4}]",
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
    add sub mul div rem and or xor sar shl shr
    addi subi muli divi remi andi ori xori sari shli shri
    beq bne blt bge bgt ble
    load store loadi storei loadf storef loadr storer
    jump jal ret
    ecall ebreak
];

pub const OP_INVALID: &'static Op = &OP_LIST[0];
