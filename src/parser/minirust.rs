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

        program -> program_items;

        program_items -> program_item program_items;
        program_items -> ;

        program_item -> func_def;
        program_item -> type_def;

        func_def -> "fn" Ident '(' param_list ')' ret_ty '{' func_body '}';

        param_list -> params;
        param_list -> ;

        params -> param ',' params;
        params -> param;

        param -> Ident ':' ty;

        ty -> "i32";
        ty -> '*' ty;
        ty -> '[' ty ';' Literal ']';
        ty -> Ident;

        ret_ty -> "->" ty;
        ret_ty -> ;

        func_body -> local_defs stmts;

        local_defs -> local_def local_defs;
        local_defs -> ;

        local_def -> "let" Ident ':' ty ';';

        stmts -> stmt stmts;
        stmts -> ;

        stmt -> expr ';';
        stmt -> assn ';';
        stmt -> return_stmt ';';
        stmt -> if_stmt;
        stmt -> while_stmt;

        expr -> '(' expr ')';
        expr -> Ident;
        expr -> Literal;
        expr -> bin_expr;
        expr -> array_item;
        expr -> func_call;
        expr -> array_literal;
        expr -> struct_literal;

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

        array_literal -> '[' array_elems ']';

        array_elems -> expr ',' array_elems;
        array_elems -> ;

        struct_literal -> Ident '{' struct_items '}';

        struct_items -> Ident ':' expr ',' struct_items;
        struct_items -> ;

        assn -> assn_target '=' expr;

        assn_target -> Ident;
        assn_target -> array_item;

        return_stmt -> "return" expr;

        if_stmt     -> "if" cond '{' stmts '}';
        while_stmt  -> "while" cond '{' stmts '}';

        cond -> expr cmp_op expr;

        cmp_op -> '>';
        cmp_op -> "==";
        cmp_op -> '<';

        type_def -> struct_def;

        struct_def -> "struct" '{' struct_def_items '}';

        struct_def_items -> struct_def_item struct_def_items;
        struct_def_items -> ;

        struct_def_item -> Ident ':' ty ',';
    }
}