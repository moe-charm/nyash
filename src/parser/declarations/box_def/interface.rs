//! Interface box parser: `interface box Name { methods... }`
use crate::ast::{ASTNode, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;
use std::collections::HashMap;

/// Parse `interface box Name { methods... }` and return an AST BoxDeclaration.
/// Caller must be positioned at the beginning of `interface box`.
pub(crate) fn parse_interface_box(p: &mut NyashParser) -> Result<ASTNode, ParseError> {
    p.consume(TokenType::INTERFACE)?;
    p.consume(TokenType::BOX)?;

    let name = if let TokenType::IDENTIFIER(name) = &p.current_token().token_type {
        let name = name.clone();
        p.advance();
        name
    } else {
        let line = p.current_token().line;
        return Err(ParseError::UnexpectedToken {
            found: p.current_token().token_type.clone(),
            expected: "identifier".to_string(),
            line,
        });
    };

    p.consume(TokenType::LBRACE)?;

    let mut methods = HashMap::new();

    while !p.match_token(&TokenType::RBRACE) && !p.is_at_end() {
        if let TokenType::IDENTIFIER(method_name) = &p.current_token().token_type {
            let method_name = method_name.clone();
            p.advance();

            // インターフェースメソッドはシグネチャのみ
            if p.match_token(&TokenType::LPAREN) {
                p.advance(); // consume '('

                let mut params = Vec::new();
                while !p.match_token(&TokenType::RPAREN) && !p.is_at_end() {
                    if let TokenType::IDENTIFIER(param) = &p.current_token().token_type {
                        params.push(param.clone());
                        p.advance();
                    }

                    if p.match_token(&TokenType::COMMA) {
                        p.advance();
                    }
                }

                p.consume(TokenType::RPAREN)?;

                // インターフェースメソッドは実装なし（空のbody）
                let method_decl = ASTNode::FunctionDeclaration {
                    name: method_name.clone(),
                    params,
                    body: vec![],       // 空の実装
                    is_static: false,   // インターフェースメソッドは通常静的でない
                    is_override: false, // デフォルトは非オーバーライド
                    span: Span::unknown(),
                };

                methods.insert(method_name, method_decl);
            } else {
                let line = p.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: p.current_token().token_type.clone(),
                    expected: "(".to_string(),
                    line,
                });
            }
        } else {
            let line = p.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: p.current_token().token_type.clone(),
                expected: "method name".to_string(),
                line,
            });
        }
    }

    p.consume(TokenType::RBRACE)?;

    Ok(ASTNode::BoxDeclaration {
        name,
        fields: vec![], // インターフェースはフィールドなし
        public_fields: vec![],
        private_fields: vec![],
        methods,
        constructors: HashMap::new(), // インターフェースにコンストラクタなし
        init_fields: vec![],          // インターフェースにinitブロックなし
        weak_fields: vec![],          // インターフェースにweak fieldsなし
        is_interface: true,           // インターフェースフラグ
        extends: vec![],              // Multi-delegation: None → vec![] として表現
        implements: vec![],
        type_parameters: Vec::new(), // インターフェースではジェネリクス未対応
        is_static: false,            // インターフェースは非static
        static_init: None,           // インターフェースにstatic initなし
        span: Span::unknown(),
    })
}
