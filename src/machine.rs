use std::{fmt, io};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Write as FmtWrite};
use std::io::Write;
use std::ops::Range;

use MachineError::*;
use MachineStatus::*;

use crate::assemble::{DebugInfo, ResolvedLabel};
use crate::constants::{BOUNDARY_ADDR, DEFAULT_MAX_CYCLES, FP_ADDR, INIT_STACK, PC_ADDR, SEG_CODE_END, SEG_CODE_SIZE, SEG_CODE_START, SEG_STACK_END, SEG_STACK_SIZE, SEG_STACK_START, SP_ADDR, SP_MIN};
use crate::isa::{Encoder, Inst};
use crate::util;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MachineError {
    IllegalSPReductionBelowMin { newsp: i32 },
    IllegalDirectWriteSP,
    IllegalDirectWritePC,
    ImminentPCSegFault { newpc: i32 },
    InvalidInstruction,
    CannotDecodeInst(i32),
    StackAccessBeyondSP { sp: i32, addr: i32 },
    StackAccessSegFault { addr: i32 },
    CodeAccessSegFault { addr: i32 },
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

pub struct Machine {
    mem_code: Vec<i32>,
    mem_stack: Vec<i32>,
    status: MachineStatus,
    ncycles: usize,
    encoder: Encoder,
    debug_info: DebugInfo,

    pub max_cycles: usize,
    pub enable_debugger: bool,
}

impl Machine {
    pub(crate) fn getpc(&self) -> i32 {
        self.mem_stack[PC_ADDR as usize]
    }

    pub(crate) fn getsp(&self) -> i32 {
        self.mem_stack[SP_ADDR as usize]
    }

    pub fn load(&mut self, addr: i32) -> Option<i32> {
        let sp = self.getsp();
        if addr >= sp {
            self.set_error(MachineError::StackAccessBeyondSP { sp, addr });
            return None;
        }
        self.unsafe_load(addr)
    }

    pub fn store(&mut self, addr: i32, val: i32) -> bool {
        let sp = self.getsp();
        if addr >= sp {
            self.set_error(MachineError::StackAccessBeyondSP { sp, addr });
            return false;
        }
        if addr == SP_ADDR {
            self.set_error(MachineError::IllegalDirectWriteSP);
            return false;
        }
        if addr == PC_ADDR {
            self.set_error(MachineError::IllegalDirectWritePC);
            return false;
        }
        self.unsafe_store(addr, val);
        true
    }

    fn stack_access_ok(&self, addr: i32) -> bool {
        addr >= SEG_STACK_START && addr < SEG_STACK_END
    }

    fn code_access_ok(&self, addr: i32) -> bool {
        addr >= SEG_CODE_START && addr < SEG_CODE_END
    }

    pub fn unsafe_store(&mut self, addr: i32, val: i32) {
        if !self.stack_access_ok(addr) {
            self.set_error(StackAccessSegFault { addr });
            return;
        }
        self.mem_stack[addr as usize] = val;
    }

    pub fn unsafe_load(&mut self, addr: i32) -> Option<i32> {
        if !self.stack_access_ok(addr) {
            self.set_error(StackAccessSegFault { addr });
            return None;
        }
        Some(self.mem_stack[addr as usize])
    }

    pub fn setpc(&mut self, newpc: i32) {
        if !self.code_access_ok(newpc) {
            self.set_error(ImminentPCSegFault { newpc });
            return;
        }
        self.unsafe_store(PC_ADDR, newpc);
    }

    pub fn setsp(&mut self, newsp: i32) {
        if newsp < SP_MIN {
            self.set_error(IllegalSPReductionBelowMin { newsp });
            return;
        }
        self.unsafe_store(SP_ADDR, newsp);
    }

    pub fn print(&mut self, val: i32) {
        println!("{:8x} [{}]", val, val);
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
            io::stdin().read_line(&mut line).unwrap();
            let line = line.trim();
            let words = line.split(" ").collect::<Vec<_>>();
            let command = words[0];
            let args: &[&str] = &words[1..];
            let int_args = args
                .iter()
                .filter_map(|s| util::parse_hex(s))
                .collect::<Vec<_>>();
            match command {
                "c" | "continue" => {
                    self.set_status(Running);
                    return;
                }
                "n" | "next" => {
                    return;
                }
                "pc" => {
                    match int_args[..] {
                        [mid, len] =>
                            println!("{}", self.code_dump_around(mid, -len..len + 1)),
                        [len] =>
                            println!("{}", self.code_dump_around_pc(-len..len + 1)),
                        [] =>
                            println!("{}", self.code_dump_around_pc(-4..5)),
                        _ => {
                            println!("format: pc addr [range]");
                        }
                    }
                }
                "ps" => {
                    match int_args[..] {
                        [mid, len] =>
                            println!("{}", self.stack_mem_dump((mid - len)..(mid + len + 1))),
                        [mid] =>
                            println!("{}", self.stack_mem_dump((mid - 4)..(mid + 4))),
                        [] =>
                            println!("{}", self.stack_dump()),
                        _ =>
                            println!("format: ps [addr] [range]"),
                    }
                }
                "pm" => {
                    println!("{:?}", self);
                }
                "s" | "store" => {
                    if let [addr, val] = int_args[..] {
                        self.unsafe_store(addr, val);
                    } else {
                        println!("format: s addr val");
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
        util::clamp_range(&mut range, SEG_CODE_START..SEG_CODE_END);
        self.code_dump(middle, range)
    }

    pub fn code_dump(&self, highlight: i32, addr_range: Range<i32>) -> String {
        let mut out = String::new();
        let mut cur_frame = match self.debug_info.frame_name_for_inst.get(&addr_range.start) {
            Some(name) => {
                writeln!(out, ".. {}:", name).unwrap();
                Some(name.clone())
            }
            None => None,
        };
        for addr in addr_range {
            if let Some(frame) = self.debug_info.frame_name_for_inst.get(&addr) {
                if frame != cur_frame.as_ref().unwrap() {
                    writeln!(out, "{}:", frame).unwrap();
                    cur_frame = Some(frame.clone());
                }
            }
            out.write_str("    ").unwrap();
            match self.inst_at_addr(addr) {
                Ok(inst) => out.write_str(&inst.to_string()).unwrap(),
                Err(err) => {
                    writeln!(out, "ERR FETCHING INST {:?}", err).unwrap();
                    continue;
                }
            };
            match self.debug_info.resolved_labels.get(&addr) {
                Some(ResolvedLabel { target, label_type, .. }) => {
                    write!(out, " {:12} {}", target, label_type).unwrap();
                }
                None => out.write_str(&" ".repeat(15)).unwrap(),
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
        util::clamp_range(&mut addr_range, SEG_STACK_START..SEG_STACK_END);
        let frame = self.debug_info.frame_name_for_inst
            .get(&self.getpc());
        let var_for_offset = match frame {
            Some(frame) => self.debug_info.call_frames
                .get(frame)
                .unwrap()
                .frame_vars.iter()
                .map(|(name, off)| (off, name))
                .collect(),
            None => HashMap::new(),
        };
        let fp = self.mem_stack[FP_ADDR as usize];
        let extra_infos = addr_range.clone().map(
            |addr| {
                vec![
                    match addr {
                        PC_ADDR =>
                            " pc",
                        SP_ADDR =>
                            " sp",
                        FP_ADDR =>
                            " fp",
                        BOUNDARY_ADDR =>
                            " --",
                        _ =>
                            ""
                    }.to_string(),
                    match addr - fp {
                        -2 => " retaddr".to_string(),
                        -1 => " saved fp".to_string(),
                        offset => {
                            match var_for_offset.get(&offset) {
                                Some(var_name) => format!(" {:12}", var_name),
                                None => " ".repeat(13),
                            }
                        }
                    },
                    if addr == fp { " <======== FP".to_string() } else { " ".repeat(13) }
                ].join("")
            });
        addr_range
            .filter_map(|addr| self.formatted_stack_val(addr))
            .zip(extra_infos)
            .map(|(desc, extra_info)| desc + &extra_info)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn formatted_stack_val(&self, addr: i32) -> Option<String> {
        if !self.stack_access_ok(addr) {
            return None;
        }
        let val = self.mem_stack[addr as usize];
        let ret = format!("{:04x}. {:8x} [{:8}]", addr, val, val);
        Some(ret)
    }

    pub fn new() -> Machine {
        let mut mem_stack = vec![0; SEG_STACK_SIZE as usize];
        for (i, x) in INIT_STACK.iter().enumerate() {
            mem_stack[i] = *x;
        }
        Machine {
            mem_stack,
            mem_code: vec![0; SEG_CODE_SIZE as usize],
            status: Idle,
            ncycles: 0,
            encoder: Encoder::new(),
            debug_info: DebugInfo::new(),
            enable_debugger: true,
            max_cycles: DEFAULT_MAX_CYCLES,
        }
    }

    pub fn copy_code(&mut self, code: &[i32]) {
        for (i, inst) in code.iter().enumerate() {
            self.mem_code[i] = *inst;
        }
    }

    pub fn attach_debug_info(&mut self, debug_info: DebugInfo) {
        self.debug_info = debug_info;
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
        if !self.code_access_ok(addr) {
            return Err(CodeAccessSegFault { addr });
        }
        let inst_loc = (addr - SEG_CODE_START) as usize;
        let bin_inst = self.mem_code[inst_loc];
        match self.encoder.decode(bin_inst) {
            None => Err(CannotDecodeInst(bin_inst)),
            Some(inst) => Ok(Inst {
                addr: Some(addr),
                ..inst
            })
        }
    }
}

impl Debug for Machine {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Machine")
            .field("status", &self.status)
            .field("ncycles", &self.ncycles)
            .finish()
    }
}