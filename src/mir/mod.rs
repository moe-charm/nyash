/*!
 * Nyash MIR (Mid-level Intermediate Representation) - Stage 1 Implementation
 *
 * ChatGPT5-designed MIR infrastructure for native compilation support
 * Based on SSA form with effect tracking and Box-aware optimizations
 */

#[cfg(feature = "aot-plan-import")]
pub mod aot_plan_import;
pub mod basic_block;
pub mod builder;
pub mod effect;
pub mod function;
pub mod instruction;
pub mod instruction_kinds; // small kind-specific metadata (Const/BinOp)
pub mod instruction_introspection; // Introspection helpers for tests (instruction names)
pub mod types; // core MIR enums (ConstValue, Ops, MirType)
pub mod loop_api; // Minimal LoopBuilder facade (adapter-ready)
pub mod loop_builder; // SSA loop construction with phi nodes
pub mod optimizer;
pub mod utils; // Phase 15 control flow utilities for root treatment
pub mod optimizer_passes; // optimizer passes (normalize/diagnostics)
pub mod optimizer_stats; // extracted stats struct
pub mod passes;
pub mod printer;
mod printer_helpers; // internal helpers extracted from printer.rs
pub mod hints; // scaffold: zero-cost guidance (no-op)
pub mod slot_registry; // Phase 9.79b.1: method slot resolution (IDs)
pub mod value_id;
pub mod verification;
pub mod verification_types; // extracted error types // Optimization subpasses (e.g., type_hints)

// Re-export main types for easy access
pub use basic_block::{BasicBlock, BasicBlockId, BasicBlockIdGenerator};
pub use builder::MirBuilder;
pub use effect::{Effect, EffectMask};
pub use function::{FunctionSignature, MirFunction, MirModule};
pub use instruction::MirInstruction;
pub use types::{
    BarrierOp, BinaryOp, CompareOp, ConstValue, MirType, TypeOpKind, UnaryOp, WeakRefOp,
};
pub use optimizer::MirOptimizer;
pub use printer::MirPrinter;
pub use slot_registry::{BoxTypeId, MethodSlot};
pub use value_id::{LocalId, ValueId, ValueIdGenerator};
pub use verification::MirVerifier;
pub use verification_types::VerificationError;
// Phase 15 control flow utilities (段階的根治戦略)
pub use utils::{
    is_current_block_terminated,
    capture_actual_predecessor_and_jump,
    collect_phi_incoming_if_reachable,
    execute_statement_with_termination_check,
};

/// MIR compilation result
#[derive(Debug, Clone)]
pub struct MirCompileResult {
    pub module: MirModule,
    pub verification_result: Result<(), Vec<VerificationError>>,
}

/// MIR compiler - converts AST to MIR/SSA form
pub struct MirCompiler {
    builder: MirBuilder,
    verifier: MirVerifier,
    optimize: bool,
}

impl MirCompiler {
    /// Create a new MIR compiler
    pub fn new() -> Self {
        Self {
            builder: MirBuilder::new(),
            verifier: MirVerifier::new(),
            optimize: true,
        }
    }
    /// Create with options
    pub fn with_options(optimize: bool) -> Self {
        Self {
            builder: MirBuilder::new(),
            verifier: MirVerifier::new(),
            optimize,
        }
    }

    /// Compile AST to MIR module with verification
    pub fn compile(&mut self, ast: crate::ast::ASTNode) -> Result<MirCompileResult, String> {
        // Convert AST to MIR using builder
        let mut module = self.builder.build_module(ast)?;

        if self.optimize {
            let mut optimizer = MirOptimizer::new();
            let stats = optimizer.optimize_module(&mut module);
            if (crate::config::env::opt_diag_fail() || crate::config::env::opt_diag_forbid_legacy())
                && stats.diagnostics_reported > 0
            {
                return Err(format!(
                    "Diagnostic failure: {} issues detected (unlowered/legacy)",
                    stats.diagnostics_reported
                ));
            }
        }

        // Verify the generated MIR
        let verification_result = self.verifier.verify_module(&module);

        Ok(MirCompileResult {
            module,
            verification_result,
        })
    }

    /// Dump MIR to string for debugging
    pub fn dump_mir(&self, module: &MirModule) -> String {
        MirPrinter::new().print_module(module)
    }
}

