use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::ops::Range;

use crate::assemble::AssemblyError::MissingTarget;
use crate::constants::{FP_ADDR, PC_ADDR, SEG_CODE_START, SP_ADDR};
use crate::isa::{Encoder, Inst, OP_INVALID};
use crate::isa;

macro_rules! parse_asm_line {
    ( $p:ident label $label:ident ) => {
        $p.add_top_level_label(stringify!($label));
    };
    ( $p:ident inner $label:ident ) => {
        $p.add_inner_label(stringify!($label));
    };
    ( $p:ident global $name:ident $loc:literal ) => {
        $p.add_global_var(stringify!($name), $loc);
    };
    ( $p:ident const $name:ident $value:literal ) => {
        $p.add_constant(stringify!($name), $value);
    };
    ( $p:ident arg $($name:ident)+ ) => {
        $(
            $p.add_arg_var(stringify!($name), 1);
        )+
    };
    ( $p:ident local $($name:ident)+ ) => {
        $(
            $p.add_local_var(stringify!($name), 1);
        )+
    };
    ( $p:ident array $name:ident $size:literal ) => {
        $p.add_local_var(stringify!($name), $size);
    };
    ( $p:ident start_frame ) => {
         $p.start_frame();
    };
    ( $p:ident end_frame ) => {
         $p.end_frame();
    };
    ( $p:ident $mnem:ident $label:ident ) => {
        $p.add_placeholder_inst(stringify!($mnem), stringify!($label));
    };
    ( $p:ident $mnem:ident ) => {
        parse_asm_line!($p $mnem 0);
    };
    ( $p:ident $mnem:ident $arg:literal ) => {
        $p.add_inst(stringify!($mnem), $arg);
    };
}

macro_rules! add_asm {
 ( [program: $p:ident] $( $mnem:ident $($label:ident)* $($a:literal)* );+; ) => {
       $(
            parse_asm_line!($p $mnem $($label)* $($a)*);
       )+
    };
}

macro_rules! program_from_asm {
    ( $( $mnem:ident $($label:ident)* $($a:literal)* );+; ) => {
       {
           let mut p = Assembler::new();
           p.init();
           add_asm! {
               [program: p]
               $(
                    $mnem $($label)* $($a)*;
               )+
           }
           p.finish();
           p
       }
    };
}

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

impl From<Assembler> for DebugInfo {
    fn from(p: Assembler) -> Self {
        let mut info = DebugInfo::new();
        info.resolved_labels = p.resolved_labels.clone();
        info.call_frames = p.call_frames.clone();
        info.frame_name_for_inst = p.frame_name_for_inst.clone();
        info
    }
}

#[derive(Copy, Clone)]
pub enum LabelType {
    Constant,
    GlobalVar,
    TopLevel,
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
            LabelType::Constant => "C",
            LabelType::GlobalVar => "G",
            LabelType::TopLevel => "T",
            LabelType::InnerLabel => "I",
            LabelType::FrameVar => "F",
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
pub enum AssemblyError {
    NeedToDefineEntryLabel,
    MissingTarget(Inst, String),
    NoSuchOp(i32, String),
}

impl Display for AssemblyError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            MissingTarget(inst, target) => {
                write!(f, "MissingTarget({} \"{}\")", inst, target)
            }
            other => write!(f, "{:?}", other)
        }
    }
}


pub struct Assembler {
    instructions: Vec<Inst>,
    call_frames: HashMap<String, CallFrame>,
    frame_name_for_inst: HashMap<i32, String>,
    cur_frame_name: String,
    global_vars: HashMap<String, i32>,
    constants: HashMap<String, i32>,
    reloc_tab: HashMap<usize, String>,
    resolved_labels: HashMap<i32, ResolvedLabel>,
    encoder: Encoder,
    errors: Vec<AssemblyError>,
}

