use std::iter::FromIterator;

use rule::Matcher;

use crate::{ast, tokenizer};
use crate::parser::ParserError::SyntaxError;
use crate::parser::rule::{Grammar, ParseTable, ProductionRule, Symbol};
use crate::parser::rule::Matcher::NonTerm;
use crate::tokenizer::Token;

#[macro_use]
mod rule;
mod minirust;


#[derive(Debug)]
pub enum ParserError {
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
enum Decision {
    Rule(ProductionRule),
    Terminal(Token),
}

#[derive(Debug, Clone)]
pub enum ParseTree {
    NonTerminal {
        rule: ProductionRule,
        children: Vec<ParseTree>,
    },
    Terminal {
        token: Token,
    },
}

impl FromIterator<Decision> for ParseTree {
    fn from_iter<I: IntoIterator<Item=Decision>>(iter: I) -> Self {
        ParseTree::Terminal { token: Token::EOF }
    }
}

impl Parser {
    pub fn parse(&self, input: &[Token]) -> Result<ParseTree, ParserError> {
        use rule::Matcher::*;

        let mut input = input.to_vec();
        input.push(Token::EOF);
        let mut stack = vec![NonTerm(Symbol::START)];
        let mut derivation = vec![];
        while !stack.is_empty() {
            let top = match stack.pop() {
                Some(top) => top,
                _ => return Err(SyntaxError { top: None, input, stack }),
            };
            match &top {
                NonTerm(symbol) => {
                    let rule = match self.table.find_rule(symbol, &input) {
                        Some(rule) => rule,
                        None => return Err(SyntaxError { input, stack, top: Some(top.clone()) }),
                    };
                    derivation.push(Decision::Rule(rule.clone()));
                    let mut new_matchers = rule.rhs;
                    new_matchers.reverse();
                    stack.extend_from_slice(&new_matchers);
                }
                Term(matcher) => {
                    if input.is_empty() {
                        return Err(SyntaxError { input, stack, top: Some(top.clone()) });
                    }
                    let token = input.remove(0);
                    if !matcher.matches(&token) {
                        return Err(SyntaxError { input, stack, top: Some(top.clone()) });
                    }
                    derivation.push(Decision::Terminal(token));
                }
            }
        }
        Ok(ParseTree::from_iter(derivation))
    }
}

impl From<ParseTable> for Parser {
    fn from(table: ParseTable) -> Self {
        Parser { table }
    }
}

impl From<&Grammar> for Parser {
    fn from(grammar: &Grammar) -> Self {
        Parser::from(ParseTable::from(grammar))
    }
}

#[allow(unused)]
fn parse(tokens: &[Token]) -> Result<ast::Node, ParserError> {
    let parser = minirust::parser();
    let parse_tree = parser.parse(tokens)?;
    Ok(ast::Node::from(parse_tree))
}


mod tests {
    use crate::parser::rule::Grammar;

    use super::*;

    fn medium_grammar() -> Grammar {
        production_rules! {
            START -> expr EOF;

            expr -> Literal '+' expr;
            expr -> Literal;
        }
    }

    #[test]
    fn test_simple() -> Result<(), ParserError> {
        let code = "3 + 4";
        let tokens = tokenizer::tokenize(&code).unwrap();
        let parser = Parser::from(&medium_grammar());
        let node = parser.parse(&tokens)?;
        println!("{:?}", node);
        Ok(())
    }
}