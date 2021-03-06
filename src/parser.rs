use table::Matcher;

use crate::{ast, tokenizer};
use crate::parser::ParserError::SyntaxError;
use crate::parser::table::{ParseTable, ProductionRule, Symbol};
use crate::tokenizer::{Token, TokenType};

#[macro_use]
mod table;
mod minirust;


#[derive(Debug)]
enum ParserError {
    SyntaxError {
        stack_left: Vec<Matcher>,
        tokens_left: Vec<Token>,
    },
}

pub struct Parser {
    table: ParseTable,
}

pub enum ParseTree {
    Node {
        rule: ProductionRule,
        children: Vec<ParseTree>,
    },
    Terminal {
        token: Token,
    },
}

impl Parser {
    fn parse(&self, tokens: &[Token]) -> Result<ParseTree, ParserError> {
        use table::Matcher::*;

        let mut tokens = tokens.to_vec();
        tokens.push(Token {
            ty: TokenType::EOF,
            val: "$".to_string(),
        });
        let mut stack = vec![NonTerm(Symbol::START)];

        Err(SyntaxError {
            tokens_left: tokens.clone(),
            stack_left: stack.clone(),
        })
    }
}

impl From<ParseTable> for Parser {
    fn from(table: ParseTable) -> Self {
        Parser { table }
    }
}

fn parse(tokens: &[Token]) -> Result<ast::Node, ParserError> {
    let parser = Parser::from(minirust::parse_table());
    let parse_tree = parser.parse(tokens)?;
    Ok(ast::Node::from(parse_tree))
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
        let tokens = tokenizer::tokenize(&code).unwrap();
        let _program = match parse(&tokens) {
            Err(e) => panic!("parser error: {:?}", e),
            Ok(node) => node,
        };
    }
}