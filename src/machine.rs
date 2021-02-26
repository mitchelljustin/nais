use std::fmt::{Debug, Formatter, Result};

use MachineStatus::*;

use crate::assemble::Program;

const MAX_CYCLES: usize = 10_000;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MachineError {
    EmptyStackPop,
    PCOutOfBounds,
    StackIndexOutOfBounds,
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
    pub program: Program,
    pub stack: Vec<i32>,
    pub status: MachineStatus,
    pub pc: i32,
    pub ncycles: usize,
}

impl Debug for Machine {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Machine")
            .field("status", &self.status)
            .field("stack", &self.stack)
            .field("pc", &self.pc)
            .field("ncycles", &self.ncycles)
            .finish()
    }
}

impl Machine {
    pub fn new(program: &Program) -> Machine {
        Machine {
            program: program.clone(),
            stack: Vec::new(),
            status: Idle,
            pc: 0,
            ncycles: 0,
        }
    }

    pub fn pop(&mut self) -> Option<i32> {
        match self.stack.pop() {
            None => {
                self.status = Error(MachineError::EmptyStackPop);
                return None;
            }
            Some(x) => Some(x)
        }
    }

    fn stack_offset_ref(&mut self, offset: i32) -> Option<&mut i32> {
        self.stack_ref(self.stack.len() as i32 - 1 - offset)
    }

    fn stack_ref(&mut self, loc: i32) -> Option<&mut i32> {
        if loc < 0 {
            self.status = Error(MachineError::StackIndexOutOfBounds);
            return None
        }
        let max_loc = self.stack.len();
        if loc >= max_loc as i32 {
            self.status = Error(MachineError::StackIndexOutOfBounds);
            return None
        }
        Some(&mut self.stack[loc as usize])
    }

    pub fn push(&mut self, x: i32) {
        self.stack.push(x)
    }

    pub fn peek(&mut self, offset: i32) -> Option<i32> {
        match self.stack_offset_ref(offset) {
            None => None,
            Some(r) => Some(*r)
        }
    }

    pub fn put(&mut self, x: i32, offset: i32) {
        match self.stack_offset_ref(offset) {
            None => {},
            Some(r) => {
                *r = x;
            }
        };
    }

    pub fn setpc(&mut self, loc: i32) {
        self.pc = loc;
    }

    pub fn jump(&mut self, offset: i32) {
        self.setpc(self.pc + offset);
    }

    pub fn store(&mut self, loc: i32, x: i32) {
        if let Some(r) = self.stack_ref(loc) {
            *r = x;
        }
    }

    pub fn load(&mut self, loc: i32) -> Option<i32> {
        match self.stack_ref(loc) {
            None => None,
            Some(r) => Some(*r),
        }
    }

    pub fn run(&mut self) {
        self.status = Running;
        while self.status == Running {
            if self.pc as usize >= self.program.len() {
                self.status = Error(MachineError::PCOutOfBounds);
                break;
            }
            let inst = self.program.inst_at(self.pc as usize);
            (inst.op.f)(self, inst.arg);
            self.pc += 1;
            self.ncycles += 1;
            if self.ncycles == MAX_CYCLES {
                self.status = Error(MachineError::MaxCyclesReached);
            }
        }
    }
}