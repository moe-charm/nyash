mod ast;
mod lexer;
mod lowering;

use ast::{ProgramV0, StmtV0};
use lowering::lower_program;

pub fn parse_json_v0_to_module(json: &str) -> Result<crate::mir::MirModule, String> {
    let prog: ProgramV0 =
        serde_json::from_str(json).map_err(|e| format!("invalid JSON v0: {}", e))?;
    if crate::config::env::cli_verbose() {
        let first = prog
            .body
            .get(1)
            .map(|s| match s { StmtV0::Try { .. } => "Try", _ => "Other" })
            .unwrap_or("<none>");
        eprintln!("[Bridge] JSON v0: body_len={} first_type={}", prog.body.len(), first);
    }
    if prog.version != 0 || prog.kind != "Program" {
        return Err("unsupported IR: expected {version:0, kind:\"Program\"}".into());
    }
    lower_program(prog)
}

pub fn parse_source_v0_to_module(input: &str) -> Result<crate::mir::MirModule, String> {
    let json = lexer::parse_source_v0_to_json(input)?;
    if std::env::var("NYASH_DUMP_JSON_IR").ok().as_deref() == Some("1") {
        println!("{}", json);
    }
    parse_json_v0_to_module(&json)
}

pub fn maybe_dump_mir(module: &crate::mir::MirModule) {
    lowering::maybe_dump_mir(module)
}
