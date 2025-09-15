#[cfg(test)]
mod tests {
    use crate::parser::NyashParser;
    use crate::backend::VM;

    #[test]
    fn vm_exec_simple_break() {
        let code = r#"
        loop(1) {
          break
        }
        return 1
        "#;
        let ast = NyashParser::parse_from_string(code).expect("parse");
        let mut compiler = crate::mir::MirCompiler::new();
        let result = compiler.compile(ast).expect("compile");
        let mut vm = VM::new();
        let out = vm.execute_module(&result.module).expect("vm exec");
        assert_eq!(out.to_string_box().value, "1");
    }

    #[test]
    fn vm_exec_continue_skips_body() {
        let code = r#"
        local i = 0
        local s = 0
        loop(i < 5) {
          i = i + 1
          if i == 3 { continue }
          s = s + 1
        }
        return s
        "#;
        let ast = NyashParser::parse_from_string(code).expect("parse");
        let mut compiler = crate::mir::MirCompiler::new();
        let result = compiler.compile(ast).expect("compile");
        let mut vm = VM::new();
        let out = vm.execute_module(&result.module).expect("vm exec");
        assert_eq!(out.to_string_box().value, "4");
    }

    #[test]
    fn vm_exec_break_inside_if() {
        let code = r#"
        local i = 0
        loop(i < 10) {
          if i == 3 { break }
          i = i + 1
        }
        return i
        "#;
        let ast = NyashParser::parse_from_string(code).expect("parse");
        let mut compiler = crate::mir::MirCompiler::new();
        let result = compiler.compile(ast).expect("compile");
        let mut vm = VM::new();
        let out = vm.execute_module(&result.module).expect("vm exec");
        assert_eq!(out.to_string_box().value, "3");
    }
}

