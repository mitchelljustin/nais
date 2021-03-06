use crate::parser::Parser;
use crate::parser::rule::Grammar;

#[allow(unused)]
pub fn parser() -> Parser {
    Parser::from(grammar())
}

#[allow(unused)]
pub fn grammar() -> Grammar {
    production_rules! {
        START -> program EOF;

        program -> func_defs;

        func_defs -> func_def func_defs;
        func_defs -> ;

        func_def -> "fn" Ident '(' param_list ')' ret_ty '{' func_body '}';

        param_list -> params;
        param_list -> ;

        // params -> param ',' params;
        params -> param;

        param -> Ident ':' ty;

        ty -> ty_prim;
        // ty -> ty_name;
        ty -> '[' ty ';' Literal ']';
        ty -> '*' ty;

        ty_prim -> "i32";

        // ty_name -> Ident;

        ret_ty -> r_arrow ty;
        ret_ty -> ;

        r_arrow -> '-' '>';

        func_body -> local_defs stmts;

        local_defs -> local_def local_defs;
        local_defs -> ;

        local_def -> "let" Ident ':' ty ';';

        stmts -> stmt stmts;
        stmts -> ;

        stmt -> expr ';';
        stmt -> assn ';';
        stmt -> return_stmt ';';
        // stmt -> if_stmt;
        // stmt -> while_stmt;

        expr -> '(' expr ')';
        expr -> var;
        expr -> literal;
        expr -> bin_expr;
        expr -> array_item;
        expr -> func_call;

        var -> Ident;

        literal -> Literal;

        bin_expr -> expr bin_op expr;

        bin_op -> '+';
        bin_op -> '-';

        array_item -> expr '[' expr ']';

        func_call -> Ident '(' arg_list ')';

        arg_list -> args;
        arg_list -> ;

        args -> arg ',' args;
        args -> arg;

        arg -> expr;

        assn -> assn_target '=' expr;

        assn_target -> var;
        assn_target -> array_item;

        return_stmt -> "return" expr;

        // if_stmt     -> "if" cond '{' stmts '}';
        // while_stmt  -> "while" cond '{' stmts '}';

        // cond -> expr cmp_op expr;

        // cmp_op -> '>';
        // cmp_op -> '=' '=';
        // cmp_op -> '<';
    }
}