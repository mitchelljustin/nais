use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::ops::Range;

use crate::encoder::Encoder;
use crate::isa::{Inst, OP_INVALID};
use crate::linker::LinkerError::MissingTarget;
use crate::mem::{addrs, inst_loc_to_addr};

#[derive(Clone)]
pub struct DebugInfo {
    pub call_frames: HashMap<String, CallFrame>,
    pub frame_name_for_inst: HashMap<i32, String>,
    pub resolved_labels: HashMap<i32, ResolvedLabel>,
}

impl DebugInfo {
    pub fn new() -> DebugInfo {
        DebugInfo {
            call_frames: HashMap::new(),
            frame_name_for_inst: HashMap::new(),
            resolved_labels: HashMap::new(),
        }
    }
}

impl From<Linker> for DebugInfo {
    fn from(linker: Linker) -> Self {
        let mut info = DebugInfo::new();
        info.resolved_labels =      linker.resolved_labels;
        info.call_frames =          linker.call_frames;
        info.frame_name_for_inst =  linker.frame_name_for_inst;
        info
    }
}

#[derive(Copy, Clone)]
pub enum LabelType {
    Constant,
    GlobalVar,
    Subroutine,
    InnerLabel,
    FrameVar,
}

#[derive(Clone)]
pub struct ResolvedLabel {
    pub inst_addr: i32,
    pub target: String,
    pub value: i32,
    pub label_type: LabelType,
}

impl Display for LabelType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LabelType::Constant =>      "const",
            LabelType::GlobalVar =>     "glob",
            LabelType::Subroutine =>    "sub",
            LabelType::InnerLabel =>    "inner",
            LabelType::FrameVar =>      "var",
        })
    }
}


#[derive(Clone)]
pub struct CallFrame {
    pub name: String,
    pub addr_range: Range<i32>,
    pub frame_labels: HashMap<String, i32>,
    pub inner_labels: HashMap<String, i32>,
    pub locals_size: i32,
    pub args_size: i32,
}

#[derive(Clone, Debug)]
pub enum LinkerError {
    NeedToDefineEntryLabel,
    MissingTarget(Inst, String),
    NoSuchOp(i32, String),
}

impl Display for LinkerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MissingTarget(inst, target) => {
                write!(f, "MissingTarget({} \"{}\")", inst, target)
            }
            other => write!(f, "{:?}", other)
        }
    }
}


pub struct Linker {
    instructions: Vec<Inst>,
    call_frames: HashMap<String, CallFrame>,
    frame_name_for_inst: HashMap<i32, String>,
    cur_frame_name: String,
    global_vars: HashMap<String, i32>,
    constants: HashMap<String, i32>,
    reloc_tab: HashMap<usize, String>,
    resolved_labels: HashMap<i32, ResolvedLabel>,
    encoder: Encoder,
    errors: Vec<LinkerError>,
}

