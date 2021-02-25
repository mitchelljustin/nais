

#[derive(Debug)]
pub struct SimpleStack {
    stack: Vec<i32>
}

impl SimpleStack {
    pub fn new() -> SimpleStack {
        SimpleStack {
            stack: Vec::new()
        }
    }

    pub fn push(&mut self, imm: i32) {
        self.stack.push(imm);
    }

    pub fn push2(&mut self, imm1: i32, imm2: i32) {
        self.stack.push(imm1);
        self.stack.push(imm2);
    }

    fn _pop_one(&mut self) -> i32 {
        return self.stack.pop().expect("stack too small")
    }

    fn _pop_two(&mut self) -> (i32, i32) {
        if self.stack.len() < 2 {
            panic!("stack too small")
        }
        let x1 = self.stack.pop().unwrap();
        let x2 = self.stack.pop().unwrap();
        return (x1, x2);
    }

    pub fn add(&mut self) {
        let (x1, x2) = self._pop_two();
        self.push(x1 + x2);
    }

    pub fn add_imm(&mut self, imm: i32) {
        let x1 = self._pop_one();
        self.push(x1 + imm);
    }

    pub fn sub(&mut self) {
        let (x1, x2) = self._pop_two();
        self.push(x1 - x2);
    }

    pub fn mul(&mut self, r: &str) {
        println!("{}", r);
    }
}
