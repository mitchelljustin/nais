use std::{fmt, io};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter, Write as FmtWrite};
use std::io::Write;
use std::ops::Range;

use MachineError::*;
use MachineStatus::*;

use crate::encoder::Encoder;
use crate::environment::Environment;
use crate::isa::Inst;
use crate::linker::{DebugInfo, ResolvedTarget};
use crate::mem::{addrs, inst_loc_to_addr, Memory, segs};
use crate::util;

#[derive(Debug, PartialEq, Clone)]
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
    LoadAddressOutOfBounds { addr: i32 },
    StoreAddressOutOfBounds { addr: i32 },
    AttemptedWriteToCodeSegment { addr: i32 },
    MaxCyclesReached,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MachineStatus {
    Idle,
    Running,
    Debugging,
    Stopped,
    Error(MachineError),
}

pub struct Machine {
    mem: Memory,

    status: MachineStatus,
    ncycles: usize,
    encoder: Encoder,

    pub(crate) env: Environment,

    pub debug_info: DebugInfo,
    pub max_cycles: usize,
    pub debug_on_error: bool,
}

impl Machine {
    pub(crate) fn getpc(&self) -> i32 {
        self.mem[addrs::PC]
    }

    pub(crate) fn getsp(&self) -> i32 {
        self.mem[addrs::SP]
    }

    pub fn stack_load(&mut self, addr: i32) -> Option<i32> {
        let sp = self.getsp();
        if addr >= sp {
            self.set_error(MachineError::StackAccessBeyondSP { sp, addr });
            return None;
        }
        if !self.stack_access_ok(addr) {
            self.set_error(StackAccessSegFault { addr });
            return None;
        }
        Some(self.load(addr))
    }

    pub fn stack_store(&mut self, addr: i32, val: i32) -> bool {
        if !self.stack_access_ok(addr) {
            self.set_error(StackAccessSegFault { addr });
            return false;
        }
        let sp = self.getsp();
        if addr >= sp {
            self.set_error(MachineError::StackAccessBeyondSP { sp, addr });
            return false;
        }
        if addr == addrs::SP {
            self.set_error(MachineError::IllegalDirectWriteSP);
            return false;
        }
        if addr == addrs::PC {
            self.set_error(MachineError::IllegalDirectWritePC);
            return false;
        }
        self.store(addr, val);
        true
    }

    fn stack_access_ok(&self, addr: i32) -> bool {
        segs::STACK.contains(addr)
    }

    fn code_access_ok(&self, addr: i32) -> bool {
        segs::CODE.contains(addr)
    }

    pub fn store(&mut self, addr: i32, val: i32) {
        if segs::CODE.contains(addr) {
            self.set_error(AttemptedWriteToCodeSegment { addr });
            return;
        }
        if !segs::ADDR_SPACE.contains(&addr) {
            self.set_error(StoreAddressOutOfBounds { addr });
            return;
        }
        self.mem[addr] = val;
    }

    pub fn load(&mut self, addr: i32) -> i32 {
        if !segs::ADDR_SPACE.contains(&addr) {
            self.set_error(LoadAddressOutOfBounds { addr });
            return 0;
        }
        self.mem[addr]
    }

    pub fn setpc(&mut self, newpc: i32) {
        if !self.code_access_ok(newpc) {
            self.set_error(ImminentPCSegFault { newpc });
            return;
        }
        self.store(addrs::PC, newpc);
    }

    pub fn setsp(&mut self, newsp: i32) {
        if newsp < addrs::INIT_SP {
            self.set_error(IllegalSPReductionBelowMin { newsp });
            return;
        }
        self.store(addrs::SP, newsp);
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
        println!("FRAME:\n{}\n", self.frame_dump());
        println!("CODE:\n{}", self.code_dump_around_pc(-4..5));
        loop {
            print!("debug% ");
            io::stdout().flush().unwrap();
            let mut line = String::new();
            io::stdin().read_line(&mut line).unwrap();
            let line = line.trim();
            let words = line.split(" ").collect::<Vec<_>>();
            let command = words[0];
            let args = &words[1..];
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
                            println!("{}", self.code_dump_around_pc(-15..16)),
                        _ => {
                            println!("format: pc addr [range]");
                        }
                    }
                }
                "ps" => {
                    match int_args[..] {
                        [mid, len] =>
                            println!("{}", self.stack_dump((mid - len)..(mid + len + 1))),
                        [mid] =>
                            println!("{}", self.stack_dump((mid - 4)..(mid + 4))),
                        [] =>
                            println!("{}", self.stack_dump_all()),
                        _ =>
                            println!("format: ps [addr] [range]"),
                    }
                }
                "pm" => {
                    match int_args[..] {
                        [start, len] =>
                            println!("{}", self.mem_dump(start..(start + len))),
                        [start] =>
                            println!("{}", self.mem_dump(start..(start + 1))),
                        _ =>
                            println!("format: pm start [len]"),
                    }
                }
                "st" => {
                    println!("{:?}", self);
                }
                "x" => {
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
        segs::CODE.clamp_range(&mut range);
        self.code_dump(middle, range)
    }

