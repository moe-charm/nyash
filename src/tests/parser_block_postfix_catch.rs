use crate::parser::NyashParser;

fn enable_stage3() {
    std::env::set_var("NYASH_PARSER_STAGE3", "1");
    // Accept block‑postfix under Stage‑3 gate
    std::env::set_var("NYASH_BLOCK_CATCH", "1");
}

#[test]
fn block_postfix_catch_basic() {
    enable_stage3();
    let src = r#"
    {
        print("x")
        throw "IO"
    } catch (e) {
        print(e)
    }
    "#;
    let ast = NyashParser::parse_from_string(src).expect("parse ok");
    fn has_try(ast: &crate::ast::ASTNode) -> bool {
        match ast {
            crate::ast::ASTNode::TryCatch { .. } => true,
            crate::ast::ASTNode::Program { statements, .. } => statements.iter().any(has_try),
            _ => false,
        }
    }
    assert!(has_try(&ast), "expected TryCatch from block‑postfix catch");
}

#[test]
fn block_postfix_cleanup_only() {
    enable_stage3();
    let src = r#"
    {
        print("ok")
    } cleanup {
        print("done")
    }
    "#;
    let ast = NyashParser::parse_from_string(src).expect("parse ok");
    // Ensure TryCatch with empty catches and Some(cleanup)
    fn check(ast: &crate::ast::ASTNode) -> bool {
        match ast {
            crate::ast::ASTNode::TryCatch { catch_clauses, finally_body, .. } => {
                catch_clauses.is_empty() && finally_body.is_some()
            }
            crate::ast::ASTNode::Program { statements, .. } => statements.iter().any(check),
            _ => false,
        }
    }
    assert!(check(&ast), "expected TryCatch with cleanup only");
}

#[test]
fn block_without_catch_with_direct_throw_should_error() {
    enable_stage3();
    let src = r#"
    { throw "Oops" }
    "#;
    assert!(NyashParser::parse_from_string(src).is_err());
}

#[test]
fn multiple_catch_after_block_should_error() {
    enable_stage3();
    let src = r#"
    { print("x") }
    catch (e) { print(e) }
    catch (e2) { print(e2) }
    "#;
    assert!(NyashParser::parse_from_string(src).is_err());
}

#[test]
fn cannot_attach_catch_to_if_block() {
    enable_stage3();
    let src = r#"
    if true { print("x") } 
    catch (e) { print(e) }
    "#;
    assert!(NyashParser::parse_from_string(src).is_err());
}
