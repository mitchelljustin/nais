use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::ops::Range;

use crate::encoder::Encoder;
use crate::isa::{Inst, OP_INVALID};
use crate::linker::LinkerError::MissingTarget;
use crate::mem::inst_loc_to_addr;

#[derive(Clone)]
pub struct DebugInfo {
    pub call_frames: HashMap<String, TopLevelLabel>,
    pub frame_for_inst_addr: HashMap<i32, String>,
    pub resolved_idents: HashMap<i32, ResolvedTarget>,
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
        info.resolved_idents = linker.resolved_targets;
        info.call_frames = linker.top_level_labels;
        info.frame_for_inst_addr = linker.frame_for_inst_addr;
        info
    }
}

#[derive(Copy, Clone, Debug)]
pub enum LabelType {
    Global,
    TopLevelLabel,
    InnerLabel,
    FrameVar,

    _Literal,
}

#[derive(Clone)]
pub struct ResolvedTarget {
    pub inst_addr: i32,
    pub idents: Vec<String>,
    pub value: i32,
    pub label_type: LabelType,
}

impl Display for LabelType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LabelType::Global           => "glob",
            LabelType::TopLevelLabel    => "label",
            LabelType::InnerLabel       => "inner",
            LabelType::FrameVar         => "var",
            _ => "",
        })
    }
}

#[derive(Clone)]
pub struct TopLevelLabel {
    pub name: String,
    pub addr_range: Range<i32>,
    pub local_mappings: HashMap<String, i32>,
    pub inner_labels: HashMap<String, i32>,
    pub locals_size: i32,
    pub params_size: i32,
}

#[derive(Clone, Debug)]
pub enum LinkerError {
    NeedToDefineEntryLabel,
    MissingTarget(Inst, Vec<String>),
    NoSuchOp(i32, String),
}

impl Display for LinkerError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MissingTarget(inst, target) => {
                write!(f, "MissingTarget({} \"{:?}\")", inst, target)
            }
            other => write!(f, "{:?}", other)
        }
    }
}


#[derive(Clone, Debug)]
pub enum TargetTerm {
    Ident(String),
    Literal(i32),
}

pub type RelocationTarget = Vec<TargetTerm>;

pub struct Linker {
    instructions: Vec<Inst>,
    to_relocate: HashMap<usize, RelocationTarget>,
    resolved_targets: HashMap<i32, ResolvedTarget>,

    top_level_labels: HashMap<String, TopLevelLabel>,
    pub(crate) cur_frame_name: String,
    frame_for_inst_addr: HashMap<i32, String>,
    global_mappings: HashMap<String, i32>,

    encoder: Encoder,

    errors: Vec<LinkerError>,
}

