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
    pub frame_for_inst_addr: HashMap<i32, String>,
    pub resolved_idents: HashMap<i32, ResolvedIdent>,
}

impl DebugInfo {
    pub fn new() -> DebugInfo {
        DebugInfo {
            call_frames: HashMap::new(),
            frame_for_inst_addr: HashMap::new(),
            resolved_idents: HashMap::new(),
        }
    }
}

impl From<Linker> for DebugInfo {
    fn from(linker: Linker) -> Self {
        let mut info = DebugInfo::new();
        info.resolved_idents =      linker.resolved_idents;
        info.call_frames =          linker.call_frames;
        info.frame_for_inst_addr =  linker.frame_for_inst_addr;
        info
    }
}

#[derive(Copy, Clone)]
pub enum LabelType {
    GlobalConstant,
    Subroutine,
    InnerLabel,
    FrameVar,
}

#[derive(Clone)]
pub struct ResolvedIdent {
    pub inst_addr: i32,
    pub target: String,
    pub value: i32,
    pub label_type: LabelType,
}

impl Display for LabelType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LabelType::GlobalConstant => "const",
            LabelType::Subroutine =>     "sub",
            LabelType::InnerLabel =>     "inner",
            LabelType::FrameVar =>       "var",
        })
    }
}


#[derive(Clone)]
pub struct CallFrame {
    pub name: String,
    pub addr_range: Range<i32>,
    pub frame_vars: HashMap<String, i32>,
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
    cur_frame_name: String,
    frame_for_inst_addr: HashMap<i32, String>,

    global_constants: HashMap<String, i32>,
    to_relocate: HashMap<usize, String>,
    resolved_idents: HashMap<i32, ResolvedIdent>,

    encoder: Encoder,

    errors: Vec<LinkerError>,
}

impl Linker {
    pub fn new() -> Linker {
        Linker {
            instructions: Vec::new(),
            call_frames: HashMap::new(),
            global_constants: HashMap::new(),
            frame_for_inst_addr: HashMap::new(),
            to_relocate: HashMap::new(),
            resolved_idents: HashMap::new(),
            cur_frame_name: String::new(),
            encoder: Encoder::new(),
            errors: Vec::new(),
        }
    }

    pub fn add_inst(&mut self, opname: &str, arg: i32) {
        let addr = self.next_inst_addr();
        self.frame_for_inst_addr.insert(addr, self.cur_frame_name.clone());
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
        self.to_relocate.insert(self.next_inst_loc(), label.to_string());
        self.add_inst(opname, 0);
    }

    pub fn add_subroutine(&mut self, name: &str) {
        let next_addr = self.next_inst_addr();
        if self.cur_frame_name != "" {
            self.cur_frame_mut().addr_range.end = next_addr;
        }
        self.call_frames.insert(name.to_string(), CallFrame {
            name: name.to_string(),
            addr_range: next_addr..-1,
            frame_vars: HashMap::new(),
            inner_labels: HashMap::new(),
            locals_size: 0,
            args_size: 0,
        });
        self.cur_frame_name = name.to_string();
    }

    pub fn add_inner_label(&mut self, name: &str) {
        let addr = self.next_inst_addr();
        self.cur_frame_mut().inner_labels.insert(name.to_string(), addr);
    }

    fn cur_frame_mut(&mut self) -> &mut CallFrame {
        match self.call_frames.get(&self.cur_frame_name) {
            Some(_) => {
                self.call_frames.get_mut(&self.cur_frame_name).unwrap()
            },
            None => {
                const DEFAULT_ENTRY_LABEL: &str = "_entry";
                self.errors.push(LinkerError::NeedToDefineEntryLabel);
                self.add_subroutine(DEFAULT_ENTRY_LABEL);
                self.call_frames.get_mut(DEFAULT_ENTRY_LABEL).unwrap()
            }
        }
    }

    pub fn cur_frame(&self) -> &CallFrame {
        self.call_frames.get(&self.cur_frame_name).unwrap()
    }

    pub fn add_local_constant(&mut self, name: &str, value: i32) {
        self.cur_frame_mut().frame_vars.insert(
            name.to_string(),
            value,
        );
    }

    pub fn add_local_var(&mut self, name: &str, size: i32) {
        let frame = self.cur_frame_mut();
        frame.frame_vars.insert(
            name.to_string(),
            frame.locals_size,
        );
        frame.locals_size += size;
    }

    pub fn add_arg_var(&mut self, name: &str, size: i32) {
        let frame = self.cur_frame_mut();
        frame.frame_vars.insert(
            name.to_string(),
            -frame.args_size - 4, // [..args retval retaddr savedfp || locals ]
        );
        frame.args_size += size;
    }

    pub fn add_global_constant(&mut self, name: &str, value: i32) {
        self.global_constants.insert(name.to_string(), value);
    }

    pub fn finish(&mut self) {
        self.cur_frame_mut().addr_range.end = self.next_inst_addr();
    }

    fn resolve_ident(&self, target: &str, inst_loc: usize) -> Option<ResolvedIdent> {
        let inst_addr = inst_loc_to_addr(inst_loc);
        if let Some(entry) = self.resolved_idents.get(&inst_addr) {
            return Some(entry.clone());
        }
        // Global constant
        let target = target.to_string();
        if let Some(&value) = self.global_constants.get(&target) {
            return Some(ResolvedIdent {
                inst_addr,
                target,
                value,
                label_type: LabelType::GlobalConstant,
            });
        }
        if let Some(frame) = self.call_frames.get(&target) {
            let value = Linker::offset_from_inst(
                frame.addr_range.start,
                inst_addr,
            );
            return Some(ResolvedIdent {
                inst_addr,
                target,
                value,
                label_type: LabelType::Subroutine,
            });
        }
        // Local frame
        let frame_name = match self.frame_for_inst_addr.get(&inst_addr) {
            None => panic!(),
            Some(x) => x,
        };
        let frame = self.call_frames.get(frame_name)?;
        // Local code (inner label)
        if let Some(&addr) = frame.inner_labels.get(&target) {
            let value = Linker::offset_from_inst(
                addr,
                inst_addr,
            );
            return Some(ResolvedIdent {
                inst_addr,
                target,
                value,
                label_type: LabelType::InnerLabel,
            });
        }
        // Local frame var
        if let Some(&value) = frame.frame_vars.get(&target) {
            return Some(ResolvedIdent {
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
        for (inst_loc, target) in self.to_relocate.iter() {
            let inst_loc = *inst_loc;
            let inst_addr = inst_loc_to_addr(inst_loc);
            match self.resolve_ident(target, inst_loc) {
                Some(resolved) => {
                    inst_updates.push((inst_loc, resolved.value));
                    self.resolved_idents.insert(inst_addr, resolved);
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

    fn offset_from_inst(target_addr: i32, inst_addr: i32) -> i32 {
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


