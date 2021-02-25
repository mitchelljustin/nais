use std::fmt::Debug;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Sub};

macro_rules! impl_simple_op {
    { $name:ident ($op:tt) } => {
        #[allow(unused)]
        pub fn $name(&mut self) {
            let a = match self.pop() {
                None => return,
                Some(x) => x
            };
            let b = match self.pop() {
                None => return,
                Some(x) => x
            };
            let c = a $op b;
            self.push(c);
        }
    };
}

macro_rules! assemble_inst {
    ($name:ident) => {
        Op::$name
    };
    ($name:ident $($arg:literal)*) => {
        Op::$name($($arg),*)
    }
}

macro_rules! assemble {
    { (word $word:ty)
        $($name:ident $($arg:literal)*);+;
    } => {
        {
            let mut program: Vec<Op<$word>> = Vec::new();
            $(
                let op = assemble_inst!($name $($arg)*);
                program.push(op);
            )+
            program
        }
    };
}

#[derive(Debug)]
#[allow(non_camel_case_types, unused)]
pub enum Op<I: Debug> {
    push(I),
    pop,
    dup,

    add,
    sub,
    mul,
    div,
    rem,
    and,
    xor,

    jump(I),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MachineState {
    Idle,
    Running,
    Stopped,
    Error(&'static str),
}

#[derive(Debug)]
pub struct Machine<I> {
    stack: Vec<I>,
    stack_size: usize,
    pc: usize,
    state: MachineState,
}

impl<I> Machine<I>
    where I:
    Add<Output=I> +
    Sub<Output=I> +
    Mul<Output=I> +
    Div<Output=I> +
    Rem<Output=I> +
    BitAnd<Output=I> +
    BitOr<Output=I> +
    BitXor<Output=I> +
    Into<i64> +
    Debug + Ord + Copy {
    pub fn new(stack_size: usize) -> Self {
        Machine {
            stack_size,
            pc: 0,
            state: MachineState::Idle,
            stack: Vec::with_capacity(stack_size),
        }
    }

    fn exec_inst(&mut self, inst: &Op<I>) {
        match inst {
            Op::push(x) => { self.push(*x); }
            Op::pop => { self.pop(); }
            Op::dup => { self.dup(); }
            Op::add => { self.add(); }
            Op::sub => { self.sub(); }
            Op::mul => { self.mul(); }
            Op::div => { self.div(); }
            Op::rem => { self.rem(); }
            Op::and => { self.and(); }
            Op::xor => { self.xor(); }
            Op::jump(x) => { self.jump(*x); }
        }
    }

    pub fn run(&mut self, program: &Vec<Op<I>>) -> (MachineState, Vec<I>) {
        self.stack.clear();
        self.state = MachineState::Running;
        self.pc = 0;
        while self.state == MachineState::Running {
            let inst = &program[self.pc];
            self.exec_inst(inst);
            self.pc += 1;
            if self.pc >= program.len() && self.state == MachineState::Running {
                self.state = MachineState::Stopped;
            }
        }

        (self.state, self.stack.clone())
    }

    pub fn pop(&mut self) -> Option<I> {
        match self.stack.pop() {
            Some(x) => Some(x),
            None => {
                self.state = MachineState::Error("popped empty stack");
                None
            }
        }
    }

    pub fn push(&mut self, x: I) {
        self.stack.push(x);
    }

    pub fn dup(&mut self) {
        if let Some(x) = self.stack.pop() {
            self.stack.push(x);
            self.stack.push(x);
        }
    }

    pub fn jump(&mut self, offset: I) {
        self.pc += offset.into() as usize;
    }

    impl_simple_op!( add (+) );
    impl_simple_op!( sub (-) );
    impl_simple_op!( mul (*) );
    impl_simple_op!( div (/) );
    impl_simple_op!( rem (%) );
    impl_simple_op!( and (&) );
    impl_simple_op!( or  (|) );
    impl_simple_op!( xor (^) );
}
