use crate::parser::table::{Grammar, ParseTable};

pub(crate) fn parse_table() -> ParseTable {
    ParseTable::from(grammar())
}

pub(crate) fn grammar() -> Grammar {
    production_rules! {
        START -> program EOF;

        program -> func_defs;

        func_defs -> func_def func_defs;
        func_defs -> ;

        func_def -> "fn" Ident '(' param_list ')' ret_ty '{' func_body '}';

        param_list -> params;
        // param_list ->;

        // params -> param ',' params;
        params -> param;

        param -> Ident ':' ty;

        ty -> "i32";
        // ty -> '[' "i32" ';' literal ']';

        ret_ty -> RArrow ty;
        // ret_ty ->;

        func_body -> local_defs stmts;

        // local_defs -> local_def local_defs;
        local_defs -> ;

        // local_def -> "let" Ident ':' ty ';';

        stmts -> stmt stmts;
        stmts ->;

        stmt -> return_stmt;
        // stmt -> assn ';';
        // stmt -> expr ';';
        // stmt -> if_stmt;
        // stmt -> while_stmt;

        return_stmt -> "return" expr ';';

        // assn -> assn_target '=' expr;

        // assn_target -> var_target;
        // assn_target -> array_target;

        // var_target -> Ident;

        // array_target -> Ident '[' expr ']';

        // if_stmt     -> "if" cond '{' stmts '}';
        // while_stmt  -> "while" cond '{' stmts '}';

        // cond -> expr cmp_op expr;

        // cmp_op -> '>';
        // cmp_op -> EqEq;
        // cmp_op -> '<';

        // expr -> '(' expr ')';
        expr -> var;
        expr -> literal;
        expr -> bin_expr;
        // expr -> array_read;

        bin_expr -> expr bin_op expr;

        bin_op -> '+';
        bin_op -> '-';

        // array_read -> var '[' expr ']';

        var -> Ident;
        literal -> Literal;
    }
}

fn add5(x: i32) -> i32 {
    return x + 5;
}