/*!
 * Phase 7 MIR Builder & VM Test - Async Operations (nowait/await)
 * 
 * Tests AST → MIR lowering and VM execution for Phase 7 async operations
 */

use nyash_rust::mir::{MirBuilder, MirPrinter};
use nyash_rust::backend::VM;
use nyash_rust::ast::{ASTNode, LiteralValue, Span};
use std::collections::HashMap;

#[test]
fn test_mir_phase7_basic_nowait_await() {
    // Build AST equivalent to:
    // static box Main { 
    //   main() { 
    //     nowait f1 = 42; 
    //     local result = await f1; 
    //     return result 
    //   }
    // }
    
    let mut main_methods = HashMap::new();
    
    // Create main method body
    let main_body = vec![
        // nowait f1 = 42
        ASTNode::Nowait {
            variable: "f1".to_string(),
            expression: Box::new(ASTNode::Literal {
                value: LiteralValue::Integer(42),
                span: Span::unknown(),
            }),
            span: Span::unknown(),
        },
        // local result = await f1
        ASTNode::Local {
            variables: vec!["result".to_string()],
            initial_values: vec![Some(Box::new(ASTNode::AwaitExpression {
                expression: Box::new(ASTNode::Variable {
                    name: "f1".to_string(),
                    span: Span::unknown(),
                }),
                span: Span::unknown(),
            }))],
            span: Span::unknown(),
        },
        // return result
        ASTNode::Return {
            value: Some(Box::new(ASTNode::Variable {
                name: "result".to_string(),
                span: Span::unknown(),
            })),
            span: Span::unknown(),
        },
    ];
    
    // Create main method
    let main_method = ASTNode::FunctionDeclaration {
        name: "main".to_string(),
        params: vec![],
        body: main_body,
        is_static: false,
        is_override: false,
        span: Span::unknown(),
    };
    
    main_methods.insert("main".to_string(), main_method);
    
    // Create static box Main
    let ast = ASTNode::BoxDeclaration {
        name: "Main".to_string(),
        fields: vec![],
        methods: main_methods,
        constructors: HashMap::new(),
        init_fields: vec![],
        weak_fields: vec![],
        is_interface: false,
        extends: vec![],
        implements: vec![],
        type_parameters: vec![],
        is_static: true,
        static_init: None,
        span: Span::unknown(),
    };
    
    // Build MIR
    let mut builder = MirBuilder::new();
    let result = builder.build_module(ast);
    
    if let Err(e) = &result {
        println!("MIR build error: {}", e);
    }
    assert!(result.is_ok(), "MIR build should succeed");
    
    let module = result.unwrap();
    
    // Print MIR for debugging
    let printer = MirPrinter::new();
    let mir_output = printer.print_module(&module);
    println!("Generated MIR:");
    println!("{}", mir_output);
    
    // Verify MIR contains expected instructions
    let function = module.get_function("main").unwrap();
    let instructions: Vec<_> = function.blocks.values()
        .flat_map(|block| &block.instructions)
        .collect();
    
    // Phase 5: FutureNew is deprecated - should contain NewBox "FutureBox" instead
    let has_future_box = instructions.iter().any(|inst| {
        matches!(inst, nyash_rust::mir::MirInstruction::NewBox { box_type, .. } if box_type == "FutureBox")
    });
    assert!(has_future_box, "MIR should contain NewBox FutureBox instruction");
    
    // Phase 5: Await is deprecated - should contain BoxCall "await" instead
    let has_await_call = instructions.iter().any(|inst| {
        matches!(inst, nyash_rust::mir::MirInstruction::BoxCall { method, .. } if method == "await")
    });
    assert!(has_await_call, "MIR should contain BoxCall await instruction");
    
    // Test VM execution
    let mut vm = VM::new();
    let execution_result = vm.execute_module(&module);
    
    if let Err(e) = &execution_result {
        println!("VM execution error: {}", e);
    }
    assert!(execution_result.is_ok(), "VM execution should succeed");
    
    let final_value = execution_result.unwrap();
    println!("VM execution result: {}", final_value.to_string_box().value);
    
    // Should return 42
    assert_eq!(final_value.to_string_box().value, "42");
}

