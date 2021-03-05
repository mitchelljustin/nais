use crate::parser::state::State;
use crate::tokenizer::TokenType;

pub(crate) fn state_transition(state: State, tokens: &[(TokenType, &str)]) -> (State, usize, bool) {
    use State::*;
    use TokenType::*;
    match (state, tokens) {
        (START, _) =>
            (program, 0, false),

        (program, [(Keyword, "fn"), ..]) =>
            (func_defs, 0, true),
        (program, []) =>
            (ACCEPT, 0, false),

        (func_defs, [(Keyword, "fn"), ..]) =>
            (func_def, 0, true),
        (func_defs, []) =>
            (program, 0, false),

        (func_def, [(Keyword, "fn"), ..]) =>
            (param_list, 3, true),
        (func_def, [(RParen, ")"), ..]) =>
            (ret_ty, 1, false),
        (func_def, [(LBrac, "{"), ..]) =>
            (func_body, 1, false),
        (func_def, [(RBrac, "}"), ..]) =>
            (func_def, 1, false),
        (func_def, []) =>
            (program, 0, false),

        (ret_ty, [(RArrow, "->"), ..]) =>
            (ty, 1, true),
        (ret_ty, [(LBrac, "{"), ..]) =>
            (func_def, 0, true),

        (param_list, [(Ident, _), ..]) =>
            (params, 0, true),
        (param_list, [_, ..]) =>
            (func_def, 0, true),

        (params, [(Ident, _), ..]) =>
            (param, 0, true),
        (params, [..]) =>
            (func_def, 0, false),

        (param, [(Ident, _), ..]) =>
            (ty, 2, true),
        (param, [(Comma, ","), ..]) =>
            (params, 1, false),

        (func_body, [(RBrac, "}"), ..]) =>
            (func_def, 0, false),
        (func_body, [_, ..]) =>
            (local_defs, 0, false),

        (local_defs, [(Keyword, "let"), ..]) =>
            (local_def, 0, true),
        (local_defs, [..]) =>
            (func_body, 0, false),

        (local_def, [(Keyword, "let"), ..]) =>
            (ty, 3, true),
        (local_def, [(Semi, ";"), ..]) =>
            (local_defs, 1, false),

        (ty, [(Keyword, "i32"), ..]) =>
            (ty, 1, true),
        (ty, [(LSqBrac, "["), ..]) =>
            (literal, 3, true),
        // +
        (ty, [(Semi, ";"), ..]) =>
            (local_defs, 1, false),
        (ty, [(RSqBrac, "]"), ..]) =>
            (ty, 3, false),
        (ty, [(LBrac, "{"), ..]) =>
            (func_def, 0, false),
        // +
        (ty, [..]) =>
            (param, 1, false),

        (stmts, [(RBrac, "}"), ..]) =>
            (func_body, 0, false),
        (stmts, [_, ..]) =>
            (stmt, 0, true),

        (stmt, [(Keyword, "return"), ..]) =>
            (expr, 1, true),
        (stmt, [(Ident, _), (Eq, "="), ..]) =>
            (assn, 0, true),
        (stmt, [(Ident, _), (LSqBrac, "["), ..]) =>
            (assn, 0, true),
        (stmt, [(Semi, ";"), ..]) =>
            (stmt, 1, false),
        (stmt, [(RBrac, "}"), ..]) =>
            (stmts, 0, false),

        (assn, [(Ident, _), (LSqBrac, "["), ..]) =>
            (assn_target, 0, true),
        (assn, [(Ident, _), (Eq, "="), ..]) =>
            (assn_target, 0, true),
        (assn, [(Eq, "="), ..]) =>
            (expr, 1, false),

        (assn_target, [(Ident, _), (LSqBrac, "["), ..]) =>
            (expr, 2, true),
        (assn_target, [(Ident, _), ..]) =>
            (assn, 1, true),
        (assn_target, [(RSqBrac, "]"), ..]) =>
            (assn, 1, false),

        (expr, [(Literal, _), ..]) =>
            (literal, 0, true),
        (expr, [(Ident, _), ..]) =>
            (expr, 1, true),
        (expr, [(Semi, ";"), ..]) =>
            (stmt, 0, false),

        (literal, [(Literal, _), (RSqBrac, "]")]) =>
            (ty, 1, true),
        (literal, [(Literal, _), ..]) =>
            (expr, 1, true),

        // ...

        _ => (REJECT, 0, true),
    }
}
