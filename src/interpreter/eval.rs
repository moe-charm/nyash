//! Evaluation entry points: execute program and nodes

use crate::ast::ASTNode;
use crate::box_trait::{NyashBox, VoidBox};
use super::{NyashInterpreter, RuntimeError, ControlFlow};

impl NyashInterpreter {
    /// ASTを実行
    pub fn execute(&mut self, ast: ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        super::core::debug_log("=== NYASH EXECUTION START ===");
        let result = self.execute_node(&ast);
        if let Err(ref e) = result {
            eprintln!("❌ Interpreter error: {}", e);
        }
        super::core::debug_log("=== NYASH EXECUTION END ===");
        result
    }

    /// ノードを実行
    fn execute_node(&mut self, node: &ASTNode) -> Result<Box<dyn NyashBox>, RuntimeError> {
        match node {
            ASTNode::Program { statements, .. } => {
                let mut result: Box<dyn NyashBox> = Box::new(VoidBox::new());

                let last = statements.len().saturating_sub(1);
                for (i, statement) in statements.iter().enumerate() {
                    let prev = self.discard_context;
                    self.discard_context = i != last; // 最終文以外は値が破棄される
                    result = self.execute_statement(statement)?;
                    self.discard_context = prev;

                    // 制御フローチェック
                    match &self.control_flow {
                        ControlFlow::Break => {
                            return Err(RuntimeError::BreakOutsideLoop);
                        }
                        ControlFlow::Continue => {
                            return Err(RuntimeError::BreakOutsideLoop);
                        }
                        ControlFlow::Return(_) => {
                            return Err(RuntimeError::ReturnOutsideFunction);
                        }
                        ControlFlow::Throw(_) => {
                            return Err(RuntimeError::UncaughtException);
                        }
                        ControlFlow::None => {}
                    }
                }

                // 🎯 Static Box Main パターン - main()メソッドの自動実行
                let has_main_method = {
                    if let Ok(definitions) = self.shared.static_box_definitions.read() {
                        if let Some(main_definition) = definitions.get("Main") {
                            main_definition.methods.contains_key("main")
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                };

                if has_main_method {
                    // Main static boxを初期化
                    self.ensure_static_box_initialized("Main")?;

                    // Main.main(args?) を呼び出し（引数が1つなら空配列をデフォルト注入）
                    let mut default_args: Vec<ASTNode> = Vec::new();
                    if let Ok(defs) = self.shared.static_box_definitions.read() {
                        if let Some(main_def) = defs.get("Main") {
                            if let Some(m) = main_def.methods.get("main") {
                                if let ASTNode::FunctionDeclaration { params, .. } = m {
                                    if params.len() == 1 {
                                        default_args.push(ASTNode::New { class: "ArrayBox".to_string(), arguments: vec![], type_arguments: vec![], span: crate::ast::Span::unknown() });
                                    }
                                }
                            }
                        }
                    }
                    let main_call_ast = ASTNode::MethodCall {
                        object: Box::new(ASTNode::FieldAccess {
                            object: Box::new(ASTNode::Variable { name: "statics".to_string(), span: crate::ast::Span::unknown() }),
                            field: "Main".to_string(),
                            span: crate::ast::Span::unknown(),
                        }),
                        method: "main".to_string(),
                        arguments: default_args,
                        span: crate::ast::Span::unknown(),
                    };

                    // main()の戻り値を最終結果として使用
                    result = self.execute_statement(&main_call_ast)?;
                }

                Ok(result)
            }
            _ => self.execute_statement(node),
        }
    }
}
