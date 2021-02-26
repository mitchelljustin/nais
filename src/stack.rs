use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter, Result};

use MachineStatus::*;

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
           let mut p = Program::new();
           $(
                parse_asm_line!(p $mnem $($label)* $($a)*);
           )+
           p.relocate_all();
           p
       }
    };
}

#[derive(Clone)]
pub struct Program {
    code: Vec<Inst>,
    label_locs: HashMap<String, i32>,
    reloc_tab: Vec<(i32, String)>
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

pub type OpArgs = (i32, i32);

pub struct Op {
    name: &'static str,
    f: fn(&mut Machine, OpArgs),
}

#[derive(Debug, Clone, Copy)]
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
        if let (Some(top), Some(sec)) = (m.pop(), m.pop()) {
            m.push(top);
            m.push(sec);
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

    pub fn printx(m: &mut Machine, (offset, _): OpArgs) {
        if let Some(x) = m.peek(offset) {
            println!("{:08x} ", x);
        }
    }

    pub fn exit(m: &mut Machine, (code, _): OpArgs) {
        if code == 0 {
            m.status = Stopped;
        } else {
            m.status = MachineStatus::Error(MachineError::ProgramExit(code))
        }
    }

    #[allow(dead_code)]
    pub fn jump(m: &mut Machine, (offset, _): OpArgs) {
        m.jump(offset);
    }

    macro_rules! with_overflow {
        ($top:ident $op:tt $arg:ident) => {
            (($top as i64) $op ($arg as i64)) as i32
        };
    }

    macro_rules! binary_op_funcs {
        ( $($name:ident ($operator:tt));+; ) => {
            $(
                pub fn $name(m: &mut Machine, _: OpArgs) {
                    if let (Some(top), Some(sec)) = (m.pop(), m.pop()) {
                        m.push(with_overflow!(top $operator sec));
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
                        m.push(with_overflow!(top $operator arg));
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
                            m.jump(offset);
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

    pub fn sar(m: &mut Machine, (shamt, _): OpArgs) {
        if let Some(top) = m.pop() {
            let top = top >> shamt;
            m.push(top);
        }
    }

    macro_rules! logical_shift_funcs {
        ( $($name:ident ($shop:tt));+; ) => {
            $(
                pub fn $name(m: &mut Machine, (shamt, _): OpArgs) {
                    if let Some(top) = m.pop() {
                        let top = top as u32;
                        let top = (top $shop shamt);
                        let top = top as i32;
                        m.push(top);
                    }
                }
            )+
        };
    }

    logical_shift_funcs! {
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
            exit breakpoint print printx
            beq bne blt bge
            add sub mul div rem and or  xor
            addi subi muli divi remi andi ori xori
            sar shl shr
        );
    }
}

const MAX_CYCLES: usize = 10_000;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MachineError {
    EmptyStackPop,
    PCOutOfBounds,
    StackOffsetOutOfBounds,
    ProgramExit(i32),
    MaxCyclesReached,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum MachineStatus {
    Idle,
    Running,
    Stopped,
    Error(MachineError),
}

pub struct Machine {
    program: Program,
    stack: Vec<i32>,
    status: MachineStatus,
    pc: usize,
    ncycles: usize,
}

impl Debug for Machine {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.debug_struct("Machine")
            .field("status", &self.status)
            .field("stack", &self.stack)
            .field("pc", &self.pc)
            .field("ncycles", &self.ncycles)
            .finish()
    }
}

impl Machine {
    pub fn new(program: &Program) -> Machine {
        Machine {
            program: program.clone(),
            stack: Vec::new(),
            status: Idle,
            pc: 0,
            ncycles: 0,
        }
    }

    pub fn pop(&mut self) -> Option<i32> {
        match self.stack.pop() {
            None => {
                self.status = Error(MachineError::EmptyStackPop);
                return None;
            }
            Some(x) => Some(x)
        }
    }

    fn stack_ref(&mut self, offset: i32) -> Option<&mut i32> {
        let offset = offset as usize;
        let max_offset = self.stack.len();
        if offset >= max_offset {
            self.status = Error(MachineError::StackOffsetOutOfBounds);
            return None
        }
        Some(&mut self.stack[max_offset - offset - 1])
    }

    pub fn peek(&mut self, offset: i32) -> Option<i32> {
        match self.stack_ref(offset) {
            None => None,
            Some(r) => Some(*r)
        }
    }

    pub fn push(&mut self, x: i32) {
        self.stack.push(x)
    }

    pub fn put(&mut self, x: i32, offset: i32) {
        match self.stack_ref(offset) {
            None => {},
            Some(r) => {
                *r = x;
            }
        };
    }

    pub fn jump(&mut self, offset: i32) {
        self.pc = (self.pc as isize + offset as isize) as usize;
    }

    pub fn run(&mut self) {
        self.status = Running;
        while self.status == Running {
            if self.pc >= self.program.len() {
                self.status = Error(MachineError::PCOutOfBounds);
                break;
            }
            let inst = self.program.code[self.pc];
            (inst.op.f)(self, inst.args);
            self.pc += 1;
            self.ncycles += 1;
            if self.ncycles == MAX_CYCLES {
                self.status = Error(MachineError::MaxCyclesReached);
            }
        }
    }
}