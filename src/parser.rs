use state::State;

use crate::parser::state::ParseTable;
use crate::tokenizer::Token;
use crate::tokenizer;

#[macro_use]
mod state;
mod ast;
mod minirust;


#[derive(Debug)]
enum ParserError {
    SyntaxError { last_state: State, tokens_left: Vec<Token> },
}

pub struct Parser {
    table: ParseTable,
}

impl Parser {
    fn parse(&self, _tokens: &[Token]) -> Result<ast::Program, ParserError> {
        unimplemented!()
    }
}

impl From<ParseTable> for Parser {
    fn from(table: ParseTable) -> Self {
        Parser { table }
    }
}

fn parse(tokens: &[Token]) -> Result<ast::Program, ParserError> {
    let parser = Parser::from(minirust::parse_table());
    parser.parse(tokens)
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