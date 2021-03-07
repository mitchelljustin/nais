use crate::parser::Parser;
use crate::parser::rule::Grammar;

#[allow(unused)]
pub fn parser() -> Parser {
    Parser::from(&grammar())
}

#[allow(unused)]
pub fn grammar() -> Grammar {
    production_rules! {
        START -> program_items EOF;

        program_items -> program_item program_items;
        program_items -> ;

        program_item -> func_def;
        program_item -> struct_def;
        program_item -> const_def;

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
        stmt -> if_stmt;
        stmt -> while_stmt;
        stmt -> return_stmt ';';

        expr -> Ident;
        expr -> Literal;
        expr -> '(' expr ')';
        expr -> bin_expr;
        expr -> deref;
        expr -> array_item;
        expr -> struct_item;
        expr -> func_call;
        expr -> array_literal;
        expr -> struct_literal;

        bin_expr -> expr bin_op expr;

        bin_op -> '+';
        bin_op -> '-';

        deref -> '*' expr;

        array_item -> Ident '[' expr ']';

        struct_item -> Ident '.' Ident;

        func_call -> Ident '(' arg_list ')';

        arg_list -> args;
        arg_list -> ;

        args -> expr ',' args;
        args -> expr;

        array_literal -> '[' array_literal_elems ']';

        array_literal_elems -> expr ',' array_literal_elems;
        array_literal_elems -> ;

        struct_literal -> Ident '{' struct_literal_items '}';

        struct_literal_items -> Ident ':' expr ',' struct_literal_items;
        struct_literal_items -> ;

        assn -> assn_target '=' expr;

        assn_target -> Ident;
        assn_target -> deref;
        assn_target -> array_item;
        assn_target -> struct_item;

        if_stmt     -> "if" cond '{' stmts '}';

        while_stmt  -> "while" cond '{' stmts '}';

        return_stmt -> "return" expr;

        cond -> expr cmp_op expr;

        cmp_op -> '>';
        cmp_op -> "==";
        cmp_op -> '<';

        struct_def -> "struct" '{' struct_def_items '}';

        struct_def_items -> struct_def_item struct_def_items;
        struct_def_items -> ;

        struct_def_item -> Ident ':' ty ',';

        const_def -> "const" Ident ':' ty '=' Literal ';';
    }
}