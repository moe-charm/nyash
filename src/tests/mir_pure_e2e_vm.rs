#[cfg(test)]
mod tests {
    use crate::parser::NyashParser;
    use crate::backend::VM;

    #[test]
    fn vm_exec_new_string_length_under_pure_mode() {
        // Enable Core-13 pure mode
        std::env::set_var("NYASH_MIR_CORE13_PURE", "1");

        // Nyash code: return (new StringBox("Hello")).length()
        let code = r#"
return (new StringBox("Hello")).length()
"#;

        // Parse -> MIR -> VM execute
        let ast = NyashParser::parse_from_string(code).expect("parse");
        let mut compiler = crate::mir::MirCompiler::new();
        let result = compiler.compile(ast).expect("compile");

        let mut vm = VM::new();
        let out = vm.execute_module(&result.module).expect("vm exec");
        // Expect 5 as string (to_string_box) for convenience
        assert_eq!(out.to_string_box().value, "5");

        // Cleanup
        std::env::remove_var("NYASH_MIR_CORE13_PURE");
    }
}

