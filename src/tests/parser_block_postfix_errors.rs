use crate::parser::NyashParser;

fn enable_stage3() {
    std::env::set_var("NYASH_PARSER_STAGE3", "1");
    std::env::set_var("NYASH_BLOCK_CATCH", "1");
}

#[test]
fn top_level_catch_is_error() {
    enable_stage3();
    let src = r#"catch (e) { print(e) }"#;
    assert!(NyashParser::parse_from_string(src).is_err());
}

#[test]
fn top_level_cleanup_is_error() {
    enable_stage3();
    let src = r#"cleanup { print("x") }"#;
    assert!(NyashParser::parse_from_string(src).is_err());
}

#[test]
fn cleanup_then_catch_after_block_is_error() {
    enable_stage3();
    let src = r#"
    { print("x") } cleanup { print("a") }
    catch (e) { print(e) }
    "#;
    assert!(NyashParser::parse_from_string(src).is_err());
}

#[test]
fn cannot_attach_catch_to_loop_block() {
    enable_stage3();
    let src = r#"
    loop { print("x") }
    catch (e) { print(e) }
    "#;
    assert!(NyashParser::parse_from_string(src).is_err());
}