impl Default for MirCompiler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ASTNode, LiteralValue};

    #[test]
    fn test_basic_mir_compilation() {
        let mut compiler = MirCompiler::new();

        // Create a simple literal AST node
        let ast = ASTNode::Literal {
            value: LiteralValue::Integer(42),
            span: crate::ast::Span::unknown(),
        };

        // Compile to MIR
        let result = compiler.compile(ast);
        assert!(result.is_ok(), "Basic MIR compilation should succeed");

        let compile_result = result.unwrap();
        assert!(
            !compile_result.module.functions.is_empty(),
            "Module should contain at least one function"
        );
    }

    #[test]
    fn test_mir_dump() {
        let mut compiler = MirCompiler::new();

        let ast = ASTNode::Literal {
            value: LiteralValue::Integer(42),
            span: crate::ast::Span::unknown(),
        };

        let result = compiler.compile(ast).unwrap();
        let mir_dump = compiler.dump_mir(&result.module);

        assert!(!mir_dump.is_empty(), "MIR dump should not be empty");
        assert!(
            mir_dump.contains("define"),
            "MIR dump should contain function definition"
        );
    }

    #[test]
    fn test_lowering_is_type_function_call_in_print() {
        // Build AST: print(isType(42, "Integer"))
        let ast = ASTNode::Print {
            expression: Box::new(ASTNode::FunctionCall {
                name: "isType".to_string(),
                arguments: vec![
                    ASTNode::Literal {
                        value: LiteralValue::Integer(42),
                        span: crate::ast::Span::unknown(),
                    },
                    ASTNode::Literal {
                        value: LiteralValue::String("Integer".to_string()),
                        span: crate::ast::Span::unknown(),
                    },
                ],
                span: crate::ast::Span::unknown(),
            }),
            span: crate::ast::Span::unknown(),
        };

        let mut compiler = MirCompiler::new();
        let result = compiler.compile(ast).expect("compile should succeed");

        // Ensure TypeOp exists in the resulting MIR
        let has_typeop = result.module.functions.values().any(|f| {
            f.blocks.values().any(|b| {
                b.all_instructions()
                    .any(|i| matches!(i, MirInstruction::TypeOp { .. }))
            })
        });
        assert!(
            has_typeop,
            "Expected TypeOp lowering for print(isType(...))"
        );
    }

    #[test]
    fn test_lowering_is_method_call_in_print() {
        // Build AST: print( (42).is("Integer") )
        let ast = ASTNode::Print {
            expression: Box::new(ASTNode::MethodCall {
                object: Box::new(ASTNode::Literal {
                    value: LiteralValue::Integer(42),
                    span: crate::ast::Span::unknown(),
                }),
                method: "is".to_string(),
                arguments: vec![ASTNode::Literal {
                    value: LiteralValue::String("Integer".to_string()),
                    span: crate::ast::Span::unknown(),
                }],
                span: crate::ast::Span::unknown(),
            }),
            span: crate::ast::Span::unknown(),
        };

        let mut compiler = MirCompiler::new();
        let result = compiler.compile(ast).expect("compile should succeed");

        // Ensure TypeOp exists in the resulting MIR
        let has_typeop = result.module.functions.values().any(|f| {
            f.blocks.values().any(|b| {
                b.all_instructions()
                    .any(|i| matches!(i, MirInstruction::TypeOp { .. }))
            })
        });
        assert!(
            has_typeop,
            "Expected TypeOp lowering for print(obj.is(...))"
        );
    }

    #[test]
    #[ignore = "MIR13 migration: extern console.log expectation pending"]
    fn test_lowering_extern_console_log() {
        // Build AST: console.log("hi") → ExternCall env.console.log
        let ast = ASTNode::MethodCall {
            object: Box::new(ASTNode::Variable {
                name: "console".to_string(),
                span: crate::ast::Span::unknown(),
            }),
            method: "log".to_string(),
            arguments: vec![ASTNode::Literal {
                value: LiteralValue::String("hi".to_string()),
                span: crate::ast::Span::unknown(),
            }],
            span: crate::ast::Span::unknown(),
        };

        let mut compiler = MirCompiler::new();
        let result = compiler.compile(ast).expect("compile should succeed");
        let dump = MirPrinter::verbose().print_module(&result.module);

        assert!(
            dump.contains("extern_call env.console.log"),
            "Expected extern_call env.console.log in MIR dump. Got:\n{}",
            dump
        );
    }

    #[test]
    fn test_lowering_boxcall_array_push() {
        // Build AST: (new ArrayBox()).push(1)
        let ast = ASTNode::MethodCall {
            object: Box::new(ASTNode::New {
                class: "ArrayBox".to_string(),
                arguments: vec![],
                type_arguments: vec![],
                span: crate::ast::Span::unknown(),
            }),
            method: "push".to_string(),
            arguments: vec![ASTNode::Literal {
                value: LiteralValue::Integer(1),
                span: crate::ast::Span::unknown(),
            }],
            span: crate::ast::Span::unknown(),
        };

        let mut compiler = MirCompiler::new();
        let result = compiler.compile(ast).expect("compile should succeed");
        let dump = MirPrinter::new().print_module(&result.module);
        // Expect a BoxCall to push (printer formats as `call <box>.<method>(...)`)
        assert!(
            dump.contains(".push("),
            "Expected BoxCall to .push(...). Got:\n{}",
            dump
        );
    }

    #[test]
    #[ignore = "MIR13 migration: method id naming in printer pending"]
    fn test_boxcall_method_id_on_universal_slot() {
        // Build AST: (new ArrayBox()).toString()
        let ast = ASTNode::MethodCall {
            object: Box::new(ASTNode::New {
                class: "ArrayBox".to_string(),
                arguments: vec![],
                type_arguments: vec![],
                span: crate::ast::Span::unknown(),
            }),
            method: "toString".to_string(),
            arguments: vec![],
            span: crate::ast::Span::unknown(),
        };

        let mut compiler = MirCompiler::new();
        let result = compiler.compile(ast).expect("compile should succeed");
        let dump = MirPrinter::new().print_module(&result.module);
        // Expect a BoxCall with numeric method id [#0] for toString universal slot
        assert!(
            dump.contains("toString[#0]"),
            "Expected method_id #0 for toString. Dump:\n{}",
            dump
        );
    }

    #[test]
    fn test_lowering_await_expression() {
        if crate::config::env::mir_core13_pure() {
            eprintln!("[TEST] skip await under Core-13 pure mode");
            return;
        }
        // Build AST: await 1  (semantic is nonsensical but should emit Await)
        let ast = ASTNode::AwaitExpression {
            expression: Box::new(ASTNode::Literal {
                value: LiteralValue::Integer(1),
                span: crate::ast::Span::unknown(),
            }),
            span: crate::ast::Span::unknown(),
        };
        let mut compiler = MirCompiler::new();
        let result = compiler.compile(ast).expect("compile should succeed");
        let dump = MirPrinter::new().print_module(&result.module);
        assert!(
            dump.contains("await"),
            "Expected await in MIR dump. Got:\n{}",
            dump
        );
    }

    #[test]
    fn test_await_has_checkpoints() {
        if crate::config::env::mir_core13_pure() {
            eprintln!("[TEST] skip await under Core-13 pure mode");
            return;
        }
        use crate::ast::{LiteralValue, Span};
        // Build: await 1
        let ast = ASTNode::AwaitExpression {
            expression: Box::new(ASTNode::Literal {
                value: LiteralValue::Integer(1),
                span: Span::unknown(),
            }),
            span: Span::unknown(),
        };
        let mut compiler = MirCompiler::new();
        let result = compiler.compile(ast).expect("compile");
        // Verifier should pass (await flanked by safepoints)
        assert!(
            result.verification_result.is_ok(),
            "Verifier failed for await checkpoints: {:?}",
            result.verification_result
        );
        let dump = compiler.dump_mir(&result.module);
        // Expect at least two safepoints in the function (before/after await)
        let sp_count = dump.matches("safepoint").count();
        assert!(
            sp_count >= 2,
            "Expected >=2 safepoints around await, got {}. Dump:\n{}",
            sp_count,
            dump
        );
    }

    #[test]
    fn test_rewritten_await_still_checkpoints() {
        if crate::config::env::mir_core13_pure() {
            eprintln!("[TEST] skip await under Core-13 pure mode");
            return;
        }
        use crate::ast::{LiteralValue, Span};
        // Enable rewrite so Await → ExternCall(env.future.await)
        std::env::set_var("NYASH_REWRITE_FUTURE", "1");
        let ast = ASTNode::AwaitExpression {
            expression: Box::new(ASTNode::Literal {
                value: LiteralValue::Integer(1),
                span: Span::unknown(),
            }),
            span: Span::unknown(),
        };
        let mut compiler = MirCompiler::new();
        let result = compiler.compile(ast).expect("compile");
        // Verifier should still pass (checkpoint verification includes ExternCall await)
        assert!(
            result.verification_result.is_ok(),
            "Verifier failed for rewritten await checkpoints: {:?}",
            result.verification_result
        );
        let dump = compiler.dump_mir(&result.module);
        assert!(
            dump.contains("env.future.await"),
            "Expected rewritten await extern call. Dump:\n{}",
            dump
        );
        let sp_count = dump.matches("safepoint").count();
        assert!(
            sp_count >= 2,
            "Expected >=2 safepoints around rewritten await, got {}. Dump:\n{}",
            sp_count,
            dump
        );
        // Cleanup env
        std::env::remove_var("NYASH_REWRITE_FUTURE");
    }

    #[test]
    #[ignore = "MIR13 migration: throw/safepoint expectations pending"]
    fn test_throw_compilation() {
        let mut compiler = MirCompiler::new();

        let throw_ast = ASTNode::Throw {
            expression: Box::new(ASTNode::Literal {
                value: LiteralValue::String("Test exception".to_string()),
                span: crate::ast::Span::unknown(),
            }),
            span: crate::ast::Span::unknown(),
        };

        let result = compiler.compile(throw_ast);
        assert!(result.is_ok(), "Throw compilation should succeed");

        let compile_result = result.unwrap();
        let mir_dump = compiler.dump_mir(&compile_result.module);
        assert!(
            mir_dump.contains("throw"),
            "MIR should contain throw instruction"
        );
        assert!(
            mir_dump.contains("safepoint"),
            "MIR should contain safepoint instruction"
        );
    }

    #[test]
    #[ignore = "MIR13 migration: loop safepoint expectation pending"]
    fn test_loop_compilation() {
        let mut compiler = MirCompiler::new();

        let loop_ast = ASTNode::Loop {
            condition: Box::new(ASTNode::Literal {
                value: LiteralValue::Bool(true),
                span: crate::ast::Span::unknown(),
            }),
            body: vec![ASTNode::Print {
                expression: Box::new(ASTNode::Literal {
                    value: LiteralValue::String("Loop body".to_string()),
                    span: crate::ast::Span::unknown(),
                }),
                span: crate::ast::Span::unknown(),
            }],
            span: crate::ast::Span::unknown(),
        };

        let result = compiler.compile(loop_ast);
        assert!(result.is_ok(), "Loop compilation should succeed");

        let compile_result = result.unwrap();
        let mir_dump = compiler.dump_mir(&compile_result.module);
        assert!(
            mir_dump.contains("br"),
            "MIR should contain branch instructions"
        );
        assert!(
            mir_dump.contains("safepoint"),
            "MIR should contain safepoint instructions"
        );
    }

    #[test]
    fn test_try_catch_compilation() {
        // Core-13 pure モードでは Try/Catch 命令は許容集合外のためスキップ
        if crate::config::env::mir_core13_pure() {
            eprintln!("[TEST] skip try/catch under Core-13 pure mode");
            return;
        }
        let mut compiler = MirCompiler::new();

        let try_catch_ast = ASTNode::TryCatch {
            try_body: vec![ASTNode::Print {
                expression: Box::new(ASTNode::Literal {
                    value: LiteralValue::String("Try block".to_string()),
                    span: crate::ast::Span::unknown(),
                }),
                span: crate::ast::Span::unknown(),
            }],
            catch_clauses: vec![crate::ast::CatchClause {
                exception_type: Some("Exception".to_string()),
                variable_name: Some("e".to_string()),
                body: vec![ASTNode::Print {
                    expression: Box::new(ASTNode::Literal {
                        value: LiteralValue::String("Catch block".to_string()),
                        span: crate::ast::Span::unknown(),
                    }),
                    span: crate::ast::Span::unknown(),
                }],
                span: crate::ast::Span::unknown(),
            }],
            finally_body: None,
            span: crate::ast::Span::unknown(),
        };

        let result = compiler.compile(try_catch_ast);
        assert!(result.is_ok(), "TryCatch compilation should succeed");

        let compile_result = result.unwrap();
        let mir_dump = compiler.dump_mir(&compile_result.module);
        assert!(
            mir_dump.contains("catch"),
            "MIR should contain catch instruction"
        );
    }
}
