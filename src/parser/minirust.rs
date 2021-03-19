use crate::parser::Parser;
use crate::parser::rule::Grammar;

#[allow(unused)]
pub fn parser() -> Parser {
    Parser::from(&grammar())
}

pub fn test() -> [i32; 10] {
    [1,2,3,4,5,6,7,8,9,10]
}

#[allow(unused)]
pub fn grammar() -> Grammar {
    production_rules! {
        START -> program_items EOF;

        program_items -> func_def program_items;
        program_items -> ;

        func_def -> "fn" Ident '(' var_def ')'
                    "->" "i32" '{' func_body '}';

        var_def -> Ident ':' ty;

        func_body -> "let" var_def ';' stmts;

        stmts -> stmt stmts;
        stmts -> ;

        stmt -> assn ';';
        stmt -> return_stmt ';';

        assn -> Ident '=' expr;

        expr -> func_call;
        expr -> Ident;
        expr -> Literal;

        func_call -> Ident '(' expr ')';

        return_stmt -> "return" expr;
    }
}