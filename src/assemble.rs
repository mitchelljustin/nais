use std::collections::HashMap;

use crate::isa::{Inst, Op, OpArgs};
use std::fmt::{Display, Formatter, Result};

macro_rules! assemble {
    ( $( $mnem:ident $($label:ident)* $($a:literal)* );+; ) => {
       {
           let mut p = Program::new();
           $(
                parse_asm_line!(p $mnem $($label)* $($a)*);
           )+
           p.relocate_all();
           p
       }
    };
}

macro_rules! parse_asm_line {
    ( $p:ident label $label:ident ) => {
        $p.add_label(stringify!($label));
    };
    ( $p:ident call $label:ident ) => {
        $p.add_placeholder_inst(isa::ops::jal, stringify!($label));
    };
    ( $p:ident $mnem:ident ) => {
        parse_asm_line!($p $mnem 0);
    };
    ( $p:ident $mnem:ident $a1:literal ) => {
        parse_asm_line!($p $mnem $a1 0);
    };
    ( $p:ident $mnem:ident $a1:literal $a2:literal ) => {
        $p.add_inst(isa::ops::$mnem, ($a1, $a2));
    };
    ( $p:ident $mnem:ident $label:ident ) => {
        $p.add_placeholder_inst(isa::ops::$mnem, stringify!($label));
    };
}

#[derive(Clone)]
pub struct Program {
    code: Vec<Inst>,
    label_locs: HashMap<String, i32>,
    reloc_tab: Vec<(i32, String)>
}

impl Display for Program {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        for (i, inst) in self.code.iter().enumerate() {
            if let Err(e) = write!(f, "{:04x}: {}\n", i, inst) {
                return Err(e)
            }
        }
        Ok(())
    }
}

impl Program {
    pub fn new() -> Program {
        Program {
            code: Vec::new(),
            label_locs: HashMap::new(),
            reloc_tab: Vec::new(),
        }
    }

    fn last_loc(&self) -> i32 {
        self.code.len() as i32
    }

    pub fn inst_at(&self, idx: usize) -> Inst {
        self.code[idx]
    }

    pub fn add_inst(&mut self, op: &'static Op, args: OpArgs) {
        self.code.push(Inst {
            op,
            args,
        });
    }

    pub fn add_placeholder_inst(&mut self, op: &'static Op, label: &str) {
        self.reloc_tab.push((self.last_loc(), String::from(label)));
        self.code.push(Inst {
            op,
            args: (0, 0),
        });
    }

    pub fn add_label(&mut self, name: &str) {
        self.label_locs.insert(String::from(name), self.last_loc());
    }

    pub fn len(&self) -> usize {
        return self.code.len()
    }

    pub fn relocate_all(&mut self) {
        for (inst_loc, label) in self.reloc_tab.iter() {
            let inst = &mut self.code[*inst_loc as usize];
            if let Some(target_loc) = self.label_locs.get(label) {
                let offset = *target_loc - *inst_loc - 1;
                inst.args.0 = offset;
            } else {
                panic!("No such label: {}", label);
            }
        }
        self.reloc_tab.clear();
    }
}
