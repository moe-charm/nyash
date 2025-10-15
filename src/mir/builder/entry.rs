use crate::ast::ASTNode;
use crate::mir::MirModule;

/// Public entry: build a MirModule from AST using MirBuilder lifecycle.
/// Thin wrapper to keep builder internals encapsulated.
pub fn build_module_from_ast(ast: ASTNode) -> Result<MirModule, String> {
    let mut b = super::MirBuilder::new();
    b.prepare_module()?;
    let v = b.lower_root(ast)?;
    b.finalize_module(v)
}
