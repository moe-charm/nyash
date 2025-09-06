#[cfg(all(test, feature = "llvm"))]
mod tests {
    use crate::parser::NyashParser;
    use crate::backend::VM;

    #[test]
    fn llvm_exec_matches_vm_for_addition_under_pure_mode() {
        std::env::set_var("NYASH_MIR_CORE13_PURE", "1");
        let code = "\nreturn 7 + 35\n";
        let ast = NyashParser::parse_from_string(code).expect("parse");
        let mut compiler = crate::mir::MirCompiler::new();
        let result = compiler.compile(ast).expect("compile");

        // VM result
        let mut vm = VM::new();
        let vm_out = vm.execute_module(&result.module).expect("vm exec");
        let vm_s = vm_out.to_string_box().value;

        // LLVM result (compile+execute parity path)
        let llvm_out = crate::backend::llvm::compile_and_execute(&result.module, "pure_llvm_parity").expect("llvm exec");
        let llvm_s = llvm_out.to_string_box().value;

        assert_eq!(vm_s, llvm_s, "VM and LLVM outputs should match");
        std::env::remove_var("NYASH_MIR_CORE13_PURE");
    }
}

