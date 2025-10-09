//! Test harness construction: collection + AST transformation
//!
//! Collects test functions/methods and builds executable test harness.

use nyash_rust::ast::{ASTNode as A, BinaryOperator, LiteralValue, Span};
use std::collections::HashMap;

use super::json_args::{InstanceSpec, TestArgSpec, TestPlan};

/// Collect top-level test_* functions
fn collect_function_tests(
    ast: &nyash_rust::ASTNode,
    args_map: &Option<HashMap<String, TestArgSpec>>,
    tests: &mut Vec<TestPlan>,
) {
    if let nyash_rust::ASTNode::Program { statements, .. } = ast {
        for st in statements {
            if let nyash_rust::ASTNode::FunctionDeclaration { name, params, .. } = st {
                if name.starts_with("test_") {
                    let label = name.clone();
                    // select args: JSON map > defaults > skip
                    let mut maybe_args: Option<Vec<nyash_rust::ASTNode>> = None;
                    if let Some(m) = args_map {
                        if let Some(v) = m.get(&label) {
                            maybe_args = Some(v.args.clone());
                        }
                    }
                    let args = if let Some(a) = maybe_args {
                        a
                    } else if !params.is_empty()
                        && std::env::var("NYASH_TEST_ARGS_DEFAULTS").ok().as_deref() == Some("1")
                    {
                        let mut a: Vec<nyash_rust::ASTNode> = Vec::new();
                        for _ in params {
                            a.push(nyash_rust::ASTNode::Literal {
                                value: nyash_rust::ast::LiteralValue::Integer(0),
                                span: nyash_rust::ast::Span::unknown(),
                            });
                        }
                        a
                    } else if params.is_empty() {
                        Vec::new()
                    } else {
                        eprintln!("[macro][test][args] missing args for {} (need {}), skipping (set NYASH_TEST_ARGS_DEFAULTS=1 for zero defaults)", label, params.len());
                        continue;
                    };
                    tests.push(TestPlan {
                        label,
                        setup: None,
                        call: nyash_rust::ASTNode::FunctionCall {
                            name: name.clone(),
                            arguments: args,
                            span: nyash_rust::ast::Span::unknown(),
                        },
                    });
                }
            }
        }
    }
}