impl Linker {
    pub fn new() -> Linker {
        Linker {
            instructions: Vec::new(),
            top_level_labels: HashMap::new(),
            global_mappings: HashMap::new(),
            frame_for_inst_addr: HashMap::new(),
            to_relocate: HashMap::new(),
            resolved_targets: HashMap::new(),
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

    pub fn add_placeholder_inst(&mut self, opname: &str, target: RelocationTarget) {
        self.to_relocate.insert(self.next_inst_loc(), target);
        self.add_inst(opname, 0);
    }

    pub fn add_top_level_label(&mut self, name: &str) {
        if self.cur_frame_name != "" {
            self.end_current_frame();
        }
        let next_addr = self.next_inst_addr();
        self.add_global_constant(
            &format!(".L.{}.start", name),
            next_addr,
        );
        self.top_level_labels.insert(name.to_string(), TopLevelLabel {
            name: name.to_string(),
            addr_range: next_addr..-1,
            local_mappings: HashMap::new(),
            inner_labels: HashMap::new(),
            locals_size: 0,
            params_size: 0,
        });
        self.cur_frame_name = name.to_string();
    }

    pub fn add_inner_label(&mut self, name: &str) {
        let addr = self.next_inst_addr();
        self.cur_frame_mut().inner_labels.insert(name.to_string(), addr);
    }

    pub(crate) fn cur_frame_mut(&mut self) -> &mut TopLevelLabel {
        match self.top_level_labels.get(&self.cur_frame_name) {
            Some(_) => {
                self.top_level_labels.get_mut(&self.cur_frame_name).unwrap()
            }
            None => {
                const DEFAULT_ENTRY_LABEL: &str = "entry";
                self.errors.push(LinkerError::NeedToDefineEntryLabel);
                self.add_top_level_label(DEFAULT_ENTRY_LABEL);
                self.top_level_labels.get_mut(DEFAULT_ENTRY_LABEL).unwrap()
            }
        }
    }

    pub fn cur_frame(&self) -> &TopLevelLabel {
        self.top_level_labels.get(&self.cur_frame_name).unwrap()
    }

    pub fn add_local_constant(&mut self, name: &str, value: i32) {
        self.cur_frame_mut().local_mappings.insert(
            name.to_string(),
            value,
        );
    }

    pub fn add_local_var(&mut self, name: &str, size: i32) {
        let frame = self.cur_frame_mut();
        frame.local_mappings.insert(
            name.to_string(),
            frame.locals_size,
        );
        frame.locals_size += size;
    }

    pub fn add_param(&mut self, name: &str, size: i32) {
        let frame = self.cur_frame_mut();
        frame.local_mappings.insert(
            name.to_string(),
            -frame.params_size - 4, // [..args retval retaddr savedfp || locals ]
        );
        frame.params_size += size;
    }

    pub fn add_global_constant(&mut self, name: &str, value: i32) {
        self.global_mappings.insert(name.to_string(), value);
    }

    pub fn add_raw_word(&mut self, value: i32) {
        let addr = self.next_inst_addr();
        let inst = Inst {
            addr:   Some(addr),
            op:     OP_INVALID,
            opcode: ((value as u32 & 0xff000000) >> 24) as u8,
            arg:    (value & 0x00ffffff) as i32,
        };
        self.instructions.push(inst);
    }

    pub fn finish(&mut self) {
        self.end_current_frame();
    }

    fn end_current_frame(&mut self) {
        let next_addr = self.next_inst_addr();
        self.cur_frame_mut().addr_range.end = next_addr;
        self.add_global_constant(
            &format!(".L.{}.end", self.cur_frame_name),
            next_addr,
        );
        self.add_global_constant(
            &format!(".L.{}.len", self.cur_frame_name),
            self.cur_frame().addr_range.len() as i32,
        );
    }

    fn resolve_ident(&self, inst_loc: usize, name: &str) -> Option<(i32, LabelType)> {
        use LabelType::*;
        // Global mapping
        if let Some(&value) = self.global_mappings.get(name) {
            return Some((value, Global));
        }
        // Top level label
        let inst_addr = inst_loc_to_addr(inst_loc);
        if let Some(label) = self.top_level_labels.get(name) {
            let value = Linker::pc_relative(
                label.addr_range.start,
                inst_addr,
            );
            return Some((value, TopLevelLabel));
        }
        // Local frame
        let frame_name = self.frame_for_inst_addr.get(&inst_addr)?;
        let frame = self.top_level_labels.get(frame_name)?;
        // Local code (inner label)
        if let Some(&addr) = frame.inner_labels.get(name) {
            let value = Linker::pc_relative(
                addr,
                inst_addr,
            );
            return Some((value, InnerLabel));
        }
        // Local var
        if let Some(&value) = frame.local_mappings.get(name) {
            return Some((value, FrameVar));
        }
        None
    }

    fn resolve(&self, inst_loc: usize, target: &RelocationTarget) -> Result<ResolvedTarget, Vec<String>> {
        let inst_addr = inst_loc_to_addr(inst_loc);
        if let Some(entry) = self.resolved_targets.get(&inst_addr) {
            return Ok(entry.clone());
        }
        // Global constant
        let target = target.clone();
        let (resolutions, unresolved): (Vec<_>, Vec<_>) = target
            .iter()
            .map(|t| match t {
                TargetTerm::Ident(name) => self.resolve_ident(inst_loc, name).ok_or(name),
                TargetTerm::Literal(x) => Ok((*x, LabelType::_Literal)),
            })
            .partition(|r| r.is_ok());
        if !unresolved.is_empty() {
            return Err(unresolved
                .into_iter()
                .map(|r| r.unwrap_err())
                .cloned()
                .collect());
        }
        let resolutions = resolutions.into_iter()
            .map(|r| r.unwrap())
            .collect::<Vec<_>>();
        let value = resolutions.iter().map(|(v, _)| v).sum();
        let label_type = resolutions[0].1;
        let idents = target
            .into_iter()
            .filter_map(|t| match t {
                TargetTerm::Ident(name) => Some(name),
                TargetTerm::Literal(_) => None,
            })
            .collect();
        Ok(ResolvedTarget {
            inst_addr,
            label_type,
            idents,
            value,
        })
    }

    pub fn relocate(&mut self) -> Result<(), Vec<(usize, Vec<String>)>> {
        let mut unrelocated = Vec::<(usize, Vec<String>)>::new();
        let mut inst_updates = Vec::<(usize, i32)>::new();
        for (inst_loc, target) in self.to_relocate.iter() {
            let inst_loc = *inst_loc;
            let inst_addr = inst_loc_to_addr(inst_loc);
            match self.resolve(inst_loc, target) {
                Ok(resolved) => {
                    inst_updates.push((inst_loc, resolved.value));
                    self.resolved_targets.insert(inst_addr, resolved);
                }
                Err(unresolved) => {
                    unrelocated.push((inst_loc, unresolved));
                }
            }
        }
        for (loc, arg) in inst_updates.into_iter() {
            self.instructions[loc].arg = arg;
        }
        if !unrelocated.is_empty() {
            Err(unrelocated)
        } else {
            Ok(())
        }
    }

    pub fn link_binary(&mut self) -> Result<Vec<i32>, Vec<LinkerError>> {
        let mut errors: Vec<LinkerError> = self.errors.clone();

        if let Err(unrelocated) = self.relocate() {
            errors.extend(unrelocated
                .into_iter()
                .map(|(loc, unresolved)|
                    MissingTarget(self.instructions[loc], unresolved))
            );
        }
        if !errors.is_empty() {
            return Err(errors);
        }
        let bin = self.instructions.iter()
            .map(|inst| self.encoder.encode(inst))
            .collect();
        Ok(bin)
    }

    fn pc_relative(target_addr: i32, inst_addr: i32) -> i32 {
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


