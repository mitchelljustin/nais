use ParserError::*;

use crate::tokenizer::{Token, tokenize, TokenType};
use crate::parser::State::param_list;

#[allow(unused)]
mod ast;


/*
(program, [])
(func_defs, [])
(func_def, [(Keyword, "fn"), (Ident, "main"), (LParen, "(")])
(param_list, [])
(ret_ty, [])
(func_body, [])
(local_defs, [])
(local_def, [(Keyword, "let"), (Ident, "x"), (Colon, ":")])
(ty, [(Keyword, "i32")])
(func_body, [])
(stmts, [])
(stmt, [])
(assn, [])
(assn_target, [(Ident, "x")])
(expr, [])
(literal, [(Literal, "3")])

 */

/*
start -> program
program -> func_defs

func_defs -> func_def func_defs
func_defs -> ""

func_def -> "fn" IDENT "(" param_list ")" ret_ty "{" func_body "}"

param_list -> params
param_list -> ""

params -> param "," params
params -> param

param -> IDENT ":" ty

ty -> "i32"
ty -> "[" "i32" ";" LITERAL "]"

ret_ty -> ""
ret_ty -> "->" ty

func_body -> local_defs stmts

local_defs -> local_def local_defs
local_defs -> ""

local_def -> "let" IDENT ":" ty ";"

stmts -> stmt stmts
stmts -> ""

stmt -> assn ";"
stmt -> expr ";"
stmt -> "return" expr ";"

assn -> assn_target "=" expr
assn_target -> IDENT
assn_target -> IDENT "[" expr "]"

expr -> "(" expr ")"
expr -> LITERAL
expr -> IDENT
expr -> bin_expr
expr -> func_call

bin_expr -> product "*" expr
bin_expr -> product "/" expr
product -> expr

bin_expr -> term "+" expr
bin_expr -> term "-" expr
term -> expr

func_call -> IDENT "(" arg_list ")"

arg_list -> ""
arg_list -> args

args -> arg "," args
args -> arg

arg -> expr

*/


#[derive(Debug)]
enum ParserError {
    SyntaxError { last_state: State, tokens_left: Vec<Token> },
}


#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone, PartialEq, Debug)]
enum State {
    start,
    program,
    func_body,
    local_defs,
    local_def,
    stmts,
    stmt,
    assn,
    assn_target,
    expr,
    bin_expr,
    product,
    term,
    func_call,
    arg_list,
    args,
    arg,
    func_defs,
    func_def,
    param_list,
    params,
    param,
    ty,
    ret_ty,

    literal,
    ident,

    ACCEPT,
    REJECT,
}

