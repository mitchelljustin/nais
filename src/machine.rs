use std::{cmp, fmt, io};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Write as FmtWrite};
use std::io::Write;
use std::ops::Range;

use MachineError::*;
use MachineStatus::*;

use crate::constants::{BOUNDARY_ADDR, FP_ADDR, INIT_STACK, MAX_CYCLES, PC_ADDR, SEG_CODE_END, SEG_CODE_START, SEG_LEN, SEG_STACK_END, SEG_STACK_START, SP_ADDR, SP_MINIMUM};
use crate::isa::{Encoder, Inst};
use crate::util;

pub struct CallFrame {
    pub name: String,
    pub start_addr: i32,
    pub args: HashMap<String, i32>,
    pub locals: HashMap<String, i32>,
}

pub trait DebugInfo {
    fn resolved_label_for_inst(&self, addr: i32) -> Option<(String, String)>;
    fn call_frame_for_inst(&self, addr: i32) -> Option<CallFrame>;
    fn value_for_label(&self, name: &str) -> Option<(i32, String)>;
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MachineError {
    IllegalStackPop,
    StackOverflow,
    PCSegFault,
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

pub struct Machine<'a> {
    mem_code: Vec<i32>,
    mem_stack: Vec<i32>,
    status: MachineStatus,
    ncycles: usize,
    encoder: Encoder,
    debug_info: Option<&'a dyn DebugInfo>,
    pub verbose: bool,
    pub max_cycles: usize,
}

impl Debug for Machine<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Machine")
            .field("status", &self.status)
            .field("ncycles", &self.ncycles)
            .finish()
    }
}


