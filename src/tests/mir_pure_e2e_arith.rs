#[cfg(test)]
mod tests {
    use crate::parser::NyashParser;
    use crate::backend::VM;

    #[test]
    fn vm_exec_addition_under_pure_mode() {
        std::env::set_var("NYASH_MIR_CORE13_PURE", "1");
        let code = "\nreturn 7 + 35\n";
        let ast = NyashParser::parse_from_string(code).expect("parse");
        let mut compiler = crate::mir::MirCompiler::new();
        let result = compiler.compile(ast).expect("compile");
        let mut vm = VM::new();
        let out = vm.execute_module(&result.module).expect("vm exec");
        assert_eq!(out.to_string_box().value, "42");
        std::env::remove_var("NYASH_MIR_CORE13_PURE");
    }
}

