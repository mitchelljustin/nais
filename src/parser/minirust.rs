use crate::parser::state::Grammar;

pub fn grammar() -> Grammar {
    production_rules! {
        START -> program;

        program -> func_defs;

        func_defs -> func_def func_defs;
        func_defs -> E;

        func_def -> "fn" Ident '(' param_list ')' '{' func_body '}';

        param_list -> params;
        param_list -> E;

        params -> param ',' params;
        params -> param;

        param -> Ident ':' ty;

        ty -> "i32";
        ty -> '[' "i32" ';' Literal ']';

        func_body -> local_defs stmts;

        local_defs -> local_def local_defs;
        local_defs -> E;

        local_def -> "let" Ident ':' ty ';';

        stmts -> stmt stmts;
        stmts -> E;

        stmt -> assn ';';
        stmt -> expr ';';
        stmt -> "return" expr ';';

        assn -> assn_target '=' expr;
        assn_target -> Ident;
        assn_target -> Ident '[' expr ']';

        expr -> '(' expr ')';
        expr -> Literal;
        expr -> Ident;
        expr -> bin_expr;

        bin_expr -> term '+' expr;
        bin_expr -> term '-' expr;
        term -> expr;
    }
}