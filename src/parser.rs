use table::Matcher;

use crate::{ast, tokenizer};
use crate::parser::ParserError::SyntaxError;
use crate::parser::table::{Grammar, ParseTable, ProductionRule, Symbol};
use crate::tokenizer::Token;

#[macro_use]
mod table;
mod minirust;


#[derive(Debug)]
enum ParserError {
    SyntaxError {
        top: Option<Matcher>,
        stack: Vec<Matcher>,
        input: Vec<Token>,
    },
}

pub struct Parser {
    table: ParseTable,
}

#[derive(Debug, Clone)]
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
    fn parse(&self, input: &[Token]) -> Result<ParseTree, ParserError> {
        use table::Matcher::*;

        let mut input = input.to_vec();
        input.push(Token::EOF);
        let mut stack = vec![NonTerm(Symbol::START)];
        while !stack.is_empty() {
            let top = match stack.pop() {
                Some(top) => top,
                _ => return Err(SyntaxError { top: None, input, stack }),
            };
            match &top {
                NonTerm(symbol) => {
                    println!("pop {:?}", symbol);
                    let rule = match self.table.get(symbol, &input) {
                        Some(rule) => rule,
                        None => return Err(SyntaxError { input, stack, top: Some(top.clone()) }),
                    };
                    let new_node = ParseTree::Node {
                        rule: rule.clone(),
                        children: vec![],
                    };
                    // TODO: append rule to try
                    println!("append {:?}", rule.rhs);
                    let mut new_matchers = rule.rhs;
                    new_matchers.reverse();
                    stack.extend_from_slice(&new_matchers);
                }
                Term(tm) => {
                    println!("pop {:?}", tm);
                    if input.is_empty() {
                        return Err(SyntaxError { input, stack, top: Some(top.clone()) });
                    }
                    let token = input.remove(0);
                    if !tm.matches(&token) {
                        return Err(SyntaxError { input, stack, top: Some(top.clone()) });
                    }
                    println!("consume {:?}", token);

                }
            }
        }
        Ok(ParseTree::Terminal {
            token: Token::EOF,
        })
    }
}

impl From<ParseTable> for Parser {
    fn from(table: ParseTable) -> Self {
        Parser { table }
    }
}

impl From<Grammar> for Parser {
    fn from(grammar: Grammar) -> Self {
        Parser::from(ParseTable::from(grammar))
    }
}

fn parse(tokens: &[Token]) -> Result<ast::Node, ParserError> {
    let parser = minirust::parser();
    let parse_tree = parser.parse(tokens)?;
    Ok(ast::Node::from(parse_tree))
}


mod tests {
    use crate::parser::table::Grammar;

    use super::*;

    fn medium_grammar() -> Grammar {
        production_rules! {
            START -> program;

            program -> expr;

            expr -> literal '+' literal;

            literal -> Literal;
        }
    }

    #[test]
    fn test_simple() -> Result<(), ParserError> {
        let code = "3 + 4";
        let tokens = tokenizer::tokenize(&code).unwrap();
        let parser = Parser::from(medium_grammar());
        let node = parser.parse(&tokens)?;
        println!("{:?}", node);
        Ok(())
    }
}