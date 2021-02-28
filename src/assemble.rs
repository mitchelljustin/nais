use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::fmt;

use crate::assemble::AssemblyError::MissingTarget;
use crate::constants::SEG_CODE_START;
use crate::isa::{Encoder, Inst, OP_INVALID};
use crate::isa;
use crate::machine::{CallFrame, DebugInfo};
use crate::unwrap_or_return;

macro_rules! parse_asm_line {
    ( $p:ident label $label:ident ) => {
        $p.add_subroutine(stringify!($label));
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
    ( $p:ident loadf $name:ident ) => {
        parse_asm_line!($p loadi fp );
        parse_asm_line!($p load $name );
    };
    ( $p:ident storef $name:ident ) => {
        parse_asm_line!($p loadi fp );
        parse_asm_line!($p store $name );
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
           p
       }
    };
}

#[derive(Copy, Clone)]
pub enum LabelType {
    Constant,
    GlobalVar,
    Subroutine,
    InnerLabel,
    FrameVar,
}

impl Display for LabelType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            LabelType::Constant => "C",
            LabelType::GlobalVar => "G",
            LabelType::Subroutine => "S",
            LabelType::InnerLabel => "I",
            LabelType::FrameVar => "F",
        })
    }
}


#[derive(Clone)]
struct LabelEntry {
    name: String,
    start_addr: i32,
    frame_labels: HashMap<String, i32>,
    locals_size: i32,
    args_size: i32,
    label_type: LabelType,
}