fn parser_transition(state: State, tokens: &[(TokenType, &str)]) -> (State, usize, bool) {
    use State::*;
    use TokenType::*;
    match (state, tokens) {
        (start, _) =>
            (program, 0, false),
        (program, [(Keyword, "fn"), ..]) =>
            (func_defs, 0, true),
        (program, []) =>
            (ACCEPT, 0, false),

        (func_defs, [(Keyword, "fn"), ..]) =>
            (func_def, 0, true),
        (func_defs, []) =>
            (program, 0, false),

        (func_def, [(Keyword, "fn"), ..]) =>
            (param_list, 3, true),
        (func_def, [(RParen, ")"), ..]) =>
            (ret_ty, 1, false),
        (func_def, [(LBrac, "{"), ..]) =>
            (func_body, 1, false),
        (func_def, [(RBrac, "}"), ..]) =>
            (func_defs, 1, false),

        (ret_ty, [(RArrow, "->"), ..]) =>
            (ty, 1, true),
        (ret_ty, [(LBrac, "{"), ..]) =>
            (func_def, 0, true),

        (param_list, [(Ident, _), ..]) =>
            (params, 0, true),
        (param_list, [_, ..]) =>
            (func_def, 0, true),

        (params, [(Ident, _), ..]) =>
            (param, 0, true),
        (params, [..]) =>
            (func_def, 0, false),

        (param, [(Ident, _), ..]) =>
            (ty, 2, true),
        (param, [(Comma, ","), ..]) =>
            (params, 1, false),

        (func_body, [(Keyword, "let"), ..]) =>
            (local_defs, 0, true),
        (func_body, [(RBrac, "}"), ..]) =>
            (func_def, 0, false),
        (func_body, [_, ..]) =>
            (stmts, 0, false),

        (local_defs, [(Keyword, "let"), ..]) =>
            (local_def, 0, true),
        (local_defs, [..]) =>
            (func_body, 0, false),

        (local_def, [(Keyword, "let"), ..]) =>
            (ty, 3, true),
        (local_def, [(Semi, ";"), ..]) =>
            (local_defs, 1, false),

        (ty, [(Keyword, "i32"), ..]) =>
            (ty, 1, true),
        (ty, [(LSqBrac, "["), (Keyword, "i32"), (Semi, ";"), ..]) =>
            (literal, 3, true),
        // +
        (ty, [(Semi, ";"), ..]) =>
            (local_defs, 1, false),
        (ty, [(RSqBrac, "]"), ..]) =>
            (ty, 3, false),
        // +
        (ty, [..]) =>
            (param, 1, false),

        (stmts, [(RBrac, "}"), ..]) =>
            (func_body, 0, false),
        (stmts, [_, ..]) =>
            (stmt, 0, true),

        (stmt, [(Keyword, "return"), ..]) =>
            (expr, 1, true),
        (stmt, [(Ident, _), (Eq, "="), ..]) =>
            (assn, 0, true),
        (stmt, [(Semi, ";"), ..]) =>
            (stmts, 1, false),

        (assn, [(Ident, _), (Eq, "="), ..]) =>
            (assn_target, 0, true),
        (assn, [(Eq, "="), ..]) =>
            (expr, 1, false),

        (assn_target, [(Ident, _), (LSqBrac, "["), ..]) =>
            (expr, 2, true),
        (assn_target, [(Ident, _), ..]) =>
            (assn, 1, true),
        (assn_target, [(RSqBrac, "]"), ..]) =>
            (assn, 1, false),

        (expr, [(Literal, _), ..]) =>
            (literal, 0, true),
        (expr, [(Semi, ";"), ..]) =>
            (stmt, 0, false),

        (literal, [(Literal, _), (RSqBrac, "]")]) =>
            (ty, 1, true),
        (literal, [(Literal, _), ..]) =>
            (expr, 1, true),

        // ...

        _ => (REJECT, 0, true),
    }
}

type QuickToken<'a> = (TokenType, &'a str);
type Transition<'a> = (State, &'a [QuickToken<'a>]);

#[derive(Debug)]
struct ASTBuilder<'a> {
    txs: Vec<Transition<'a>>,
}


