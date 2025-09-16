use nyash_rust::grammar::engine;

#[test]
fn grammar_sub_mul_div_rules_exist_and_basic_cases() {
    let eng = engine::get();

    // Sub
    assert!(!eng.sub_rules().is_empty(), "sub rules should not be empty");
    assert!(
        eng.decide_sub_result("Integer", "Integer").is_some(),
        "sub i64+i64 should be defined"
    );

    // Mul
    assert!(!eng.mul_rules().is_empty(), "mul rules should not be empty");
    assert!(
        eng.decide_mul_result("Integer", "Integer").is_some(),
        "mul i64*i64 should be defined"
    );

    // Div
    assert!(!eng.div_rules().is_empty(), "div rules should not be empty");
    assert!(
        eng.decide_div_result("Integer", "Integer").is_some(),
        "div i64/i64 should be defined"
    );
}
