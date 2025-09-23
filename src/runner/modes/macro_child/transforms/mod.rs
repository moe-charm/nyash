/*!
 * Macro child transforms — split modules
 */

mod peek;
mod array;
mod map;
mod loops;
mod foreach;
mod scopebox;
mod lift;
mod if_to_loopform;
mod postfix;

// Re-exported via thin wrappers to keep names stable
pub(super) fn transform_peek_match_literal(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    peek::transform_peek_match_literal(ast)
}
pub(super) fn transform_array_prepend_zero(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    array::transform_array_prepend_zero(ast)
}
pub(super) fn transform_map_insert_tag(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    map::transform_map_insert_tag(ast)
}
pub(super) fn transform_loop_normalize(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    loops::transform_loop_normalize(ast)
}
pub(super) fn transform_for_foreach(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    foreach::transform_for_foreach(ast)
}
pub(super) fn transform_scopebox_inject(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    scopebox::transform_scopebox_inject(ast)
}
pub(super) fn transform_lift_nested_functions(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    lift::transform_lift_nested_functions(ast)
}
pub(super) fn transform_if_to_loopform(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    if_to_loopform::transform_if_to_loopform(ast)
}
pub(super) fn transform_postfix_handlers(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    postfix::transform_postfix_handlers(ast)
}

// Core normalization pass used by runners (always-on when macros enabled).
// Order matters: for/foreach → match(MatchExpr) → loop tail alignment.
pub fn normalize_core_pass(ast: &nyash_rust::ASTNode) -> nyash_rust::ASTNode {
    let a1 = transform_for_foreach(ast);
    let a2 = transform_peek_match_literal(&a1);
    let a3 = transform_loop_normalize(&a2);
    let a4 = if std::env::var("NYASH_SCOPEBOX_ENABLE").ok().map(|v| v=="1"||v=="true"||v=="on").unwrap_or(false) {
        transform_scopebox_inject(&a3)
    } else { a3 };
    let a4b = transform_lift_nested_functions(&a4);
    let a5 = if std::env::var("NYASH_IF_AS_LOOPFORM").ok().map(|v| v=="1"||v=="true"||v=="on").unwrap_or(false) {
        transform_if_to_loopform(&a4b)
    } else { a4b };
    let a6 = if std::env::var("NYASH_CATCH_NEW").ok().map(|v| v=="1"||v=="true"||v=="on").unwrap_or(false) {
        transform_postfix_handlers(&a5)
    } else { a5 };
    a6
}
