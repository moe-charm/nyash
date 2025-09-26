//! Methods parsing (name(params){ body }) with special birth() prologue
use crate::ast::{ASTNode, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;

/// Try to parse a method declaration starting at `method_name` (already consumed identifier).
/// Returns Some(method_node) when parsed; None when not applicable (i.e., next token is not '(').
pub(crate) fn try_parse_method(
    p: &mut NyashParser,
    method_name: String,
    is_override: bool,
    birth_once_props: &Vec<String>,
) -> Result<Option<ASTNode>, ParseError> {
    if !p.match_token(&TokenType::LPAREN) {
        return Ok(None);
    }
    p.advance(); // consume '('

    let mut params = Vec::new();
    while !p.match_token(&TokenType::RPAREN) && !p.is_at_end() {
        crate::must_advance!(p, _unused, "method parameter parsing");
        if let TokenType::IDENTIFIER(param) = &p.current_token().token_type {
            params.push(param.clone());
            p.advance();
        }
        if p.match_token(&TokenType::COMMA) {
            p.advance();
        }
    }
    p.consume(TokenType::RPAREN)?;
    let mut body = p.parse_block_statements()?;

    // Inject eager init for birth_once at the very beginning of user birth()
    if method_name == "birth" && !birth_once_props.is_empty() {
        let mut injected: Vec<ASTNode> = Vec::new();
        for pprop in birth_once_props.iter() {
            let me_node = ASTNode::Me { span: Span::unknown() };
            let compute_call = ASTNode::MethodCall {
                object: Box::new(me_node.clone()),
                method: format!("__compute_birth_{}", pprop),
                arguments: vec![],
                span: Span::unknown(),
            };
            let tmp = format!("__ny_birth_{}", pprop);
            let local_tmp = ASTNode::Local {
                variables: vec![tmp.clone()],
                initial_values: vec![Some(Box::new(compute_call))],
                span: Span::unknown(),
            };
            let set_call = ASTNode::MethodCall {
                object: Box::new(me_node.clone()),
                method: "setField".to_string(),
                arguments: vec![
                    ASTNode::Literal {
                        value: crate::ast::LiteralValue::String(format!("__birth_{}", pprop)),
                        span: Span::unknown(),
                    },
                    ASTNode::Variable { name: tmp, span: Span::unknown() },
                ],
                span: Span::unknown(),
            };
            injected.push(local_tmp);
            injected.push(set_call);
        }
        let mut new_body = injected;
        new_body.extend(body.into_iter());
        body = new_body;
    }

    let method = ASTNode::FunctionDeclaration {
        name: method_name.clone(),
        params,
        body,
        is_static: false,
        is_override,
        span: Span::unknown(),
    };
    Ok(Some(method))
}
