#[cfg(test)]
mod tests {
    use crate::parser::NyashParser;

    fn is_allowed_core13(inst: &crate::mir::MirInstruction) -> bool {
        use crate::mir::MirInstruction as I;
        matches!(inst,
            I::Const { .. }
            | I::BinOp { .. }
            | I::Compare { .. }
            | I::Jump { .. }
            | I::Branch { .. }
            | I::Return { .. }
            | I::Phi { .. }
            | I::Call { .. }
            | I::BoxCall { .. }
            | I::ExternCall { .. }
            | I::TypeOp { .. }
            | I::Safepoint
            | I::Barrier { .. }
        )
    }

    #[test]
    fn final_mir_contains_only_core13_instructions() {
        std::env::set_var("NYASH_MIR_CORE13_PURE", "1");
        let code = r#"
local x
x = 1
if (x == 1) { x = x + 41 }
return new StringBox("ok").length()
"#;
        let ast = NyashParser::parse_from_string(code).expect("parse");
        let mut compiler = crate::mir::MirCompiler::new();
        let result = compiler.compile(ast).expect("compile");
        // Count non-Core13 instructions
        let mut bad = 0usize;
        for (_name, f) in &result.module.functions {
            for (_bb, b) in &f.blocks {
                for i in &b.instructions { if !is_allowed_core13(i) { bad += 1; } }
                if let Some(t) = &b.terminator { if !is_allowed_core13(t) { bad += 1; } }
            }
        }
        assert_eq!(bad, 0, "final MIR must contain only Core-13 instructions");
        std::env::remove_var("NYASH_MIR_CORE13_PURE");
    }
}

