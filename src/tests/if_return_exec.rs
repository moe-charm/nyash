use crate::backend::VM;
use crate::parser::NyashParser;
use crate::runtime::NyashRuntime;

#[test]
fn vm_if_then_return_else_fallthrough_false() {
    // If condition false: then is skipped, fallthrough returns 2
    let code = "\nif (0) { return 1 }\nreturn 2\n";
    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let _runtime = NyashRuntime::new();
    let mut compiler = crate::mir::MirCompiler::new();
    let compile_result = compiler.compile(ast).expect("mir compile failed");
    let mut vm = VM::new();
    let result = vm.execute_module(&compile_result.module).expect("vm exec failed");
    assert_eq!(result.to_string_box().value, "2");
}

#[test]
fn vm_if_then_return_true() {
    // If condition true: then branch returns 1
    let code = "\nif (1) { return 1 }\nreturn 2\n";
    let ast = NyashParser::parse_from_string(code).expect("parse failed");
    let _runtime = NyashRuntime::new();
    let mut compiler = crate::mir::MirCompiler::new();
    let compile_result = compiler.compile(ast).expect("mir compile failed");
    let mut vm = VM::new();
    let result = vm.execute_module(&compile_result.module).expect("vm exec failed");
    assert_eq!(result.to_string_box().value, "1");
}
