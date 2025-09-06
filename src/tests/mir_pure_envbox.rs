mod tests {
    use crate::ast::{ASTNode, LiteralValue, Span};
    use crate::mir::{MirCompiler, MirPrinter};

    #[test]
    fn pure_mode_new_emits_env_box_new() {
        // Enable pure mode
        std::env::set_var("NYASH_MIR_CORE13_PURE", "1");
        // new StringBox("Hello")
        let ast = ASTNode::New {
            class: "StringBox".to_string(),
            arguments: vec![ASTNode::Literal { value: LiteralValue::String("Hello".into()), span: Span::unknown() }],
            type_arguments: vec![],
            span: Span::unknown(),
        };
        let mut c = MirCompiler::new();
        let result = c.compile(ast).expect("compile");
        let dump = MirPrinter::new().print_module(&result.module);
        assert!(dump.contains("extern_call env.box.new"), "expected env.box.new in MIR. dump=\n{}", dump);
        std::env::remove_var("NYASH_MIR_CORE13_PURE");
    }
}