#[test]
fn test_mir_phase7_multiple_nowait_await() {
    // Build AST equivalent to:
    // static box Main { 
    //   main() { 
    //     nowait f1 = 10; 
    //     nowait f2 = 20; 
    //     local result1 = await f1; 
    //     local result2 = await f2; 
    //     return result1 + result2 
    //   }
    // }
    
    let mut main_methods = HashMap::new();
    
    // Create main method body
    let main_body = vec![
        // nowait f1 = 10
        ASTNode::Nowait {
            variable: "f1".to_string(),
            expression: Box::new(ASTNode::Literal {
                value: LiteralValue::Integer(10),
                span: Span::unknown(),
            }),
            span: Span::unknown(),
        },
        // nowait f2 = 20
        ASTNode::Nowait {
            variable: "f2".to_string(),
            expression: Box::new(ASTNode::Literal {
                value: LiteralValue::Integer(20),
                span: Span::unknown(),
            }),
            span: Span::unknown(),
        },
        // local result1 = await f1
        ASTNode::Local {
            variables: vec!["result1".to_string()],
            initial_values: vec![Some(Box::new(ASTNode::AwaitExpression {
                expression: Box::new(ASTNode::Variable {
                    name: "f1".to_string(),
                    span: Span::unknown(),
                }),
                span: Span::unknown(),
            }))],
            span: Span::unknown(),
        },
        // local result2 = await f2
        ASTNode::Local {
            variables: vec!["result2".to_string()],
            initial_values: vec![Some(Box::new(ASTNode::AwaitExpression {
                expression: Box::new(ASTNode::Variable {
                    name: "f2".to_string(),
                    span: Span::unknown(),
                }),
                span: Span::unknown(),
            }))],
            span: Span::unknown(),
        },
        // return result1 + result2
        ASTNode::Return {
            value: Some(Box::new(ASTNode::BinaryOp {
                left: Box::new(ASTNode::Variable {
                    name: "result1".to_string(),
                    span: Span::unknown(),
                }),
                operator: nyash_rust::ast::BinaryOperator::Add,
                right: Box::new(ASTNode::Variable {
                    name: "result2".to_string(),
                    span: Span::unknown(),
                }),
                span: Span::unknown(),
            })),
            span: Span::unknown(),
        },
    ];
    
    // Create main method
    let main_method = ASTNode::FunctionDeclaration {
        name: "main".to_string(),
        params: vec![],
        body: main_body,
        is_static: false,
        is_override: false,
        span: Span::unknown(),
    };
    
    main_methods.insert("main".to_string(), main_method);
    
    // Create static box Main
    let ast = ASTNode::BoxDeclaration {
        name: "Main".to_string(),
        fields: vec![],
        methods: main_methods,
        constructors: HashMap::new(),
        init_fields: vec![],
        weak_fields: vec![],
        is_interface: false,
        extends: vec![],
        implements: vec![],
        type_parameters: vec![],
        is_static: true,
        static_init: None,
        span: Span::unknown(),
    };
    
    // Build MIR
    let mut builder = MirBuilder::new();
    let result = builder.build_module(ast);
    
    assert!(result.is_ok(), "MIR build should succeed");
    let module = result.unwrap();
    
    // Print MIR for debugging
    let printer = MirPrinter::new();
    let mir_output = printer.print_module(&module);
    println!("Generated MIR for multiple nowait/await:");
    println!("{}", mir_output);
    
    // Test VM execution
    let mut vm = VM::new();
    let execution_result = vm.execute_module(&module);
    
    assert!(execution_result.is_ok(), "VM execution should succeed");
    let final_value = execution_result.unwrap();
    println!("VM execution result: {}", final_value.to_string_box().value);
    
    // Should return 30 (10 + 20)
    assert_eq!(final_value.to_string_box().value, "30");
}

#[test]
fn test_mir_phase7_nested_await() {
    // Build AST equivalent to:
    // static box Main { 
    //   main() { 
    //     nowait outer = {
    //       nowait inner = 5;
    //       await inner * 2
    //     };
    //     return await outer 
    //   }
    // }
    
    let mut main_methods = HashMap::new();
    
    // Create inner computation: nowait inner = 5; await inner * 2
    let inner_computation = ASTNode::Program {
        statements: vec![
            ASTNode::Nowait {
                variable: "inner".to_string(),
                expression: Box::new(ASTNode::Literal {
                    value: LiteralValue::Integer(5),
                    span: Span::unknown(),
                }),
                span: Span::unknown(),
            },
            ASTNode::BinaryOp {
                left: Box::new(ASTNode::AwaitExpression {
                    expression: Box::new(ASTNode::Variable {
                        name: "inner".to_string(),
                        span: Span::unknown(),
                    }),
                    span: Span::unknown(),
                }),
                operator: nyash_rust::ast::BinaryOperator::Multiply,
                right: Box::new(ASTNode::Literal {
                    value: LiteralValue::Integer(2),
                    span: Span::unknown(),
                }),
                span: Span::unknown(),
            },
        ],
        span: Span::unknown(),
    };
    
    // Create main method body
    let main_body = vec![
        // nowait outer = { ... }
        ASTNode::Nowait {
            variable: "outer".to_string(),
            expression: Box::new(inner_computation),
            span: Span::unknown(),
        },
        // return await outer
        ASTNode::Return {
            value: Some(Box::new(ASTNode::AwaitExpression {
                expression: Box::new(ASTNode::Variable {
                    name: "outer".to_string(),
                    span: Span::unknown(),
                }),
                span: Span::unknown(),
            })),
            span: Span::unknown(),
        },
    ];
    
    // Create main method
    let main_method = ASTNode::FunctionDeclaration {
        name: "main".to_string(),
        params: vec![],
        body: main_body,
        is_static: false,
        is_override: false,
        span: Span::unknown(),
    };
    
    main_methods.insert("main".to_string(), main_method);
    
    // Create static box Main
    let ast = ASTNode::BoxDeclaration {
        name: "Main".to_string(),
        fields: vec![],
        methods: main_methods,
        constructors: HashMap::new(),
        init_fields: vec![],
        weak_fields: vec![],
        is_interface: false,
        extends: vec![],
        implements: vec![],
        type_parameters: vec![],
        is_static: true,
        static_init: None,
        span: Span::unknown(),
    };
    
    // Build MIR
    let mut builder = MirBuilder::new();
    let result = builder.build_module(ast);
    
    assert!(result.is_ok(), "MIR build should succeed");
    let module = result.unwrap();
    
    // Print MIR for debugging
    let printer = MirPrinter::new();
    let mir_output = printer.print_module(&module);
    println!("Generated MIR for nested await:");
    println!("{}", mir_output);
    
    // Test VM execution
    let mut vm = VM::new();
    let execution_result = vm.execute_module(&module);
    
    assert!(execution_result.is_ok(), "VM execution should succeed");
    let final_value = execution_result.unwrap();
    println!("VM execution result: {}", final_value.to_string_box().value);
    
    // Should return 10 (5 * 2)
    assert_eq!(final_value.to_string_box().value, "10");
}