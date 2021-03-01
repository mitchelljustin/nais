use std::{fmt, io};
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
    pub var_names: HashMap<i32, String>,
}

pub trait DebugInfo {
    fn resolved_label_for_inst(&self, addr: i32) -> Option<(String, String)>;
    fn call_frame_for_inst(&self, addr: i32) -> Option<CallFrame>;
    fn value_for_label(&self, name: &str) -> Option<(i32, String)>;
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MachineError {
    IllegalStackPop,
    PCSegFault { newpc: i32 },
    InvalidInstruction,
    CannotDecodeInst(i32),
    StackAccessOutOfBounds { sp: i32, addr: i32 },
    StackSegFault { addr: i32 },
    ProgramExit(i32),
    NoSuchEnvCall(i32),
    MaxCyclesReached,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MachineStatus {
    Idle,
    Running,
    Debugging,
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
    pub enable_debugger: bool,
}

impl<'a> Machine<'a> {
    pub(crate) fn getpc(&self) -> i32 {
        self.mem_stack[PC_ADDR as usize]
    }

    pub(crate) fn getsp(&self) -> i32 {
        self.mem_stack[SP_ADDR as usize]
    }

    fn check_stack_addr(&self, addr: i32) -> Option<MachineError> {
        if addr < SEG_STACK_START || addr >= SEG_STACK_END {
            return Some(StackSegFault { addr });
        }
        None
    }

    fn stack_ref_mut(&mut self, addr: i32) -> Option<&mut i32> {
        if let Some(err) = self.check_stack_addr(addr) {
            self.set_status(Error(err));
            return None;
        }

        Some(&mut self.mem_stack[addr as usize])
    }

    pub fn store(&mut self, addr: i32, val: i32) {
        if let Some(ptr) = self.stack_ref_mut(addr) {
            *ptr = val;
        }
    }

    pub fn load(&mut self, addr: i32) -> Option<i32> {
        match self.stack_ref_mut(addr) {
            None => None,
            Some(ptr) => Some(*ptr),
        }
    }

    pub fn setpc(&mut self, newpc: i32) {
        if newpc < SEG_CODE_START || newpc >= SEG_CODE_END {
            self.set_error(PCSegFault { newpc });
            return;
        }
        self.store(PC_ADDR, newpc);
    }

    pub fn setsp(&mut self, newsp: i32) {
        if newsp <= SP_MINIMUM {
            self.set_error(IllegalStackPop);
            return;
        }
        self.store(SP_ADDR, newsp);
    }

    pub fn print(&mut self, val: i32) {
        if self.verbose {
            println!("\n>> {:8x} [{}]\n", val, val);
        } else {
            println!("{:8x} [{}]", val, val);
        }
    }

    pub fn breakpoint(&mut self) {
        println!("{}", match self.status {
            Error(_) =>
                "ERROR BREAKPOINT",
            _ =>
                "USER BREAKPOINT",
        });
        self.set_status(Debugging);
    }

    fn debug_cycle(&mut self) {
        println!("{}", self.code_dump_around_pc(-4..5));
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
                    self.set_status(Running);
                    return;
                }
                "n" | "next" => {
                    return;
                }
                "pc" | "code" => {
                    match int_args[..] {
                        [mid, len] =>
                            println!("{}", self.code_dump_around(mid, -len..len + 1)),
                        [mid] =>
                            println!("{}", self.code_dump_around(mid, -4..5)),
                        [] =>
                            println!("{}", self.code_dump_around_pc(-4..5)),
                        _ =>
                            println!("format: pc addr [range]"),
                    }
                    println!("{}", self.code_dump_around_pc(code_range(0)));
                }
                "ps" | "stack" => {
                    match int_args[..] {
                        [mid, len] =>
                            println!("{}", self.stack_mem_dump((mid - len)..(mid + len + 1))),
                        [mid] =>
                            println!("{}", self.stack_mem_dump((mid - 4)..(mid + 4))),
                        [] =>
                            println!("{}", self.stack_dump()),
                        _ =>
                            println!("format: ps addr [range]"),
                    }
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
        let mut range = (middle + drange.start)..(middle + drange.end);
        util::clamp(&mut range, SEG_CODE_START..SEG_CODE_END);
        self.code_dump(middle, range)
    }

    pub fn code_dump(&self, highlight: i32, addr_range: Range<i32>) -> String {
        let mut out = String::new();
        let mut cur_frame = match self.debug_info
            .map(|d| d.call_frame_for_inst(addr_range.start)) {
            Some(Some(frame)) => {
                writeln!(out, ".. {}:", frame.name).unwrap();
                frame.name
            }
            _ => "".to_string(),
        };
        for addr in addr_range {
            let maybe_frame = self.debug_info.map(|d| d.call_frame_for_inst(addr));
            if let Some(Some(frame)) = maybe_frame {
                if frame.name != cur_frame {
                    cur_frame = frame.name;
                    writeln!(out, "{}:", cur_frame).unwrap();
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
            let maybe_label = self.debug_info.map(|d| d.resolved_label_for_inst(addr));
            if let Some(Some((lab, lab_type))) = maybe_label {
                write!(out, " {:10} ({})", lab, lab_type).unwrap();
            } else {
                out.write_str(&" ".repeat(15)).unwrap();
            }
            if addr == highlight {
                out.write_str(" <========").unwrap()
            }
            out.write_str("\n").unwrap();
        }
        out
    }

    pub fn stack_dump(&self) -> String {
        self.stack_mem_dump(0..self.getsp())
    }

    pub fn stack_mem_dump(&self, mut addr_range: Range<i32>) -> String {
        util::clamp(&mut addr_range, SEG_STACK_START..SEG_STACK_END);
        let fp = self.mem_stack[FP_ADDR as usize];
        let var_names = match self.debug_info
            .map(|d| d.call_frame_for_inst(self.getpc())) {
            Some(Some(frame)) => frame.var_names,
            _ => HashMap::new()
        };
        let extra_frame_info = |addr: i32| {
            let mut out = String::new();
            let offset_from_fp = addr - fp;
            match offset_from_fp {
                -1 => out.write_str("saved fp").unwrap(),
                -2 => out.write_str("retaddr").unwrap(),
                _ => {
                    if let Some(var_name) = var_names.get(&offset_from_fp) {
                        write!(out, "{} ", var_name).unwrap();
                    }
                }
            }
            if addr == fp {
                out.write_str("<======== FP").unwrap();
            }
            out
        };
        addr_range
            .filter_map(|addr| self.stack_addr_dump(addr))
            .enumerate()
            .map(|(addr, s)| s + &extra_frame_info(addr as i32))
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
            enable_debugger: true,
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

    pub fn set_status(&mut self, status: MachineStatus) {
        self.status = status;
    }

    pub fn set_error(&mut self, error: MachineError) {
        self.set_status(Error(error))
    }

    pub fn is_running(&self) -> bool {
        match self.status {
            Running | Debugging => true,
            _ => false,
        }
    }

    pub fn run(&mut self) {
        self.set_status(Running);
        while self.is_running() {
            self.cycle();
        }
        if self.status != Stopped && self.enable_debugger {
            println!("{:?}", self);
            self.breakpoint();
            self.debug_cycle();
        }
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
        self.setpc(self.getpc() + 1);
        self.ncycles += 1;
        if self.status == Debugging {
            self.debug_cycle();
        }
        if self.ncycles == self.max_cycles {
            self.set_status(Error(MaxCyclesReached));
        }
    }

    fn inst_at_addr(&self, addr: i32) -> Result<Inst, MachineError> {
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

impl Debug for Machine<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Machine")
            .field("status", &self.status)
            .field("ncycles", &self.ncycles)
            .finish()
    }
}