    pub fn code_dump(&self, highlight: i32, addr_range: Range<i32>) -> String {
        let mut out = String::new();
        let mut cur_frame = match self.debug_info.frame_for_inst_addr.get(&addr_range.start) {
            Some(name) => {
                writeln!(out, ".. {}:", name).unwrap();
                Some(name.clone())
            }
            None => None,
        };
        for addr in addr_range {
            if let Some(frame) = self.debug_info.frame_for_inst_addr.get(&addr) {
                if frame != cur_frame.as_ref().unwrap() {
                    writeln!(out, "{}:", frame).unwrap();
                    cur_frame = Some(frame.clone());
                }
            }
            out.write_str("    ").unwrap();
            match self.load_inst(addr) {
                Ok(inst) => out.write_str(&inst.to_string()).unwrap(),
                Err(MachineError::CannotDecodeInst(bin_inst)) => {
                    writeln!(out, "{:x} [0x{:08x}]", addr, bin_inst).unwrap();
                    continue;
                }
                Err(err) => {
                    writeln!(out, "ERR FETCHING INST {:?}", err).unwrap();
                    continue;
                }
            };
            match self.debug_info.resolved_idents.get(&addr) {
                Some(ResolvedTarget { idents, label_type, .. }) => {
                    write!(out, " {:12} {}", idents.first().unwrap_or(&"".to_string()), label_type).unwrap();
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

    pub fn frame_dump(&self) -> String {
        let fp = self.mem[addrs::FP];
        self.stack_dump(fp - 8..self.getsp())
    }

    pub fn stack_dump_all(&self) -> String {
        self.stack_dump(0..self.getsp())
    }

    pub fn stack_dump(&self, mut addr_range: Range<i32>) -> String {
        segs::STACK.clamp_range(&mut addr_range);
        let frame = self.debug_info.frame_for_inst_addr
            .get(&self.getpc());
        let var_for_offset = match frame {
            Some(frame) => self.debug_info.call_frames
                .get(frame)
                .unwrap()
                .local_mappings.iter()
                .filter(|(name, _)| name.len() > 0 && !name.starts_with("."))
                .map(|(name, offset)| (offset, name))
                .collect(),
            None => HashMap::new(),
        };
        let fp = self.mem[addrs::FP];
        let extra_infos = addr_range.clone().map(
            |addr| {
                vec![
                    match addr {
                        addrs::PC =>
                            " pc",
                        addrs::SP =>
                            " sp",
                        addrs::FP =>
                            " fp",
                        addrs::BOUNDARY =>
                            " --",
                        _ =>
                            ""
                    }.to_string(),
                    match addr - fp {
                        -3 => " retval".to_string(),
                        -2 => " retaddr".to_string(),
                        -1 => " saved fp".to_string(),
                        offset => {
                            match var_for_offset.get(&offset) {
                                None =>
                                    " ".repeat(13),
                                Some(var_name) =>
                                    format!(" {:12}", var_name),
                            }
                        }
                    },
                    if addr == fp {
                        " <======== FP".to_string()
                    } else {
                        " ".repeat(13)
                    }
                ].join("")
            });
        addr_range
            .map(|addr| self.formatted_stack_val(addr).unwrap())
            .zip(extra_infos)
            .map(|(desc, extra_info)| desc + &extra_info)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn mem_dump(&self, addr_range: Range<i32>) -> String {
        addr_range
            .map(|addr| {
                if !segs::ADDR_SPACE.contains(&addr) {
                    return "INVALID".to_string();
                }
                let val = self.mem[addr];
                let maybe_char =
                    if (0x20..=0x7f).contains(&val) {
                        format!(" '{}'", char::from(val as u8))
                    } else {
                        "".to_string()
                    };
                format!("{:01x} {:04x}: {:8x} [{:12}]{}", addr >> 16, addr & 0xffff, val, val, maybe_char)
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn formatted_stack_val(&self, addr: i32) -> Option<String> {
        if !self.stack_access_ok(addr) {
            return None;
        }
        let val = self.mem[addr];
        Some(format!("{:04x}: {:8x} [{:12}]", addr, val, val))
    }

    pub fn new() -> Machine {
        Machine {
            mem: Memory::new(),
            env: Default::default(),
            encoder: Encoder::new(),
            debug_info: DebugInfo::new(),
            status: Idle,
            ncycles: 0,
            debug_on_error: true,
            max_cycles: 1_000_000,
        }
    }

    pub fn load_code(&mut self, code: &[i32]) {
        for (loc, bin_inst) in code.iter().enumerate() {
            let addr = inst_loc_to_addr(loc);
            self.mem[addr] = *bin_inst;
        }
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
        if self.status != Stopped && self.debug_on_error {
            println!("{:?}", self);
            self.breakpoint();
            self.debug_cycle();
        }
    }

    pub fn cycle(&mut self) {
        let pc = self.getpc();
        let inst = match self.load_inst(pc) {
            Err(e) => {
                self.set_status(Error(e));
                return;
            }
            Ok(inst) => inst
        };
        (inst.op.func)(self, inst.arg);
        self.setpc(self.getpc() + 1);
        self.ncycles += 1;
        if self.status == Debugging {
            self.debug_cycle();
        }
        if self.ncycles == self.max_cycles {
            self.set_status(Error(MaxCyclesReached));
        }
    }

    fn load_inst(&self, addr: i32) -> Result<Inst, MachineError> {
        if !self.code_access_ok(addr) {
            return Err(CodeAccessSegFault { addr });
        }
        let bin_inst = self.mem[addr];
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