impl<'a> ASTBuilder<'a> {
    fn new(txs: &'a [Transition<'a>]) -> ASTBuilder<'a> {
        ASTBuilder {
            txs: txs.to_vec(),
        }
    }

    fn consume_single(&mut self, expected_state: State) -> Option<&[QuickToken]> {
        let (state, tokens) = self.txs[0];
        if state == expected_state {
            self.txs.remove(0);
            Some(tokens)
        } else {
            None
        }
    }

    fn consume_expr(&mut self) -> Option<ast::Expr> {
        self.consume_single(State::expr)?;
        None
    }

    fn consume_assn_target(&mut self) -> Option<ast::AssnTarget> {
        match self.consume_single(State::assn_target) {
            Some([(TokenType::Ident, name), (TokenType::LSqBrac, "[")]) => {
                let name = name.to_string();
                let index = self.consume_expr()?;
                Some(ast::AssnTarget::ArrayItem {
                    name,
                    index,
                })
            }
            Some([(TokenType::Ident, name)]) => {
                let name = name.to_string();
                Some(ast::AssnTarget::Variable { name })
            }
            _ => None,
        }
    }

    fn consume_stmt(&mut self) -> Option<ast::Stmt> {
        self.consume_single(State::stmt)?;
        match self.txs.remove(0) {
            (State::assn, _) => {
                let target = self.consume_assn_target()?;
                let value = self.consume_expr()?;
                Some(ast::Stmt::Assignment {
                    target,
                    value,
                })
            },
            _ => None,
        }
    }

    fn consume_literal(&mut self) -> Option<ast::Literal> {
        match self.consume_single(State::literal) {
            Some([(TokenType::Literal, val)]) => Some(ast::Literal {
                val: i32::from_str_radix(val, 10).unwrap(), // TODO: handle error
            }),
            _ => None,
        }
    }

    fn consume_ty(&mut self) -> Option<ast::Ty> {
        match self.consume_single(State::ty) {
            Some([(TokenType::Keyword, "i32")]) => Some(ast::Ty::I32),
            Some([(TokenType::LSqBrac, "["), ..]) => {
                let len = self.consume_literal()?;
                Some(ast::Ty::I32Array { len })
            }
            _ => None,
        }
    }

    fn consume_local_def(&mut self) -> Option<ast::VariableDef> {
        match self.consume_single(State::local_def) {
            Some([_, (TokenType::Ident, name), _]) => {
                let name = name.to_string();
                let ty = self.consume_ty()?;
                Some(ast::VariableDef {
                    name,
                    ty,
                })
            }
            _ => None,
        }
    }

    fn consume_func_def(&mut self) -> Option<ast::FuncDef> {
        match self.consume_single(State::func_def) {
            Some([_, (TokenType::Ident, name), _]) => {
                let name = name.to_string();
                self.consume_single(State::param_list);
                let params = Vec::new(); // TODO
                self.consume_single(State::ret_ty);
                let ret_ty = None; // TODO
                self.consume_single(State::func_body)?;
                self.consume_single(State::local_defs)?;
                let locals = self.collect(ASTBuilder::consume_local_def);
                self.consume_single(State::stmts)?;
                let body = self.collect(ASTBuilder::consume_stmt);
                Some(ast::FuncDef {
                    name,
                    params,
                    ret_ty,
                    locals,
                    body,
                })
            }
            _ => None,
        }
    }

    fn collect<T>(&mut self, consume: fn(&mut Self) -> Option<T>) -> Vec<T> {
        let mut vec = Vec::new();
        while let Some(obj) = consume(self) {
            vec.push(obj);
        }
        vec
    }

    pub fn build(&mut self) -> ast::Program {
        let first_tx = self.txs.remove(0);
        if let (State::program, _) = first_tx {
            self.consume_single(State::func_defs);
            let func_defs = self.collect(ASTBuilder::consume_func_def);
            ast::Program {
                func_defs,
            }
        } else {
            panic!("Expected program: {:?}", first_tx);
        }
    }
}

fn parse(tokens: &[Token]) -> Result<ast::Program, ParserError> {
    let tokens_as_tuples = tokens
        .iter()
        .map(|t| (t.ty, t.val.as_str()))
        .collect::<Vec<_>>();
    let mut tokens = &tokens_as_tuples[..];
    let mut state = State::start;
    let mut transitions = Vec::new();
    while state != State::ACCEPT && state != State::REJECT {
        let (next_state, n_eat, emit) = parser_transition(state, &tokens);
        let (eaten_tokens, next_tokens) = tokens.split_at(n_eat);
        let transition = (state, eaten_tokens);
        if emit {
            println!("{:?}", transition);
            transitions.push(transition);
        }
        tokens = next_tokens;
        state = next_state;
    }
    if state == State::REJECT {
        return Err(SyntaxError {
            last_state: transitions.last().unwrap().0,
            tokens_left: tokens.iter().map(Token::from).collect(),
        });
    }
    transitions.push((State::ACCEPT, tokens));
    let root = ASTBuilder::new(&transitions).build();
    Ok(root)
}


mod tests {
    use super::*;

    #[test]
    fn test_simple() {
        let code = "
        fn main() {
            let x: i32;
            x = 3;
        }
        ";
        let tokens = tokenize(&code).unwrap();
        let program = match parse(&tokens) {
            Err(e) => panic!("parser error: {:?}", e),
            Ok(node) => node,
        };
        assert_eq!(program.func_defs.len(), 1);
        assert_eq!(program.func_defs[0].name, "main");
    }
}