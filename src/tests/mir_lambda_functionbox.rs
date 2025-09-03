use crate::parser::NyashParser;
use crate::mir::{MirCompiler};
use crate::backend::VM;

#[test]
fn lambda_value_then_call_returns_increment() {
    // Nyash code:
    // f = function(a) { return a + 1 }
    // f(41)
    let code = r#"
    f = function(a) {
        return a + 1
    }
    f(41)
    "#;
    let ast = NyashParser::parse_from_string(code).expect("parse");
    let mut mc = MirCompiler::new();
    let cr = mc.compile(ast).expect("mir");
    // Execute on VM
    let mut vm = VM::new();
    let out = vm.execute_module(&cr.module).expect("vm exec");
    if let crate::backend::vm::VMValue::Integer(i) = out { assert_eq!(i, 42); }
    else { panic!("Expected Integer 42, got {:?}", out); }
}

