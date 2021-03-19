use std::collections::HashMap;
use std::fmt::Debug;

use crate::tokenizer;
use crate::tokenizer::Token;

#[allow(non_camel_case_types, unused)]
#[derive(Copy, Clone, PartialEq, Debug, Eq, Hash)]
pub enum Symbol {
    START,

    program_items,
    program_item,
    func_def,
    struct_def,
    struct_def_items,
    struct_def_item,
    const_def,
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
    param_list,
    params,
    var_def,
    ty,
    prim_ty,
    ret_ty,
    if_stmt,
    while_stmt,
    return_stmt,
    cond,
    bin_op,
    cmp_op,
    array_item,
    struct_item,
    deref,
    deref_target,

    undefined,
}


#[derive(Debug, Clone, PartialEq)]
pub struct TokenMatcher {
    token: Token,
    exact_val: bool,
}

impl TokenMatcher {
    pub fn matches(&self, tok: &Token) -> bool {
        if self.exact_val {
            &self.token == tok
        } else {
            use Token::*;
            match (tok, &self.token) {
                (Unknown(_), Unknown(_)) => true,
                (Space(_), Space(_)) => true,
                (Ident(_), Ident(_)) => true,
                (Keyword(_), Keyword(_)) => true,
                (Literal(_), Literal(_)) => true,
                (Sym(_), Sym(_)) => true,
                (EOF, EOF) => true,
                _ => false,
            }
        }
    }