impl<'a> Machine<'a> {
    pub fn new() -> Machine<'a> {
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
            debug_info: None,
            verbose: false,
            max_cycles: MAX_CYCLES,
        }
    }

    pub fn copy_code(&mut self, code: &[i32]) {
        for (i, inst) in code.iter().enumerate() {
            self.mem_code[i] = *inst;
        }
    }

    pub fn attach_debug_info(&mut self, debug_info: &'a dyn DebugInfo) {
        self.debug_info = Some(debug_info);
    }

    pub fn run(&mut self) {
        self.set_status(Running);
        while self.status == Running {
            self.cycle();
        }
        if self.status != Stopped {
            println!("{:?}", self);
            self.breakpoint();
        }
    }

    pub fn set_status(&mut self, status: MachineStatus) {
        self.status = status;
    }

    pub fn setpc(&mut self, addr: i32) {
        if addr < SEG_CODE_START || addr >= SEG_CODE_END {
            self.status = Error(PCSegFault);
            return;
        }
        self.mem_stack[PC_ADDR as usize] = addr;
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

    pub fn breakpoint(&mut self) {
        println!("\nBREAKPOINT");
        self.jump(1);
        println!("{}", self.code_dump_around_pc(-4..5));
        self.run_debugger();
    }

    fn run_debugger(&mut self) {
        let mut next_counter = 0;
        loop {
            print!("debug% ");
            io::stdout().flush().unwrap();
            let mut line = String::new();
            if let Err(_) = io::stdin().read_line(&mut line) {
                return;
            }
            let line = line.trim();
            let parts = line.split(" ").collect::<Vec<_>>();
            let int_args = parts[1..]
                .iter()
                .filter_map(|s| util::parse_hex(s))
                .collect::<Vec<_>>();
            let code_range = |i| match int_args[i..] {
                [a, b] => a..b,
                [r] => -r..r + 1,
                _ => -10..11
            };
            let command = parts[0];
            match command {
                "c" | "continue" => {
                    return;
                },
                "n" | "next" => {
                    let pc = self.getpc();
                    self.cycle();
                    if self.getpc() == pc + 1 {
                        next_counter += 1;
                    } else {
                        next_counter = 0;
                    }
                    println!("{}", self.code_dump_around_pc((-4 - next_counter)..5));
                }
                "pc" | "code" => {
                    println!("{}", self.code_dump_around_pc(code_range(0)));
                }
                "ps" | "stack" => {
                    println!("{}", self.stack_dump());
                }
                "pm" | "machine" => {
                    println!("{:?}", self);
                }
                "s" | "store" => {
                    if let [addr, val] = int_args[..] {
                        self.store(addr, val);
                    } else {
                        println!("format: s|store addr val");
                    }
                }
                "l" | "load" => {
                    if let [addr] = int_args[..] {
                        if let Some(dump) = self.stack_addr_dump(addr) {
                            println!("{}", dump);
                        } else {
                            println!("Invalid address")
                        }
                    } else {
                        println!("format: l|load addr");
                    }
                }
                "lc" | "loadcode" => {
                    if let [addr] = int_args[..] {
                        println!("{}", self.code_dump_around(addr, code_range(1)));
                    } else {
                        println!("format: lc|loadcode addr [range]");
                    }
                }
                "x" | "exit" => {
                    self.set_status(Stopped);
                    return;
                }
                "" => {}
                _ => {
                    println!("?");
                }
            }
        }
    }

    pub fn code_dump_around_pc(&self, drange: Range<i32>) -> String {
        self.code_dump_around(self.getpc(), drange)
    }

    pub fn code_dump_around(&self, middle: i32, drange: Range<i32>) -> String {
        let lo = cmp::max(SEG_CODE_START, middle + drange.start);
        let hi = cmp::min(SEG_CODE_END, middle + drange.end);
        self.code_dump(middle, lo..hi)
    }

    pub fn code_dump(&self, highlight: i32, range: Range<i32>) -> String {
        let mut out = String::new();
        let mut cur_frame = match self.call_frame_for_inst(range.start) {
            None => None,
            Some(frame) => {
                writeln!(out, ".. {}:", frame.name).unwrap();
                Some(frame)
            }
        };
        for addr in range {
            if let Some(frame) = self.call_frame_for_inst(addr) {
                if frame.name != cur_frame.as_ref().unwrap().name {
                    writeln!(out, "{}:", frame.name).unwrap();
                    cur_frame = Some(frame);
                }
            }
            out.write_str("    ").unwrap();
            match self.inst_at_addr(addr) {
                Ok(inst) => { out.write_str(&inst.to_string()).unwrap(); }
                Err(err) => {
                    writeln!(out, "ERR FETCHING INST {:?}", err).unwrap();
                    continue;
                }
            };
            if let Some((lab, lab_type)) = self.resolved_label_for_inst(addr) {
                write!(out, " {:8} ({})", lab, lab_type).unwrap();
            } else {
                out.write_str(&" ".repeat(13)).unwrap();
            }
            if addr == highlight {
                out.write_str(" <========").unwrap()
            }
            out.write_str("\n").unwrap();
        }
        out
    }

    pub fn stack_dump(&self) -> String {
        let fp = self.mem_stack[FP_ADDR as usize] as usize;
        (0..self.getsp())
            .filter_map(|addr| self.stack_addr_dump(addr))
            .enumerate()
            .map(|(addr, s)| if addr == fp {
                s + "<======== FP"
            } else {
                s
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn stack_addr_dump(&self, addr: i32) -> Option<String> {
        if let Some(_) = self.check_stack_addr(addr) {
            return None;
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
            return Err(PCSegFault);
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

impl DebugInfo for Machine<'_> {
    fn resolved_label_for_inst(&self, addr: i32) -> Option<(String, String)> {
        match self.debug_info {
            None => None,
            Some(info) => info.resolved_label_for_inst(addr)
        }
    }

    fn call_frame_for_inst(&self, addr: i32) -> Option<CallFrame> {
        match self.debug_info {
            None => None,
            Some(info) => info.call_frame_for_inst(addr)
        }
    }

    fn value_for_label(&self, name: &str) -> Option<(i32, String)> {
        match self.debug_info {
            None => None,
            Some(info) => info.value_for_label(name)
        }
    }
}