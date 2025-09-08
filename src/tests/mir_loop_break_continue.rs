#[cfg(test)]
mod tests {
    use crate::parser::NyashParser;

    #[test]
    fn loop_with_continue_and_break_verifies() {
        let code = r#"
local i
i = 0
loop (i < 5) {
  i = i + 1
  if (i == 2) { continue }
  if (i == 4) { break }
}
return 0
"#;
        let ast = NyashParser::parse_from_string(code).expect("parse");
        let mut compiler = crate::mir::MirCompiler::new();
        let result = compiler.compile(ast).expect("compile");
        assert!(result.verification_result.is_ok(), "MIR should verify");
    }
}

