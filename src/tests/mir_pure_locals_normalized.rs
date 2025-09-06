#[cfg(test)]
mod tests {
    use crate::parser::NyashParser;
    use crate::mir::MirPrinter;

    #[test]
    fn locals_rewritten_to_env_local_calls_in_pure_mode() {
        // Enable Core-13 pure mode
        std::env::set_var("NYASH_MIR_CORE13_PURE", "1");

        // Use locals and arithmetic so Load/Store would appear without normalization
        let code = r#"
local x
x = 10
x = x + 32
return x
"#;

        let ast = NyashParser::parse_from_string(code).expect("parse");
        let mut compiler = crate::mir::MirCompiler::new();
        let result = compiler.compile(ast).expect("compile");

        let dump = MirPrinter::new().print_module(&result.module);
        // Expect env.local.get/set present (pure-mode normalization)
        assert!(dump.contains("extern_call env.local.get"), "expected env.local.get in MIR. dump=\n{}", dump);
        assert!(dump.contains("extern_call env.local.set"), "expected env.local.set in MIR. dump=\n{}", dump);

        std::env::remove_var("NYASH_MIR_CORE13_PURE");
    }
}

