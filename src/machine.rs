use std::fmt::{Debug, Formatter, Result};

use MachineStatus::*;

use crate::assemble::Program;
use std::iter;

const MAX_CYCLES: usize = 10_000;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MachineError {
    EmptyStackPop,
    PCOutOfBounds,
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
    pub program: Program,
    pub stack: Vec<i32>,
    pub status: MachineStatus,
    pub ncycles: usize,
}

impl Debug for Machine {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        let hex_stack: String = self.stack
            .iter()
            .fold(String::from("["), |acc, x| format!("{}{:x}, ", acc, x)) + "]";
        f.debug_struct("Machine")
            .field("status", &self.status)
            .field("stack", &self.stack)
            .field("stack(hex)", &hex_stack)
            .field("ncycles", &self.ncycles)
            .finish()
    }
}

const PC_ADDR: i32 = 0;
const SP_ADDR: i32 = 1;
const FP_ADDR: i32 = 2;

impl Machine {
    pub fn new(program: &Program) -> Machine {
        Machine {
            program: program.clone(),
            stack: vec![0, 4, 4, 11111111], // PC, SP, FP, boundary
            status: Idle,
            ncycles: 0,
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

    pub fn setfp(&mut self) {
        *self.fp() = *self.sp();
    }



    pub fn run(&mut self) {
        self.status = Running;
        while self.status == Running {
            let pc = self.getpc();
            if pc as usize >= self.program.len() {
                self.status = Error(MachineError::PCOutOfBounds);
                break;
            }
            let inst = self.program.inst_at(pc as usize);
            (inst.op.f)(self, inst.arg);
            self.jump(1);
            self.ncycles += 1;
            if self.ncycles == MAX_CYCLES {
                self.status = Error(MachineError::MaxCyclesReached);
            }
        }
    }
}