/// Collect Box.test_* methods (static and instance)
fn collect_box_tests(
    ast: &nyash_rust::ASTNode,
    args_map: &Option<HashMap<String, TestArgSpec>>,
    tests: &mut Vec<TestPlan>,
) {
    if let nyash_rust::ASTNode::Program { statements, .. } = ast {
        for st in statements {
            if let nyash_rust::ASTNode::BoxDeclaration {
                name: box_name,
                methods,
                ..
            } = st
            {
                for (mname, mnode) in methods {
                    if !mname.starts_with("test_") {
                        continue;
                    }
                    if let nyash_rust::ASTNode::FunctionDeclaration {
                        is_static, params, ..
                    } = mnode
                    {
                        if *is_static {
                            // Static: BoxName.test_*()
                            let mut args: Vec<nyash_rust::ASTNode> = Vec::new();
                            if let Some(m) = args_map {
                                if let Some(v) = m.get(&format!("{}.{}", box_name, mname)) {
                                    args = v.args.clone();
                                }
                            }
                            if args.is_empty() && !params.is_empty() {
                                if std::env::var("NYASH_TEST_ARGS_DEFAULTS").ok().as_deref()
                                    == Some("1")
                                {
                                    for _ in params {
                                        args.push(nyash_rust::ASTNode::Literal {
                                            value: nyash_rust::ast::LiteralValue::Integer(0),
                                            span: nyash_rust::ast::Span::unknown(),
                                        });
                                    }
                                } else {
                                    eprintln!("[macro][test][args] missing args for {}.{} (need {}), skipping", box_name, mname, params.len());
                                    continue;
                                }
                            }
                            let call = nyash_rust::ASTNode::MethodCall {
                                object: Box::new(nyash_rust::ASTNode::Variable {
                                    name: box_name.clone(),
                                    span: nyash_rust::ast::Span::unknown(),
                                }),
                                method: mname.clone(),
                                arguments: args,
                                span: nyash_rust::ast::Span::unknown(),
                            };
                            tests.push(TestPlan {
                                label: format!("{}.{}", box_name, mname),
                                setup: None,
                                call,
                            });
                        } else {
                            // Instance: try new BoxName() then .test_*()
                            let inst_var = format!("__t_{}", box_name.to_lowercase());
                            // Instance override via JSON
                            let mut inst_ctor: Option<InstanceSpec> = None;
                            if let Some(m) = args_map {
                                if let Some(v) = m.get(&format!("{}.{}", box_name, mname)) {
                                    inst_ctor = v.instance.clone();
                                }
                            }
                            let inst_init: nyash_rust::ASTNode = if let Some(spec) = inst_ctor {
                                match spec.ctor.as_str() {
                                    "new" => nyash_rust::ASTNode::New {
                                        class: box_name.clone(),
                                        arguments: spec.args,
                                        type_arguments: spec.type_args,
                                        span: nyash_rust::ast::Span::unknown(),
                                    },
                                    "birth" => nyash_rust::ASTNode::MethodCall {
                                        object: Box::new(nyash_rust::ASTNode::Variable {
                                            name: box_name.clone(),
                                            span: nyash_rust::ast::Span::unknown(),
                                        }),
                                        method: "birth".into(),
                                        arguments: spec.args,
                                        span: nyash_rust::ast::Span::unknown(),
                                    },
                                    other => {
                                        eprintln!("[macro][test][args] unknown ctor '{}' for {}.{}, using new()", other, box_name, mname);
                                        nyash_rust::ASTNode::New {
                                            class: box_name.clone(),
                                            arguments: vec![],
                                            type_arguments: vec![],
                                            span: nyash_rust::ast::Span::unknown(),
                                        }
                                    }
                                }
                            } else {
                                nyash_rust::ASTNode::New {
                                    class: box_name.clone(),
                                    arguments: vec![],
                                    type_arguments: vec![],
                                    span: nyash_rust::ast::Span::unknown(),
                                }
                            };
                            let setup = nyash_rust::ASTNode::Local {
                                variables: vec![inst_var.clone()],
                                initial_values: vec![Some(Box::new(inst_init))],
                                span: nyash_rust::ast::Span::unknown(),
                            };
                            let mut args: Vec<nyash_rust::ASTNode> = Vec::new();
                            if let Some(m) = args_map {
                                if let Some(v) = m.get(&format!("{}.{}", box_name, mname)) {
                                    args = v.args.clone();
                                }
                            }
                            if args.is_empty() && !params.is_empty() {
                                if std::env::var("NYASH_TEST_ARGS_DEFAULTS").ok().as_deref()
                                    == Some("1")
                                {
                                    for _ in params {
                                        args.push(nyash_rust::ASTNode::Literal {
                                            value: nyash_rust::ast::LiteralValue::Integer(0),
                                            span: nyash_rust::ast::Span::unknown(),
                                        });
                                    }
                                } else {
                                    eprintln!("[macro][test][args] missing args for {}.{} (need {}), skipping", box_name, mname, params.len());
                                    continue;
                                }
                            }
                            let call = nyash_rust::ASTNode::MethodCall {
                                object: Box::new(nyash_rust::ASTNode::Variable {
                                    name: inst_var.clone(),
                                    span: nyash_rust::ast::Span::unknown(),
                                }),
                                method: mname.clone(),
                                arguments: args,
                                span: nyash_rust::ast::Span::unknown(),
                            };
                            tests.push(TestPlan {
                                label: format!("{}.{}", box_name, mname),
                                setup: Some(setup),
                                call,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Apply NYASH_TEST_FILTER if set
fn apply_filter(tests: &mut Vec<TestPlan>) {
    if let Ok(substr) = std::env::var("NYASH_TEST_FILTER") {
        if !substr.is_empty() {
            tests.retain(|tp| tp.label.contains(&substr));
        }
    }
}

/// Build harness main() body from collected tests
fn build_harness_body(tests: &[TestPlan]) -> Vec<A> {
    let mut body: Vec<A> = Vec::new();
    // locals: pass=0, fail=0
    body.push(A::Local {
        variables: vec!["pass".into(), "fail".into()],
        initial_values: vec![
            Some(Box::new(A::Literal {
                value: LiteralValue::Integer(0),
                span: Span::unknown(),
            })),
            Some(Box::new(A::Literal {
                value: LiteralValue::Integer(0),
                span: Span::unknown(),
            })),
        ],
        span: Span::unknown(),
    });
    for tp in tests {
        // optional setup
        if let Some(set) = tp.setup.clone() {
            body.push(set);
        }
        // local r = CALL
        body.push(A::Local {
            variables: vec!["r".into()],
            initial_values: vec![Some(Box::new(tp.call.clone()))],
            span: Span::unknown(),
        });
        // if r { print("PASS t"); pass = pass + 1 } else { print("FAIL t"); fail = fail + 1 }
        let pass_msg = A::Literal {
            value: LiteralValue::String(format!("PASS {}", tp.label)),
            span: Span::unknown(),
        };
        let fail_msg = A::Literal {
            value: LiteralValue::String(format!("FAIL {}", tp.label)),
            span: Span::unknown(),
        };
        let then_body = vec![
            A::Print {
                expression: Box::new(pass_msg),
                span: Span::unknown(),
            },
            A::Assignment {
                target: Box::new(A::Variable {
                    name: "pass".into(),
                    span: Span::unknown(),
                }),
                value: Box::new(A::BinaryOp {
                    operator: BinaryOperator::Add,
                    left: Box::new(A::Variable {
                        name: "pass".into(),
                        span: Span::unknown(),
                    }),
                    right: Box::new(A::Literal {
                        value: LiteralValue::Integer(1),
                        span: Span::unknown(),
                    }),
                    span: Span::unknown(),
                }),
                span: Span::unknown(),
            },
        ];
        let else_body = vec![
            A::Print {
                expression: Box::new(fail_msg),
                span: Span::unknown(),
            },
            A::Assignment {
                target: Box::new(A::Variable {
                    name: "fail".into(),
                    span: Span::unknown(),
                }),
                value: Box::new(A::BinaryOp {
                    operator: BinaryOperator::Add,
                    left: Box::new(A::Variable {
                        name: "fail".into(),
                        span: Span::unknown(),
                    }),
                    right: Box::new(A::Literal {
                        value: LiteralValue::Integer(1),
                        span: Span::unknown(),
                    }),
                    span: Span::unknown(),
                }),
                span: Span::unknown(),
            },
        ];
        body.push(A::If {
            condition: Box::new(A::Variable {
                name: "r".into(),
                span: Span::unknown(),
            }),
            then_body,
            else_body: Some(else_body),
            span: Span::unknown(),
        });
    }
    // print summary and return fail
    body.push(A::Print {
        expression: Box::new(A::Literal {
            value: LiteralValue::String(format!("Summary: {} tests", tests.len())),
            span: Span::unknown(),
        }),
        span: Span::unknown(),
    });
    body.push(A::Return {
        value: Some(Box::new(A::Variable {
            name: "fail".into(),
            span: Span::unknown(),
        })),
        span: Span::unknown(),
    });
    body
}

/// Transform AST to inject test harness main()
pub fn build_harness_ast(ast: &nyash_rust::ASTNode, tests: Vec<TestPlan>) -> nyash_rust::ASTNode {
    let body = build_harness_body(&tests);
    let make_harness_main = |body: Vec<A>| -> A {
        A::FunctionDeclaration {
            name: "main".into(),
            params: vec!["args".into()],
            body,
            is_static: false,
            is_override: false,
            span: Span::unknown(),
        }
    };

    // Check if main exists and decide policy
    let mut has_main_fn = false;
    if let nyash_rust::ASTNode::Program { statements, .. } = ast {
        for st in statements {
            if let nyash_rust::ASTNode::FunctionDeclaration { name, .. } = st {
                if name == "main" {
                    has_main_fn = true;
                    break;
                }
            }
        }
    }

    let force = std::env::var("NYASH_TEST_FORCE").ok().as_deref() == Some("1");
    let entry_mode = std::env::var("NYASH_TEST_ENTRY").ok(); // Some("wrap"|"override")
    let ret_policy = std::env::var("NYASH_TEST_RETURN").ok(); // Some("tests"|"original")

    // Transform AST according to policy
    if let nyash_rust::ASTNode::Program { statements, span } = ast.clone() {
        let mut out_stmts: Vec<A> = Vec::with_capacity(statements.len() + 1);
        let mut orig_call_fn: Option<A> = None;
        for st in statements {
            match st {
                A::FunctionDeclaration {
                    name,
                    params,
                    body: orig_body,
                    is_static,
                    is_override,
                    span: fspan,
                } if name == "main" => {
                    if has_main_fn && (force || entry_mode.is_some()) {
                        // rename original main
                        let new_name = "__ny_orig_main".to_string();
                        out_stmts.push(A::FunctionDeclaration {
                            name: new_name.clone(),
                            params: params.clone(),
                            body: orig_body.clone(),
                            is_static,
                            is_override,
                            span: fspan,
                        });
                        if entry_mode.as_deref() == Some("wrap") {
                            let args_exprs = if !params.is_empty() {
                                vec![A::Variable {
                                    name: "args".into(),
                                    span: nyash_rust::ast::Span::unknown(),
                                }]
                            } else {
                                vec![]
                            };
                            orig_call_fn = Some(A::FunctionCall {
                                name: new_name,
                                arguments: args_exprs,
                                span: nyash_rust::ast::Span::unknown(),
                            });
                        }
                    } else {
                        // keep as-is (no injection)
                        out_stmts.push(A::FunctionDeclaration {
                            name,
                            params,
                            body: orig_body,
                            is_static,
                            is_override,
                            span: fspan,
                        });
                    }
                }
                other => out_stmts.push(other),
            }
        }
        if has_main_fn && !(force || entry_mode.is_some()) {
            if std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1") {
                eprintln!("[macro][test] existing main detected; skip harness (set --test-entry or NYASH_TEST_FORCE=1)");
            }
            return nyash_rust::ASTNode::Program {
                statements: out_stmts,
                span,
            };
        }
        // Compose harness main now
        let mut body2 = body;
        // Summary is already included in body. Append call/return per policy.
        if let Some(call) = orig_call_fn.take() {
            if ret_policy.as_deref() == Some("original") {
                // local __ny_orig_ret = __ny_orig_main(args)
                body2.push(A::Local {
                    variables: vec!["__ny_orig_ret".into()],
                    initial_values: vec![Some(Box::new(call))],
                    span: nyash_rust::ast::Span::unknown(),
                });
                // return __ny_orig_ret
                body2.push(A::Return {
                    value: Some(Box::new(A::Variable {
                        name: "__ny_orig_ret".into(),
                        span: nyash_rust::ast::Span::unknown(),
                    })),
                    span: nyash_rust::ast::Span::unknown(),
                });
            } else {
                // default: tests policy; still call original but ignore result
                body2.push(call);
                // return fail already appended earlier
            }
        }
        let harness_fn = make_harness_main(body2);
        out_stmts.push(harness_fn);
        return nyash_rust::ASTNode::Program {
            statements: out_stmts,
            span,
        };
    }
    ast.clone()
}

/// Public API: collect tests and build harness
pub fn inject_test_harness(
    ast: &nyash_rust::ASTNode,
    args_map: &Option<HashMap<String, TestArgSpec>>,
) -> nyash_rust::ASTNode {
    let mut tests: Vec<TestPlan> = Vec::new();
    collect_function_tests(ast, args_map, &mut tests);
    collect_box_tests(ast, args_map, &mut tests);
    apply_filter(&mut tests);

    if tests.is_empty() {
        if std::env::var("NYASH_MACRO_TRACE").ok().as_deref() == Some("1") {
            eprintln!("[macro][test] no tests found (functions starting with 'test_')");
        }
        return ast.clone();
    }

    build_harness_ast(ast, tests)
}
