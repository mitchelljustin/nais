use std::{cmp, fmt};
use std::fmt::{Debug, Formatter, Write};
use std::ops::Range;

use MachineError::*;
use MachineStatus::*;

use crate::constants::{BOUNDARY_ADDR, FP_ADDR, INIT_STACK, MAX_CYCLES, PC_ADDR, SEG_CODE_END, SEG_CODE_START, SEG_LEN, SEG_STACK_END, SEG_STACK_START, SP_ADDR, SP_MINIMUM};
use crate::isa::{Encoder, Inst};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MachineError {
    IllegalStackPop,
    StackOverflow,
    CodeSegFault,
    InvalidInstruction,
    CannotDecodeInst(i32),
    StackIndexOutOfBounds,
    StackSegFault,
    ProgramExit(i32),
    NoSuchEnvCall(i32),
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
    mem_code: Vec<i32>,
    mem_stack: Vec<i32>,
    status: MachineStatus,
    ncycles: usize,
    encoder: Encoder,
    pub verbose: bool,
    pub max_cycles: usize,
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
        let mut mem_stack = vec![0; SEG_LEN as usize];
        for (i, x) in INIT_STACK.iter().enumerate() {
            mem_stack[i] = *x;
        }
        Machine {
            mem_stack,
            mem_code: vec![0; SEG_LEN as usize],
            status: Idle,
            ncycles: 0,
            encoder: Encoder::new(),
            verbose: false,
            max_cycles: MAX_CYCLES,
        }
    }

    pub fn copy_code(&mut self, code: &[i32]) {
        for (i, inst) in code.iter().enumerate() {
            self.mem_code[i] = *inst;
        }
    }

    pub fn run(&mut self) {
        self.set_status(Running);
        while self.status == Running {
            self.cycle();
        }
    }

    pub fn set_status(&mut self, status: MachineStatus) {
        self.status = status;
    }

    pub fn setpc(&mut self, loc: i32) {
        self.mem_stack[PC_ADDR as usize] = loc;
    }

    pub fn getpc(&self) -> i32 {
        self.mem_stack[PC_ADDR as usize]
    }

    fn getsp(&self) -> i32 {
        return self.mem_stack[SP_ADDR as usize];
    }

    fn setsp(&mut self, sp: i32) {
        self.mem_stack[SP_ADDR as usize] = sp;
    }

    pub fn pop(&mut self) -> Option<i32> {
        let sp = self.getsp();
        if sp <= SP_MINIMUM {
            self.set_status(Error(IllegalStackPop));
            return None;
        }
        let sp = sp - 1;
        self.setsp(sp);
        Some(self.mem_stack[sp as usize])
    }

    pub fn push(&mut self, val: i32) {
        let sp = self.getsp();
        if sp >= SEG_STACK_END {
            self.set_status(Error(StackOverflow));
            return;
        }
        self.mem_stack[sp as usize] = val;
        self.setsp(sp + 1);
    }

    pub fn extend(&mut self, amt: i32) {
        self.mem_stack[SP_ADDR as usize] += amt;
    }

    pub fn drop(&mut self, amt: i32) {
        self.mem_stack[SP_ADDR as usize] -= amt;
    }

    fn stack_ref_mut(&mut self, addr: i32) -> Option<&mut i32> {
        if let Some(err) = self.check_stack_addr(addr) {
            self.set_status(Error(err));
            return None;
        }

        Some(&mut self.mem_stack[addr as usize])
    }

    fn check_stack_addr(&self, addr: i32) -> Option<MachineError> {
        if addr < SEG_STACK_START || addr >= SEG_STACK_END {
            return Some(StackSegFault);
        }
        if addr >= self.getsp() {
            return Some(StackIndexOutOfBounds);
        }
        None
    }

    pub fn jump(&mut self, offset: i32) {
        let pc = self.getpc();
        self.setpc(pc + offset);
    }

    pub fn print(&mut self, val: i32) {
        if self.verbose {
            println!("\n>> {:8x} [{}]\n", val, val);
        } else {
            println!("{:8x} [{}]", val, val);
        }
    }

    pub fn store(&mut self, addr: i32, val: i32) {
        if let Some(r) = self.stack_ref_mut(addr) {
            *r = val;
        }
    }

    pub fn load(&mut self, addr: i32) -> Option<i32> {
        match self.stack_ref_mut(addr) {
            None => None,
            Some(r) => Some(*r),
        }
    }

    pub fn code_dump(&self, range: Range<i32>) -> String {
        let pc = self.getpc();
        let lo = cmp::max(SEG_CODE_START, pc + range.start);
        let hi = cmp::min(SEG_CODE_END, pc + range.end);
        let mut out = String::new();
        for addr in lo..=hi {
            let extra = if addr == pc { "<========= PC" } else { "" };
            match self.inst_at_addr(addr) {
                Ok(inst) =>
                    writeln!(out, "{:<32}{}", inst.to_string(), extra).unwrap(),
                Err(err) =>
                    writeln!(out, "ERR FETCHING INST {:?}", err).unwrap()
            };
        }
        out
    }

    pub fn stack_dump(&self) -> String {
        (0..self.getsp())
            .filter_map(|addr| self.stack_addr_dump(addr))
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn stack_addr_dump(&self, addr: i32) -> Option<String> {
        if let Some(_) = self.check_stack_addr(addr) {
            return None
        }
        let val = self.mem_stack[addr as usize];
        let extra = match addr {
            PC_ADDR => "pc",
            SP_ADDR => "sp",
            FP_ADDR => "fp",
            BOUNDARY_ADDR => "boundary",
            _ => ""
        };
        let ret = format!("{:04x}. {:8x} [{:8}] {}", addr, val, val, extra);
        Some(ret)
    }

    pub fn cycle(&mut self) {
        let pc = self.getpc();
        let inst = match self.inst_at_addr(pc) {
            Err(e) => {
                self.set_status(Error(e));
                return;
            }
            Ok(inst) => inst
        };
        if self.verbose {
            println!("{:<4} {}", self.ncycles, inst);
        }
        (inst.op.f)(self, inst.arg);
        self.jump(1);
        self.ncycles += 1;
        if self.ncycles == self.max_cycles {
            self.set_status(Error(MaxCyclesReached));
        }
    }

    fn inst_at_addr(&self, addr: i32) -> Result<Inst, MachineError> {
        if addr < SEG_CODE_START || addr >= SEG_CODE_END {
            return Err(CodeSegFault);
        }
        let inst_addr = (addr - SEG_CODE_START) as usize;
        let bin_inst = self.mem_code[inst_addr];
        match self.encoder.decode(bin_inst) {
            None => Err(CannotDecodeInst(bin_inst)),
            Some(inst) => Ok(Inst {
                addr: Some(addr),
                ..inst
            })
        }
    }
}