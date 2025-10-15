use super::ast::ProgramV0;
use super::convert_to_ast;
use crate::mir::{MirModule, MirPrinter};

/// Unified JSON v0 lowering: always convert to AST and delegate to MirBuilder.
pub(super) fn lower_program(prog: ProgramV0) -> Result<MirModule, String> {
    if prog.body.is_empty() {
        return Err("empty body".into());
    }
    let ast = convert_to_ast::convert_program_to_ast(prog)?;
    let module = crate::mir::builder::entry::build_module_from_ast(ast)?;
    Ok(module)
}

pub(super) fn maybe_dump_mir(module: &MirModule) {
    if crate::config::env::cli_verbose() {
        let p = MirPrinter::new();
        println!("{}", p.print_module(module));
    }
}
