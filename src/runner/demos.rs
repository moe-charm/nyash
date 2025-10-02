//! Runner demo helpers (moved out of mod.rs to reduce file size)
use nyash_rust::ast::ASTNode;
use nyash_rust::box_trait::{AddBox, BoolBox, BoxCore, IntegerBox, NyashBox, StringBox, VoidBox};
// use nyash_rust::interpreter::NyashInterpreter; // Legacy interpreter removed
use nyash_rust::parser::NyashParser;
use nyash_rust::tokenizer::NyashTokenizer;

pub(super) fn demo_basic_boxes() {
    println!("\n📦 1. Basic Box Creation:");
    let string_box = StringBox::new("Hello, Nyash!".to_string());
    let int_box = IntegerBox::new(42);
    let bool_box = BoolBox::new(true);
    let void_box = VoidBox::new();
    println!("  StringBox: {}", string_box.to_string_box().value);
    println!("  IntegerBox: {}", int_box.to_string_box().value);
    println!("  BoolBox: {}", bool_box.to_string_box().value);
    println!("  VoidBox: {}", void_box.to_string_box().value);
    println!(
        "  Box IDs: String={}, Integer={}, Bool={}, Void={}",
        string_box.box_id(),
        int_box.box_id(),
        bool_box.box_id(),
        void_box.box_id()
    );
}

pub(super) fn demo_box_operations() {
    println!("\n🔄 2. Box Operations:");
    let left = IntegerBox::new(10);
    let right = IntegerBox::new(32);
    let add_box = AddBox::new(Box::new(left), Box::new(right));
    println!("  10 + 32 = {}", add_box.to_string_box().value);
    let str1 = StringBox::new("Hello, ".to_string());
    let str2 = StringBox::new("World!".to_string());
    let concat_box = AddBox::new(Box::new(str1), Box::new(str2));
    println!(
        "  \"Hello, \" + \"World!\" = {}",
        concat_box.to_string_box().value
    );
}

pub(super) fn demo_box_collections() {
    println!("\n📚 3. Box Collections:");
    println!("  Box collections functionality placeholder");
    println!("  (ArrayBox and other collection types will be demonstrated here)");
}

pub(super) fn demo_environment_system() {
    println!("\n🌍 4. Environment & Scope Management:");
    println!("  Environment demo placeholder - full testing done in interpreter");
}

pub(super) fn demo_tokenizer_system() {
    println!("\n🔤 5. Tokenizer System:");
    let test_code = "x = 42 + y";
    println!("  Input: {}", test_code);
    let mut tokenizer = NyashTokenizer::new(test_code);
    match tokenizer.tokenize() {
        Ok(tokens) => {
            println!("  Tokenized {} tokens successfully", tokens.len());
        }
        Err(e) => println!("  Tokenization error: {}", e),
    }
}

pub(super) fn demo_parser_system() {
    println!("\n🌳 6. Parser & AST System:");
    println!("  📝 Simple Box Declaration Test:");
    let simple_code = r#"
    box TestBox {
        value
        
        getValue() {
            return this.value
        }
    }
    "#;
    match NyashParser::parse_from_string(simple_code) {
        Ok(ast) => {
            println!("    Input: {}", simple_code.trim());
            println!("    AST: {}", ast);
            if let ASTNode::Program { statements, .. } = &ast {
                println!("    Program has {} statements", statements.len());
                for (i, stmt) in statements.iter().enumerate() {
                    println!("      [{}] {}", i, stmt.info());
                }
            }
        }
        Err(e) => println!("    Parser error: {}", e),
    }
}

pub(super) fn demo_interpreter_system() {
    println!("\n🎭 7. Interpreter System:");
    println!("  ⚠️  Legacy interpreter removed - use VM or LLVM backends instead");
    println!("  💡 Try: ./target/release/hakorune --backend vm program.nyash");
    println!("  💡 Try: ./target/release/hakorune --backend llvm program.nyash");
}

/// Run all demo sections (moved from runner/mod.rs)
pub(super) fn run_all_demos() {
    println!("🦀 Nyash Rust Implementation - Everything is Box! 🦀");
    println!("====================================================");
    demo_basic_boxes();
    demo_box_operations();
    demo_box_collections();
    demo_environment_system();
    demo_tokenizer_system();
    demo_parser_system();
    // demo_interpreter_system(); // Disabled - legacy interpreter removed
    println!("\n🎉 All Box operations completed successfully!");
    println!("Memory safety guaranteed by Rust's borrow checker! 🛡️");
}
