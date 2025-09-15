#[cfg(test)]
mod tests {
    use crate::parser::NyashParser;
    use crate::backend::VM;

    fn run(code: &str) -> String {
        let ast = NyashParser::parse_from_string(code).expect("parse");
        let mut compiler = crate::mir::MirCompiler::new();
        let result = compiler.compile(ast).expect("compile");
        let mut vm = VM::new();
        let out = vm.execute_module(&result.module).expect("vm exec");
        out.to_string_box().value
    }

    #[test]
    fn phi_merge_then_only_assignment() {
        let code = r#"
        local x = 5
        if 1 < 2 { x = 7 } else { }
        return x
        "#;
        assert_eq!(run(code), "7");
    }

    #[test]
    fn phi_merge_else_only_assignment() {
        let code = r#"
        local y = 5
        if 2 < 1 { y = 7 } else { }
        return y
        "#;
        assert_eq!(run(code), "5");
    }

    #[test]
    fn shortcircuit_and_skips_rhs_side_effect() {
        let code = r#"
        local x = 0
        ((x = x + 1) < 0) && ((x = x + 1) < 0)
        return x
        "#;
        // LHS false ⇒ RHS not evaluated ⇒ x == 1
        assert_eq!(run(code), "1");
    }

    #[test]
    fn shortcircuit_or_skips_rhs_side_effect() {
        let code = r#"
        local x = 0
        ((x = x + 1) >= 0) || ((x = x + 1) < 0)
        return x
        "#;
        // LHS true ⇒ RHS not evaluated ⇒ x == 1
        assert_eq!(run(code), "1");
    }

    #[test]
    fn nested_loops_break_continue_mixed() {
        let code = r#"
        local i = 0
        local s = 0
        loop(i < 3) {
          local j = 0
          loop(j < 4) {
            j = j + 1
            if j == 1 { continue }
            if j == 3 { break }
            s = s + 1
          }
          i = i + 1
        }
        return s
        "#;
        // For each i: j=1 continue (skip s), j=2 => s++, then j=3 break ⇒ s increments once per outer iter ⇒ 3
        assert_eq!(run(code), "3");
    }
}