    pub fn prefix_matches(prefix: &[TokenMatcher], tokens: &[Token]) -> bool {
        prefix
            .iter()
            .zip(tokens)
            .all(|(m, tok)| m.matches(tok))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Matcher {
    NonTerm(Symbol),
    Term(TokenMatcher),
}

impl Matcher {
    fn is_terminal(&self) -> bool {
        match self {
            Matcher::Term(_) => true,
            Matcher::NonTerm(_) => false,
        }
    }
}

impl From<Symbol> for Matcher {
    fn from(symbol: Symbol) -> Self {
        Matcher::NonTerm(symbol)
    }
}

impl From<char> for Matcher {
    fn from(ch: char) -> Self {
        Matcher::from(ch.to_string().as_str())
    }
}

impl From<&str> for Matcher {
    fn from(text: &str) -> Self {
        let token = tokenizer::tokenize(text).unwrap().get(0).unwrap().to_owned();
        let matcher = TokenMatcher {
            token,
            exact_val: true,
        };
        Matcher::Term(matcher)
    }
}

pub enum TokenMatcherTypeAlias {
    Ident,
    Literal,
    EOF,
}

impl From<TokenMatcherTypeAlias> for Matcher {
    fn from(alias: TokenMatcherTypeAlias) -> Self {
        let token = match alias {
            TokenMatcherTypeAlias::Ident =>
                Token::Ident(String::new()),
            TokenMatcherTypeAlias::Literal =>
                Token::Literal(String::new()),
            TokenMatcherTypeAlias::EOF =>
                Token::EOF,
        };
        Matcher::Term(TokenMatcher {
            token,
            exact_val: false,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProductionRule {
    pub lhs: Symbol,
    pub rhs: Vec<Matcher>,
}

pub type Grammar = Vec<ProductionRule>;


#[macro_export]
macro_rules! production_rules {
    {$(
        $lhs:ident -> $( $matcher:expr )*;
    )+} => {
        {
            #[allow(unused)]
            use crate::parser::rule::Symbol::*;
            #[allow(unused)]
            use crate::parser::rule::Matcher::*;
            #[allow(unused)]
            use crate::parser::rule::TokenMatcherTypeAlias::*;

            use crate::parser::rule::{Matcher, ProductionRule};

            vec![
                $(
                    ProductionRule {
                        lhs: $lhs,
                        rhs: vec![ $(Matcher::from($matcher),)* ]
                    },
                )+
            ]
        }
    };
}

impl From<Matcher> for Option<TokenMatcher> {
    fn from(m: Matcher) -> Self {
        match m {
            Matcher::Term(tm) => Some(tm),
            _ => None,
        }
    }
}

const PREFIX_MAX_LEN: usize = 5;

#[derive(Debug, Clone)]
struct ParseTableEntry {
    rule: ProductionRule,
    own_prefix: Vec<TokenMatcher>,
    take_if_begins_with: Vec<Vec<TokenMatcher>>,
}

#[derive(Debug)]
pub struct ParseTable {
    entries: Vec<ParseTableEntry>,
}

impl ParseTable {
    fn new() -> ParseTable {
        ParseTable {
            entries: Vec::new(),
        }
    }

    pub fn add_rule(&mut self, rule: &ProductionRule) {
        let terminals = rule.rhs
            .iter()
            .take_while(|m| m.is_terminal())
            .map(|m| match m {
                Matcher::Term(tm) => tm,
                Matcher::NonTerm(_) => panic!(),
            })
            .cloned()
            .collect::<Vec<_>>();
        self.entries.push(ParseTableEntry {
            rule: rule.to_owned(),
            own_prefix: terminals.clone(),
            take_if_begins_with: vec![terminals],
        })
    }

    pub fn disambiguate(&mut self) {
        let mut prefixes_for_sym = HashMap::new();
        for entry in self.entries.iter() {
            let symbol = entry.rule.lhs;
            if prefixes_for_sym.get(&symbol).is_none() {
                prefixes_for_sym.insert(symbol, Vec::<Vec<TokenMatcher>>::new());
            }
            prefixes_for_sym
                .get_mut(&symbol)
                .unwrap()
                .push(entry.own_prefix.clone());
        }
        for entry in self.entries.iter_mut() {
            let own_prefix = entry.own_prefix.clone();
            if own_prefix.len() >= PREFIX_MAX_LEN {
                continue;
            }
            let first_nonterm = match entry.rule.rhs
                .iter()
                .find_map(|m| match m {
                    Matcher::NonTerm(s) => Some(s),
                    Matcher::Term(_) => None,
                }) {
                Some(t) => t,
                None => continue,
            };
            let extra_prefixes = prefixes_for_sym.get(first_nonterm).unwrap();
            let new_prefixes = extra_prefixes
                .iter()
                .map(|prefix| vec![own_prefix.clone(), prefix.clone()].concat())
                .collect();
            entry.take_if_begins_with = new_prefixes;
        }
    }

    pub fn find_rule(&self, lhs: &Symbol, tokens: &[Token]) -> Option<ProductionRule> {
        for entry in self.entries.iter() {
            if entry.rule.lhs != *lhs {
                continue;
            }
            for prefix in entry.take_if_begins_with.iter() {
                if TokenMatcher::prefix_matches(prefix, tokens) {
                    return Some(entry.rule.to_owned());
                }
            }
        }
        None
    }
}

impl From<&Grammar> for ParseTable {
    fn from(grammar: &Grammar) -> Self {
        let mut table = ParseTable::new();
        for rule in grammar.iter() {
            table.add_rule(rule);
        }
        table.disambiguate();
        table
    }
}


mod tests {
    use crate::tokenizer::{ident, sym, literal};

    use super::*;

    #[test]
    fn test_simple_grammar() {
        let simple_grammar = production_rules! {
            START -> expr EOF;

            expr -> Ident '+' Literal;
            expr -> Ident;
            expr -> Literal;
        };
        let table = ParseTable::from(&simple_grammar);
        assert_eq!(
            table.find_rule(&Symbol::START, &[Token::EOF]),
            None,
        );
        assert_eq!(
            table.find_rule(&Symbol::START, &[ident("x"), Token::EOF]).as_ref(),
            simple_grammar.get(0),
        );
        assert_eq!(
            table.find_rule(&Symbol::expr, &[ident("x"), Token::EOF]).as_ref(),
            simple_grammar.get(1),
        );
        assert_eq!(
            table.find_rule(&Symbol::expr, &[ident("x"), sym("+"), literal("3"), Token::EOF]).as_ref(),
            simple_grammar.get(3),
        );
    }
}