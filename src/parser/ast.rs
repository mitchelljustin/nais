#![allow(unused)]

use crate::parser::state::State;
use crate::tokenizer::TokenType;

#[derive(Debug)]
pub struct Program {
    pub func_defs: Vec<FuncDef>,
}

#[derive(Debug)]
pub struct FuncDef {
    pub name: String,
    pub params: Vec<VariableDef>,
    pub ret_ty: Option<Ty>,
    pub locals: Vec<VariableDef>,
    pub body: Vec<Stmt>,
}

#[derive(Debug)]
pub struct VariableDef {
    pub name: String,
    pub ty: Ty,
}

#[derive(Debug, PartialEq)]
pub enum Ty {
    I32,
    I32Array { len: Literal },
}

#[derive(Debug)]
pub enum Stmt {
    Assignment { target: AssnTarget, value: Expr },
    Expr { expr: Expr },
    Return { retval: Expr },
}

#[derive(Debug)]
pub enum AssnTarget {
    Variable { name: String },
    ArrayItem { name: String, index: Expr },
}

#[derive(Debug)]
pub enum Expr {
    Literal { val: Literal },
    Variable { name: String },
    BinExpr { left: Box<Expr>, op: BinOp, right: Box<Expr> },
    FuncCall { func_name: String, args: Vec<Expr> },
}

#[derive(Debug, PartialEq)]
pub struct Literal {
    pub val: i32,
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Sar,
}


#[derive(Debug)]
pub struct Builder {}