#[derive(Clone, Debug)]
pub enum AssemblyError {
    MissingTarget(Inst, String),
    NoSuchOp(usize, String),
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
    label_entries: HashMap<String, LabelEntry>,
    global_vars: HashMap<String, i32>,
    constants: HashMap<String, i32>,
    frame_name_for_inst: Vec<String>,
    cur_frame_name: String,
    reloc_tab: HashMap<usize, String>,
    resolved_labels: HashMap<String, (i32, LabelType)>,
    encoder: Encoder,
    errors: Vec<AssemblyError>,
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            instructions: Vec::new(),
            label_entries: HashMap::new(),
            global_vars: HashMap::new(),
            constants: HashMap::new(),
            frame_name_for_inst: Vec::new(),
            reloc_tab: HashMap::new(),
            resolved_labels: HashMap::new(),
            cur_frame_name: String::new(),
            encoder: Encoder::new(),
            errors: Vec::new(),
        }
    }

    pub fn init(&mut self) {
        self.add_global_var("pc", 0);
        self.add_global_var("sp", 1);
        self.add_global_var("fp", 2);
        self.add_subroutine("_entry");
        for (callcode, name) in isa::ENV_CALLS.iter().enumerate() {
            self.add_constant(name, callcode as i32);
        }
    }

    pub fn add_inst(&mut self, opname: &str, arg: i32) {
        let loc = self.instructions.len();
        let addr = Some(SEG_CODE_START + loc as i32);
        let inst = match self.encoder.make_inst(opname, arg) {
            None => {
                self.errors.push(AssemblyError::NoSuchOp(
                    loc, String::from(opname)));
                self.instructions.push(Inst {
                    opcode: 0x00,
                    op: OP_INVALID,
                    arg,
                    addr,
                });
                return;
            }
            Some(inst) => inst
        };
        self.instructions.push(Inst {
            addr,
            ..inst
        });
        self.frame_name_for_inst.push(self.cur_frame_name.clone());
    }

    pub fn add_placeholder_inst(&mut self, opname: &str, label: &str) {
        self.reloc_tab.insert(self.instructions.len(), String::from(label));
        self.add_inst(opname, 0);
    }

    pub fn add_subroutine(&mut self, name: &str) {
        self.add_label_entry(name, LabelType::Subroutine);
        self.cur_frame_name = name.to_string();
    }

    pub fn add_inner_label(&mut self, name: &str) {
        self.add_label_entry(name, LabelType::InnerLabel);
    }

    fn add_label_entry(&mut self, name: &str, label_type: LabelType) {
        self.label_entries.insert(name.to_string(), LabelEntry {
            label_type,
            name: name.to_string(),
            start_addr: self.instructions.len() as i32,
            frame_labels: HashMap::new(),
            locals_size: 0,
            args_size: 0,
        });
    }

    fn cur_frame(&mut self) -> &mut LabelEntry {
        self.label_entries
            .get_mut(&self.cur_frame_name)
            .expect("current label not found")
    }

    pub fn add_global_var(&mut self, name: &str, abs_loc: i32) {
        self.global_vars.insert(
            String::from(name),
            abs_loc,
        );
    }

    pub fn add_local_var(&mut self, name: &str, sz: i32) {
        let frame = self.cur_frame();
        frame.frame_labels.insert(
            String::from(name),
            frame.locals_size,
        );
        frame.locals_size += sz;
        if sz > 1 {
            frame.frame_labels.insert(
                format!("{}_len", name),
                sz,
            );
        }
    }

    pub fn add_arg_var(&mut self, name: &str, sz: i32) {
        let label_entry = self.cur_frame();
        label_entry.frame_labels.insert(
            String::from(name),
            -label_entry.args_size - 3, // [..args retaddr savedfp || locals ]
        );
        label_entry.args_size += sz;
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
            self.add_inst("extend", extend_sz);
        }
    }

    pub fn end_frame(&mut self) {
        let drop_sz = self.cur_frame().locals_size;
        if drop_sz > 0 {
            self.add_inst("drop", drop_sz);
        }
        self.add_placeholder_inst("storei", "fp");
    }

    fn resolve_label(&self, target: &str, inst_loc: usize) -> Option<(i32, LabelType)> {
        if let Some(&entry) = self.resolved_labels.get(target) {
            return Some(entry);
        }
        // Constant
        if let Some(&value) = self.constants.get(target) {
            return Some((value, LabelType::Constant));
        }
        // Code label
        if let Some(label_entry) = self.label_entries.get(target) {
            let value = Assembler::calc_inst_offset(
                label_entry.start_addr,
                inst_loc,
            );
            return Some((value, LabelType::Subroutine));
        }
        // Global variable
        if let Some(&value) = self.global_vars.get(target) {
            return Some((value, LabelType::GlobalVar));
        }
        // Local scope
        let scope_label = &self.frame_name_for_inst[inst_loc];
        let scope_entry = self.label_entries.get(scope_label).unwrap();
        let inner_label_name = Assembler::make_inner_label(scope_label, target);
        // Local code (inner label)
        if let Some(label_entry) = self.label_entries.get(&inner_label_name) {
            let value = Assembler::calc_inst_offset(
                label_entry.start_addr,
                inst_loc,
            );
            return Some((value, LabelType::InnerLabel));
        }
        // Local frame var
        if let Some(&value) = scope_entry.frame_labels.get(target) {
            return Some((value, LabelType::FrameVar));
        }
        None
    }

    pub fn relocate(&mut self) -> Vec<(usize, String)> {
        let mut unrelocated = Vec::<(usize, String)>::new();
        let mut arg_updates = Vec::<(usize, i32)>::new();
        for (inst_loc, target) in self.reloc_tab.iter() {
            match self.resolve_label(target, *inst_loc) {
                Some((value, label_type)) => {
                    self.resolved_labels.insert(target.clone(), (value, label_type));
                    arg_updates.push((*inst_loc, value));
                }
                None => {
                    unrelocated.push((*inst_loc, target.clone()));
                }
            }
        }
        for (loc, arg) in arg_updates.into_iter() {
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

    fn calc_inst_offset(target_loc: i32, inst_loc: usize) -> i32 {
        target_loc - (inst_loc as i32) - 1
    }

    fn make_inner_label(scope_label: &str, name: &str) -> String {
        format!("{}.{}", scope_label, name)
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

impl DebugInfo for Assembler {
    fn resolved_label_for_inst(&self, addr: i32) -> Option<(String, String)> {
        let loc = (addr - SEG_CODE_START) as usize;
        let label = match self.reloc_tab.get(&loc) {
            None => return None,
            Some(l) => l.clone()
        };
        let label_type = match self.resolved_labels.get(&label) {
            None => return None,
            Some((_, t)) => *t
        };
        Some((label, label_type.to_string()))
    }

    fn call_frame_for_inst(&self, addr: i32) -> Option<CallFrame> {
        let loc = (addr - SEG_CODE_START) as usize;
        let name = unwrap_or_return!(self.frame_name_for_inst.get(loc));
        let LabelEntry {
            name,
            start_addr,
            frame_labels,
            ..
        } = unwrap_or_return!(self.label_entries.get(name).cloned());
        let (locals, args) = frame_labels.into_iter().partition(
            |&(_, offset)| offset >= 0
        );
        Some(
            CallFrame {
                name,
                start_addr,
                locals,
                args,
            }
        )
    }

    fn value_for_label(&self, name: &str) -> Option<(i32, String)> {
        self.resolved_labels.get(name)
            .map(|&(val, t)| (val, t.to_string()))
    }
}