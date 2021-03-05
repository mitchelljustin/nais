use ParserError::*;
use state::State;

use crate::tokenizer::{Token, tokenize, TokenType, QuickToken};

#[allow(unused)]
mod ast;
mod state;


#[derive(Debug)]
enum ParserError {
    SyntaxError { last_state: State, tokens_left: Vec<Token> },
}


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
        let tokens = self.consume_single(State::expr)?;
        if let [(TokenType::Ident, name)] = tokens {
            let name = name.to_string();
            return Some(ast::Expr::Variable { name });
        }
        if let Some(val) = self.consume_literal() {
            return Some(ast::Expr::Literal { val });
        }

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
        let tokens = self.consume_single(State::stmt)?;
        if let [(TokenType::Keyword, "return")] = tokens {
            let retval = self.consume_expr()?;
            return Some(ast::Stmt::Return {
                retval,
            });
        }

        match self.txs.remove(0) {
            (State::assn, _) => {
                let target = self.consume_assn_target()?;
                let value = self.consume_expr()?;
                Some(ast::Stmt::Assignment {
                    target,
                    value,
                })
            }

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

    fn consume_ret_ty(&mut self) -> Option<ast::Ty> {
        if let Some([]) = self.consume_single(State::ret_ty) {
            return None;
        }
        self.consume_ty()
    }

    fn consume_func_def(&mut self) -> Option<ast::FuncDef> {
        match self.consume_single(State::func_def) {
            Some([_, (TokenType::Ident, name), _]) => {
                let name = name.to_string();
                self.consume_single(State::param_list);
                let params = Vec::new(); // TODO
                self.consume_single(State::ret_ty);
                let ret_ty = self.consume_ret_ty();
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
    let mut state = State::START;
    let mut transitions = Vec::new();
    while state != State::ACCEPT && state != State::REJECT {
        let (next_state, n_eat, emit) = state::state_transition(state, &tokens);
        let (eaten_tokens, next_tokens) = tokens.split_at(n_eat);
        let transition = (state, eaten_tokens);
        if emit {
            println!("{:?}", transition);
            transitions.push(transition);
        } else {
            println!("// {:?}", transition);
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
            return x;
        }
        fn f() -> i32 {
            return 1;
        }
        ";
        let tokens = tokenize(&code).unwrap();
        let program = match parse(&tokens) {
            Err(e) => panic!("parser error: {:?}", e),
            Ok(node) => node,
        };
        println!("{:?}", program);
        assert_eq!(program.func_defs.len(), 2);
        let main = &program.func_defs[0];
        assert_eq!(main.name, "main");
        assert_eq!(main.locals[0].name, "x");
        assert_eq!(main.locals[0].ty, ast::Ty::I32);
        match &main.body[0] {
            ast::Stmt::Assignment { target, value } => {
                if let ast::AssnTarget::Variable { name } = target {
                    assert_eq!(name, "x");
                }
                if let ast::Expr::Literal { val: ast::Literal { val } } = value {
                    assert_eq!(*val, 3);
                }
            }
            _ => panic!("expected assignment")
        }
    }
}