use nyash_rust::mir::{MirCompiler};
use nyash_rust::ast::{ASTNode, LiteralValue, Span};

fn main() {
    println!("=== Testing MIR Control Flow Compilation ===\n");
    
    // Test 1: Basic Throw instruction
    println!("Test 1: Basic Throw Instruction");
    let throw_ast = ASTNode::Throw {
        expression: Box::new(ASTNode::Literal {
            value: LiteralValue::String("Test exception".to_string()),
            span: Span::unknown(),
        }),
        span: Span::unknown(),
    };
    
    let mut compiler = MirCompiler::new();
    match compiler.compile(throw_ast) {
        Ok(result) => {
            println!("✓ Throw compilation successful");
            let mir_dump = compiler.dump_mir(&result.module);
            println!("MIR Output:\n{}", mir_dump);
        },
        Err(e) => println!("✗ Throw compilation failed: {}", e),
    }
    
    println!("\n" + &"=".repeat(50) + "\n");
    
    // Test 2: Basic Loop instruction
    println!("Test 2: Basic Loop Instruction");
    let loop_ast = ASTNode::Loop {
        condition: Box::new(ASTNode::Literal {
            value: LiteralValue::Bool(true),
            span: Span::unknown(),
        }),
        body: vec![
            ASTNode::Print {
                expression: Box::new(ASTNode::Literal {
                    value: LiteralValue::String("Hello from loop".to_string()),
                    span: Span::unknown(),
                }),
                span: Span::unknown(),
            }
        ],
        span: Span::unknown(),
    };
    
    let mut compiler2 = MirCompiler::new();
    match compiler2.compile(loop_ast) {
        Ok(result) => {
            println!("✓ Loop compilation successful");
            let mir_dump = compiler2.dump_mir(&result.module);
            println!("MIR Output:\n{}", mir_dump);
        },
        Err(e) => println!("✗ Loop compilation failed: {}", e),
    }
    
    println!("\n" + &"=".repeat(50) + "\n");
    
    // Test 3: TryCatch compilation
    println!("Test 3: TryCatch Instruction");
    let try_catch_ast = ASTNode::TryCatch {
        try_body: vec![
            ASTNode::Print {
                expression: Box::new(ASTNode::Literal {
                    value: LiteralValue::String("In try block".to_string()),
                    span: Span::unknown(),
                }),
                span: Span::unknown(),
            }
        ],
        catch_clauses: vec![
            nyash_rust::ast::CatchClause {
                exception_type: Some("Exception".to_string()),
                variable_name: Some("e".to_string()),
                body: vec![
                    ASTNode::Print {
                        expression: Box::new(ASTNode::Literal {
                            value: LiteralValue::String("In catch block".to_string()),
                            span: Span::unknown(),
                        }),
                        span: Span::unknown(),
                    }
                ],
                span: Span::unknown(),
            }
        ],
        finally_body: None,
        span: Span::unknown(),
    };
    
    let mut compiler3 = MirCompiler::new();
    match compiler3.compile(try_catch_ast) {
        Ok(result) => {
            println!("✓ TryCatch compilation successful");
            let mir_dump = compiler3.dump_mir(&result.module);
            println!("MIR Output:\n{}", mir_dump);
        },
        Err(e) => println!("✗ TryCatch compilation failed: {}", e),
    }
    
    println!("\n=== All tests completed ===");
}