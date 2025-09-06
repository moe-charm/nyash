#[cfg(all(test, feature = "llvm"))]
mod tests {
    use crate::parser::NyashParser;
    use std::fs;

    #[test]
    fn llvm_can_build_object_under_pure_mode() {
        // Enable Core-13 pure mode
        std::env::set_var("NYASH_MIR_CORE13_PURE", "1");

        // A simple program that exercises env.box.new and locals
        let code = r#"
local s
s = new StringBox("abc")
return s.length()
"#;

        let ast = NyashParser::parse_from_string(code).expect("parse");
        let mut compiler = crate::mir::MirCompiler::new();
        let result = compiler.compile(ast).expect("compile");

        // Build object via LLVM backend
        let out = "nyash_pure_llvm_build_test";
        crate::backend::llvm::compile_to_object(&result.module, &format!("{}.o", out)).expect("llvm object build");

        // Verify object exists and has content
        let meta = fs::metadata(format!("{}.o", out)).expect("obj exists");
        assert!(meta.len() > 0, "object file should be non-empty");

        // Cleanup
        let _ = fs::remove_file(format!("{}.o", out));
        std::env::remove_var("NYASH_MIR_CORE13_PURE");
    }
}

