use crate::parser::state::{Grammar, ParseTable};

pub(crate) fn parse_table() -> ParseTable {
    ParseTable::from(grammar())
}

pub(crate) fn grammar() -> Grammar {
    production_rules! {
        START -> program;

        program -> func_defs;

        func_defs -> func_def func_defs;
        func_defs -> EMPTY;

        func_def -> "fn" Ident '(' param_list ')' ret_ty '{' func_body '}';

        // param_list -> params;
        param_list -> EMPTY;

        // params -> param ',' params;
        // params -> param;

        // param -> Ident ':' ty;

        // ret_ty -> RArrow ty;
        ret_ty -> EMPTY;

        func_body -> local_defs stmts;

        local_defs -> local_def local_defs;
        local_defs -> EMPTY;

        local_def -> "let" Ident ':' ty ';';

        ty -> "i32";
        // ty -> '[' "i32" ';' Literal ']';

        stmts -> stmt stmts;
        stmts -> EMPTY;

        stmt -> assn ';';
        // stmt -> if_stmt;
        // stmt -> while_stmt;
        stmt -> return_stmt;
        stmt -> expr ';';

        assn -> assn_target '=' expr;

        assn_target -> Ident;
        // assn_target -> Ident '[' expr ']';

        // if_stmt     -> "if" cond '{' stmts '}';
        // while_stmt  -> "while" cond '{' stmts '}';
        return_stmt -> "return" expr ';';

        // cond -> expr cmp_op expr;

        // cmp_op -> '>';
        // cmp_op -> EqEq;
        // cmp_op -> '<';

        expr -> '(' expr ')';
        expr -> Literal;
        expr -> Ident;
        expr -> bin_expr;

        bin_expr -> expr bin_op expr;

        bin_op -> '+';
        bin_op -> '-';
    }
}