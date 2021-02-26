use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};

use MachineStatus::*;

type I = i32;

macro_rules! parse_asm_line {
    ( $p:ident label $label:ident ) => {
        $p.add_label(stringify!($label));
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

macro_rules! assemble {
    ( $( $mnem:ident $($label:ident)* $($a:literal)* );+; ) => {
       {
           let mut program: Program = Program::new();
           $(
                parse_asm_line!(program $mnem $($label)* $($a)*);
           )+
           program.relocate_all();
           program
       }
    };
}

pub struct Program {
    code: Vec<Inst>,
    label_locs: HashMap<String, I>,
    reloc_tab: Vec<(I, String)>
}

impl Program {
    pub fn new() -> Program {
        Program {
            code: Vec::new(),
            label_locs: HashMap::new(),
            reloc_tab: Vec::new(),
        }
    }

    fn last_loc(&self) -> I {
        self.code.len() as I
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
            let mut inst: &mut Inst = &mut self.code[*inst_loc as usize];
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

pub type OpArgs = (I, I);

pub struct Op {
    name: &'static str,
    f: fn(&mut Machine, OpArgs),
}

#[derive(Debug, Clone)]
pub struct Inst {
    pub op: &'static Op,
    pub args: OpArgs,
}

impl Display for Inst {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{} {} {}", self.op.name, self.args.0, self.args.1)
    }
}

impl Debug for Op {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("OpFn")
            .field("name", &self.name)
            .finish()
    }
}

pub mod isa {
    use crate::stack::{MachineError, MachineStatus};
    use crate::stack::MachineStatus::Stopped;

    use super::{Machine, OpArgs};

    pub fn push(m: &mut Machine, (x, _): OpArgs) {
        m.push(x);
    }

    pub fn pop(m: &mut Machine, (n, _): OpArgs) {
        for _ in 0..n {
            m.pop();
        }
    }

    pub fn dup(m: &mut Machine, (offset, _): OpArgs) {
        if let Some(x) = m.peek(offset) {
            m.push(x);
        }
    }

    pub fn put(m: &mut Machine, (offset, _): OpArgs) {
        if let Some(top) = m.pop() {
            m.put(top, offset);
        }
    }

    pub fn swap(m: &mut Machine, _: OpArgs) {
        if let (Some(x1), Some(x2)) = (m.pop(), m.pop()) {
            m.push(x1);
            m.push(x2);
        }
    }

    pub fn breakpoint(m: &mut Machine, _: OpArgs) {
        println!("BREAKPOINT: {:?}", m);
    }

    pub fn print(m: &mut Machine, (offset, _): OpArgs) {
        if let Some(x) = m.peek(offset) {
            println!("{} ", x);
        }
    }

    pub fn exit(m: &mut Machine, (code, _): OpArgs) {
        if code == 0 {
            m.status = Stopped;
        } else {
            m.status = MachineStatus::Error(MachineError::ProgramExit(code))
        }
    }

    pub fn jump(m: &mut Machine, (offset, _): OpArgs) {
        m.pc = (m.pc as isize + offset as isize) as usize;
    }

    macro_rules! binary_op_funcs {
        ( $($name:ident ($operator:tt));+; ) => {
            $(
                pub fn $name(m: &mut Machine, _: OpArgs) {
                    if let (Some(top), Some(sec)) = (m.pop(), m.pop()) {
                        m.push(top $operator sec);
                    }
                }
            )+
        }
    }

    binary_op_funcs! {
        add ( + );
        sub ( - );
        mul ( * );
        div ( / );
        rem ( % );
        and ( & );
        or  ( | );
        xor ( ^ );
    }

    macro_rules! binary_op_imm_funcs {
        ( $($name:ident ($operator:tt));+; ) => {
            $(
                pub fn $name(m: &mut Machine, (arg, _): OpArgs) {
                    if let Some(top) = m.pop() {
                        m.push(top $operator arg);
                    }
                }
            )+
        }
    }

    binary_op_imm_funcs! {
        addi ( + );
        subi ( - );
        muli ( * );
        divi ( / );
        remi ( % );
        andi ( & );
        ori  ( | );
        xori ( ^ );
    }

    macro_rules! branch_cmp_funcs {
        ( $($name:ident ($cmp:tt));+; ) => {
            $(
                pub fn $name(m: &mut Machine, (offset, _): OpArgs) {
                    if let (Some(top), Some(sec)) = (m.pop(), m.pop()) {
                        if sec $cmp top {
                            jump(m, (offset, 0));
                        }
                    }
                }
            )+
        }
    }

    branch_cmp_funcs! {
        beq ( == );
        bne ( != );
        blt ( < );
        bge ( >= );
    }

    macro_rules! shift_op_funcs {
        ( $($name:ident ($shop:tt));+; ) => {
            $(
                pub fn $name(m: &mut Machine, (shamt, _): OpArgs) {
                    if let Some(top) = m.pop() {
                        let top = (top $shop shamt);
                        m.push(top);
                    }
                }
            )+
        };
    }

    shift_op_funcs! {
        shl ( << );
        shr ( >> );
    }

    pub mod ops {
        use super::super::Op;

        macro_rules! register_ops {
            ( $($name:ident)+ ) => {
                $(
                    #[allow(unused, non_upper_case_globals)]
                    pub const $name: &Op = &Op {
                        name: stringify!($name),
                        f: super::$name,
                    };
                )+
            }
        }

        register_ops!(
            push pop dup swap put
            exit breakpoint print
            beq bne blt bge
            add sub mul div rem and or  xor
            addi subi muli divi remi andi ori xori
            shl shr
        );
    }
}

const MAX_CYCLES: isize = 10_000_000;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MachineError {
    EmptyStackPop,
    PCOutOfBounds,
    StackOffsetOutOfBounds,
    ProgramExit(I),
    MaxCyclesReached,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MachineStatus {
    Idle,
    Running,
    Stopped,
    Error(MachineError),
}

#[derive(Debug)]
pub struct Machine {
    stack: Vec<I>,
    status: MachineStatus,
    pc: usize,
}

impl Machine {
    pub fn new() -> Machine {
        Machine {
            stack: Vec::new(),
            status: Idle,
            pc: 0,
        }
    }

    pub fn pop(&mut self) -> Option<I> {
        match self.stack.pop() {
            None => {
                self.status = Error(MachineError::EmptyStackPop);
                return None;
            }
            Some(x) => Some(x)
        }
    }

    fn stack_ref(&mut self, offset: I) -> Option<&mut I> {
        let offset = offset as usize;
        let max_offset = self.stack.len();
        if offset >= max_offset {
            self.status = Error(MachineError::StackOffsetOutOfBounds);
            return None
        }
        Some(&mut self.stack[max_offset - offset - 1])
    }

    pub fn peek(&mut self, offset: I) -> Option<I> {
        match self.stack_ref(offset) {
            None => None,
            Some(r) => Some(*r)
        }
    }

    pub fn push(&mut self, x: I) {
        self.stack.push(x)
    }

    pub fn put(&mut self, x: I, offset: I) {
        match self.stack_ref(offset) {
            None => {},
            Some(r) => {
                *r = x;
            }
        };
    }

    pub fn run(&mut self, program: &Program) -> (isize, MachineStatus, Vec<I>) {
        let mut ncycles: isize = 0;
        self.pc = 0;
        self.stack.clear();
        self.status = Running;
        while self.status == Running {
            if self.pc >= program.len() {
                self.status = Error(MachineError::PCOutOfBounds);
                break;
            }
            let inst = &program.code[self.pc];
            (inst.op.f)(self, inst.args.clone());
            self.pc += 1;
            ncycles += 1;
            if ncycles == MAX_CYCLES {
                self.status = Error(MachineError::MaxCyclesReached);
            }
        }
        (ncycles, self.status, self.stack.clone())
    }
}