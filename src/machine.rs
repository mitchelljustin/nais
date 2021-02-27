use std::fmt;
use std::fmt::{Debug, Formatter, Write};
use std::iter;

use MachineStatus::*;

use crate::constants::{BOUNDARY_ADDR, CODE_MAX_LEN, CODE_START, FP_ADDR, INIT_STACK, MAX_CYCLES, PC_ADDR, SP_ADDR};
use crate::isa::{Encoder, Inst};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MachineError {
    EmptyStackPop,
    CodeSegFault,
    InvalidInstruction,
    CannotDecodeInstruction(i32),
    StackIndexOutOfBounds,
    StackIndexNegative,
    ProgramExit(i32),
    MaxCyclesReached,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MachineStatus {
    Idle,
    Running,
    Stopped,
    Error(MachineError),
}

pub struct Machine {
    code_mem: Vec<i32>,
    pub stack: Vec<i32>,
    pub status: MachineStatus,
    pub ncycles: usize,
    encoder: Encoder
}

impl Debug for Machine {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Machine")
            .field("status", &self.status)
            .field("ncycles", &self.ncycles)
            .finish()
    }
}


impl Machine {
    pub fn new() -> Machine {
        Machine {
            code_mem: vec![0; CODE_MAX_LEN],
            stack: Vec::from(INIT_STACK),
            status: Idle,
            ncycles: 0,
            encoder: Encoder::new(),
        }
    }

    pub fn load_code(&mut self, bin_code: &[i32]) {
        for (i, inst) in bin_code.iter().enumerate() {
            self.code_mem[i] = *inst;
        }
    }

    pub fn run(&mut self) {
        self.status = Running;
        while self.status == Running {
            self.cycle();
        }
    }

    fn pc(&mut self) -> &mut i32 {
        return &mut self.stack[PC_ADDR as usize]
    }

    fn sp(&mut self) -> &mut i32 {
        return &mut self.stack[SP_ADDR as usize]
    }

    fn update_sp(&mut self) {
        *self.sp() = self.stack.len() as i32
    }

    fn fp(&mut self) -> &mut i32 {
        return &mut self.stack[FP_ADDR as usize]
    }

    pub fn pop(&mut self) -> Option<i32> {
        match self.stack.pop() {
            None => {
                self.status = Error(MachineError::EmptyStackPop);
                return None;
            }
            Some(x) => {
                self.update_sp();
                Some(x)
            }
        }
    }

    pub fn push(&mut self, x: i32) {
        self.stack.push(x);
        self.update_sp();
    }

    fn stack_offset_ref(&mut self, offset: i32) -> Option<&mut i32> {
        let fp = *self.fp();
        self.stack_ref(fp + offset)
    }

    fn stack_ref(&mut self, loc: i32) -> Option<&mut i32> {
        if loc < 0 {
            self.status = Error(MachineError::StackIndexNegative);
            return None
        }
        let max_loc = self.stack.len();
        if loc >= max_loc as i32 {
            self.status = Error(MachineError::StackIndexOutOfBounds);
            return None
        }
        Some(&mut self.stack[loc as usize])
    }

    pub fn extend(&mut self, amt: i32) {
        self.stack.extend(iter::repeat(0).take(amt as usize));
        self.update_sp();
    }

    pub fn load(&mut self, offset: i32) -> Option<i32> {
        match self.stack_offset_ref(offset) {
            None => None,
            Some(r) => Some(*r)
        }
    }

    pub fn store(&mut self, x: i32, offset: i32) {
        match self.stack_offset_ref(offset) {
            None => {},
            Some(r) => {
                *r = x;
            }
        };
    }

    pub fn setpc(&mut self, loc: i32) {
        *self.pc() = loc;
    }

    pub fn getpc(&mut self) -> i32 {
        *self.pc()
    }

    pub fn pushpc(&mut self) {
        let pc = self.getpc();
        self.push(pc);
    }

    pub fn jump(&mut self, offset: i32) {
        let pc = self.getpc();
        self.setpc(pc + offset);
    }

    pub fn store_abs(&mut self, loc: i32, x: i32) {
        if let Some(r) = self.stack_ref(loc) {
            *r = x;
        }
    }

    pub fn load_abs(&mut self, loc: i32) -> Option<i32> {
        match self.stack_ref(loc) {
            None => None,
            Some(r) => Some(*r),
        }
    }

    pub fn stack_dump(&self) -> String {
        let mut out = String::new();
        for (i, x) in self.stack.iter().enumerate() {
            let extra = match i as i32 {
                PC_ADDR => "pc",
                SP_ADDR => "sp",
                FP_ADDR => "fp",
                BOUNDARY_ADDR => "boundary",
                _ => ""
            };
            write!(out, "{:02x}. {:8x} [{:8}] {}\n", i, x, x, extra).unwrap();
        }
        out
    }

    fn cycle(&mut self) {
        let pc = self.getpc();
        let inst = match self.inst_at_addr(pc) {
            Err(e) => {
                self.status = Error(e);
                return
            },
            Ok(inst) => inst
        };
        println!("{:<5} {}", self.ncycles, inst);
        (inst.op.f)(self, inst.arg);
        self.jump(1);
        self.ncycles += 1;
        if self.ncycles == MAX_CYCLES {
            self.status = Error(MachineError::MaxCyclesReached);
        }
    }

    fn inst_at_addr(&mut self, addr: i32) -> Result<Inst, MachineError> {
        if addr < CODE_START || addr >= (CODE_START + CODE_MAX_LEN as i32) {
            return Err(MachineError::CodeSegFault);
        }
        let inst_addr = (addr - CODE_START) as usize;
        let bin_inst = self.code_mem[inst_addr];
        match self.encoder.decode(bin_inst) {
            None => Err(MachineError::CannotDecodeInstruction(bin_inst)),
            Some(inst) => Ok(Inst{
                addr: Some(addr),
                ..inst
            })
        }
    }

}