impl Linker {
    pub fn new() -> Linker {
        Linker {
            instructions: Vec::new(),
            call_frames: HashMap::new(),
            global_vars: HashMap::new(),
            constants: HashMap::new(),
            frame_name_for_inst: HashMap::new(),
            reloc_tab: HashMap::new(),
            resolved_labels: HashMap::new(),
            cur_frame_name: String::new(),
            encoder: Encoder::new(),
            errors: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        self.add_global_var("pc", addrs::PC);
        self.add_global_var("sp", addrs::SP);
        self.add_global_var("fp", addrs::FP);
        self.add_constant("retval", -3);
    }

    pub fn add_inst(&mut self, opname: &str, arg: i32) {
        let addr = self.next_inst_addr();
        self.frame_name_for_inst.insert(addr, self.cur_frame_name.clone());
        match self.encoder.make_inst(opname, arg) {
            Some(inst) => {
                self.instructions.push(Inst {
                    addr: Some(addr),
                    ..inst
                });
            }
            None => {
                self.errors.push(LinkerError::NoSuchOp(
                    addr, opname.to_string()));
                self.instructions.push(Inst {
                    opcode: 0x00,
                    op: OP_INVALID,
                    addr: Some(addr),
                    arg,
                });
            }
        };
    }

    fn next_inst_loc(&self) -> usize {
        self.instructions.len()
    }

    fn next_inst_addr(&self) -> i32 {
        inst_loc_to_addr(self.next_inst_loc())
    }

    pub fn add_placeholder_inst(&mut self, opname: &str, label: &str) {
        self.reloc_tab.insert(self.next_inst_loc(), label.to_string());
        self.add_inst(opname, 0);
    }

    pub fn add_subroutine_label(&mut self, name: &str) {
        let next_addr = self.next_inst_addr();
        if self.cur_frame_name != "" {
            self.cur_frame().addr_range.end = next_addr;
        }
        self.call_frames.insert(name.to_string(), CallFrame {
            name: name.to_string(),
            addr_range: next_addr..-1,
            frame_labels: HashMap::new(),
            inner_labels: HashMap::new(),
            locals_size: 0,
            args_size: 0,
        });
        self.cur_frame_name = name.to_string();
    }

    pub fn add_inner_label(&mut self, name: &str) {
        let addr = self.next_inst_addr();
        self.cur_frame().inner_labels.insert(name.to_string(), addr);
    }

    fn cur_frame(&mut self) -> &mut CallFrame {
        match self.call_frames.get(&self.cur_frame_name) {
            Some(_) => {
                self.call_frames.get_mut(&self.cur_frame_name).unwrap()
            },
            None => {
                const DEFAULT_ENTRY_LABEL: &str = "_entry";
                self.errors.push(LinkerError::NeedToDefineEntryLabel);
                self.add_subroutine_label(DEFAULT_ENTRY_LABEL);
                self.call_frames.get_mut(DEFAULT_ENTRY_LABEL).unwrap()
            }
        }
    }

    pub fn add_global_var(&mut self, name: &str, abs_loc: i32) {
        self.global_vars.insert(
            String::from(name),
            abs_loc,
        );
    }

    pub fn add_local_var(&mut self, name: &str, size: i32) {
        let frame = self.cur_frame();
        frame.frame_labels.insert(
            name.to_string(),
            frame.locals_size,
        );
        frame.locals_size += size;
    }

    pub fn add_arg_var(&mut self, name: &str, size: i32) {
        let frame = self.cur_frame();
        frame.frame_labels.insert(
            name.to_string(),
            -frame.args_size - 4, // [..args retval retaddr savedfp || locals ]
        );
        frame.args_size += size;
    }

    pub fn add_constant(&mut self, name: &str, value: i32) {
        self.constants.insert(String::from(name), value);
    }

    pub fn locals_alloc(&mut self) {
        let extend_sz = self.cur_frame().locals_size;
        if extend_sz > 0 {
            self.add_inst("addsp", extend_sz);
        }
    }

    pub fn locals_free(&mut self) {
        let drop_sz = self.cur_frame().locals_size;
        if drop_sz > 0 {
            self.add_inst("addsp", -drop_sz);
        }
    }

    pub fn finish(&mut self) {
        self.cur_frame().addr_range.end = self.next_inst_addr();
    }

    fn resolve_label(&self, inst_loc: usize, target: &str) -> Option<ResolvedLabel> {
        let inst_addr = inst_loc_to_addr(inst_loc);
        if let Some(entry) = self.resolved_labels.get(&inst_addr) {
            return Some(entry.clone());
        }
        // Constant
        let target = target.to_string();
        if let Some(&value) = self.constants.get(&target) {
            return Some(ResolvedLabel {
                inst_addr,
                target,
                value,
                label_type: LabelType::Constant,
            });
        }
        // Top level label
        if let Some(label_entry) = self.call_frames.get(&target) {
            let value = Linker::calc_inst_offset(
                label_entry.addr_range.start,
                inst_addr,
            );
            return Some(ResolvedLabel {
                inst_addr,
                target,
                value,
                label_type: LabelType::Subroutine,
            });
        }
        // Global variable
        if let Some(&value) = self.global_vars.get(&target) {
            return Some(ResolvedLabel {
                inst_addr,
                target,
                value,
                label_type: LabelType::GlobalVar,
            });
        }
        // Local frame
        let frame_name = match self.frame_name_for_inst.get(&inst_addr) {
            None => {
                panic!();
            },
            Some(x) => x,
        };
        let frame = match self.call_frames.get(frame_name) {
            None => return None,
            Some(f) => f,
        };
        // Local code (inner label)
        if let Some(&addr) = frame.inner_labels.get(&target) {
            let value = Linker::calc_inst_offset(
                addr,
                inst_addr,
            );
            return Some(ResolvedLabel {
                inst_addr,
                target,
                value,
                label_type: LabelType::InnerLabel,
            });
        }
        // Local frame var
        if let Some(&value) = frame.frame_labels.get(&target) {
            return Some(ResolvedLabel {
                inst_addr,
                target,
                value,
                label_type: LabelType::FrameVar,
            });
        }
        None
    }

    pub fn relocate(&mut self) -> Vec<(usize, String)> {
        let mut unrelocated = Vec::<(usize, String)>::new();
        let mut inst_updates = Vec::<(usize, i32)>::new();
        for (inst_loc, target) in self.reloc_tab.iter() {
            let inst_loc = *inst_loc;
            let inst_addr = inst_loc_to_addr(inst_loc);
            match self.resolve_label(inst_loc, target) {
                Some(resolved) => {
                    inst_updates.push((inst_loc, resolved.value));
                    self.resolved_labels.insert(inst_addr, resolved);
                }
                None => {
                    unrelocated.push((inst_loc, target.clone()));
                }
            }
        }
        for (loc, arg) in inst_updates.into_iter() {
            self.instructions[loc].arg = arg;
        }
        unrelocated
    }

    pub fn link_binary(&mut self) -> Result<Vec<i32>, Vec<LinkerError>> {
        let mut errors: Vec<LinkerError> = self.errors
            .drain(..)
            .collect();

        let unrelocated = self.relocate();
        errors.extend(
            unrelocated.into_iter()
                .map(|(loc, target)|
                    MissingTarget(self.instructions[loc], target.clone()))
        );
        if !errors.is_empty() {
            return Err(errors);
        }
        let bin = self.instructions.iter()
            .map(|inst| self.encoder.encode(inst))
            .collect();
        Ok(bin)
    }

    fn calc_inst_offset(target_addr: i32, inst_addr: i32) -> i32 {
        target_addr - inst_addr - 1
    }
}

impl Display for Linker {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.instructions
            .iter()
            .map(|inst| inst.to_string())
            .collect::<Vec<_>>()
            .join("\n")
        )
    }
}


