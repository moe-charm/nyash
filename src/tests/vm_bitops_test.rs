use crate::parser::NyashParser;
use crate::interpreter::NyashInterpreter;

#[test]
fn vm_exec_bitwise_and_shift() {
    let code = r#"
        return (5 & 3) + (5 | 2) + (5 ^ 1) + (1 << 5) + (32 >> 3)
    "#;
    let ast = NyashParser::parse_from_string(code).expect("parse ok");
    let mut interp = NyashInterpreter::new();
    let out = interp.execute(ast).expect("exec ok");
    assert_eq!(out.to_string_box().value, "48");
}

#[test]
fn vm_exec_shift_masking() {
    // 1 << 100 should mask to 1 << (100 & 63) = 1 << 36
    let code = r#" return 1 << 100 "#;
    let ast = NyashParser::parse_from_string(code).expect("parse ok");
    let mut interp = NyashInterpreter::new();
    let out = interp.execute(ast).expect("exec ok");
    // compute expected as i64
    let expected = (1_i64 as i64).wrapping_shl((100_u32) & 63);
    assert_eq!(out.to_string_box().value, expected.to_string());
}