fn inst_loc_to_addr(loc: usize) -> i32 {
    loc as i32 + SEG_CODE_START
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
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
        self.add_global_var("pc", PC_ADDR);
        self.add_global_var("sp", SP_ADDR);
        self.add_global_var("fp", FP_ADDR);
        for (callcode, name) in isa::ENV_CALLS.iter().enumerate() {
            self.add_constant(name, callcode as i32);
        }
    }

    pub fn add_inst(&mut self, opname: &str, arg: i32) {
        let addr = self.next_inst_addr();
        let inst = match self.encoder.make_inst(opname, arg) {
            None => {
                self.errors.push(AssemblyError::NoSuchOp(
                    addr, opname.to_string()));
                self.instructions.push(Inst {
                    opcode: 0x00,
                    op: OP_INVALID,
                    addr: Some(addr),
                    arg,
                });
                return;
            }
            Some(inst) => inst
        };
        self.instructions.push(Inst {
            addr: Some(addr),
            ..inst
        });
        self.frame_name_for_inst.insert(addr, self.cur_frame_name.clone());
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

    pub fn add_top_level_label(&mut self, name: &str) {
        if self.cur_frame_name != "" {
            self.cur_frame().addr_range.end = self.next_inst_addr();
        }
        self.call_frames.insert(name.to_string(), CallFrame {
            name: name.to_string(),
            addr_range: self.next_inst_addr()..-1,
            frame_vars: HashMap::new(),
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
                self.errors.push(AssemblyError::NeedToDefineEntryLabel);
                self.add_top_level_label(DEFAULT_ENTRY_LABEL);
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

    pub fn add_local_var(&mut self, name: &str, sz: i32) {
        let frame = self.cur_frame();
        frame.frame_vars.insert(
            String::from(name),
            frame.locals_size,
        );
        frame.locals_size += sz;
        if sz > 1 {
            frame.frame_vars.insert(
                format!("{}_len", name),
                sz,
            );
        }
    }

    pub fn add_arg_var(&mut self, name: &str, sz: i32) {
        let frame = self.cur_frame();
        frame.frame_vars.insert(
            String::from(name),
            -frame.args_size - 3, // [..args retaddr savedfp || locals ]
        );
        frame.args_size += sz;
    }

    pub fn add_constant(&mut self, name: &str, value: i32) {
        self.constants.insert(String::from(name), value);
    }

    pub fn start_frame(&mut self) {
        self.add_placeholder_inst("loadi", "fp");
        self.add_placeholder_inst("loadi", "sp");
        self.add_placeholder_inst("storei", "fp");
        let extend_sz = self.cur_frame().locals_size;
        if extend_sz > 0 {
            self.add_inst("addsp", extend_sz);
        }
    }

    pub fn end_frame(&mut self) {
        let drop_sz = self.cur_frame().locals_size;
        if drop_sz > 0 {
            self.add_inst("addsp", -drop_sz);
        }
        self.add_placeholder_inst("storei", "fp");
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
            let value = Assembler::calc_inst_offset(
                label_entry.addr_range.start,
                inst_addr,
            );
            return Some(ResolvedLabel {
                inst_addr,
                target,
                value,
                label_type: LabelType::TopLevel,
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
        let frame_name = self.frame_name_for_inst.get(&inst_addr).unwrap();
        let frame = match self.call_frames.get(frame_name) {
            None => return None,
            Some(f) => f,
        };
        // Local code (inner label)
        if let Some(&addr) = frame.inner_labels.get(&target) {
            let value = Assembler::calc_inst_offset(
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
        if let Some(&value) = frame.frame_vars.get(&target) {
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

    pub fn assemble(&mut self) -> Result<Vec<i32>, Vec<AssemblyError>> {
        let mut errors: Vec<AssemblyError> = self.errors
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
        Ok(
            self.instructions.iter()
                .map(|inst| self.encoder.encode(inst))
                .collect()
        )
    }

    fn calc_inst_offset(target_addr: i32, inst_addr: i32) -> i32 {
        target_addr - inst_addr - 1
    }
}

impl Display for Assembler {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.instructions
            .iter()
            .map(|inst| inst.to_string())
            .collect::<Vec<String>>()
            .join("\n")
        )
    }
}


