/*!
 * Basic MIR Test - Direct module testing
 */
use nyash_rust::mir::*;
use nyash_rust::ast::{ASTNode, LiteralValue, Span};

fn main() {
    println!("🚀 Testing MIR Basic Infrastructure");
    
    // Test 1: Create a simple literal AST and compile to MIR
    let ast = ASTNode::Literal {
        value: LiteralValue::Integer(42),
        span: Span::unknown(),
    };
    
    let mut compiler = MirCompiler::new();
    match compiler.compile(ast) {
        Ok(result) => {
            println!("✅ MIR compilation successful!");
            
            // Test verification
            match &result.verification_result {
                Ok(()) => println!("✅ MIR verification passed"),
                Err(errors) => {
                    println!("❌ MIR verification failed with {} errors:", errors.len());
                    for error in errors {
                        println!("  - {}", error);
                    }
                }
            }
            
            // Test MIR printing
            let mir_output = compiler.dump_mir(&result.module);
            println!("\n📊 Generated MIR:");
            println!("{}", mir_output);
            
            // Show statistics
            let stats = result.module.stats();
            println!("\n📊 Module Statistics:");
            println!("  Functions: {}", stats.function_count);
            println!("  Total Blocks: {}", stats.total_blocks);
            println!("  Total Instructions: {}", stats.total_instructions);
            println!("  Total Values: {}", stats.total_values);
            
        },
        Err(e) => {
            println!("❌ MIR compilation failed: {}", e);
        }
    }
    
    println!("\n🎯 MIR Test Complete!");
}