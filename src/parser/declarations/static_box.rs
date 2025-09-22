/*!
 * Static Box Definition Parser
 *
 * static box宣言と関連ヘルパー関数
 */

use crate::ast::{ASTNode, Span};
use crate::parser::common::ParserUtils;
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;
use std::collections::HashMap;

impl NyashParser {
    /// static box宣言をパース: static box Name { ... }
    pub fn parse_static_box(&mut self) -> Result<ASTNode, ParseError> {
        self.consume(TokenType::BOX)?;
        let (name, type_parameters, extends, implements) =
            crate::parser::declarations::static_def::header::parse_static_header(self)?;

        self.consume(TokenType::LBRACE)?;
        self.skip_newlines(); // ブレース後の改行をスキップ

        let mut fields = Vec::new();
        let mut methods = HashMap::new();
        let constructors = HashMap::new();
        let mut init_fields = Vec::new();
        let mut weak_fields = Vec::new(); // 🔗 Track weak fields for static box
        let mut static_init: Option<Vec<ASTNode>> = None;

        // Track last inserted method name to allow postfix catch/cleanup fallback parsing
        let mut last_method_name: Option<String> = None;
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines(); // ループ開始時に改行をスキップ

            // Fallback: method-level postfix catch/cleanup immediately following a method
            if crate::parser::declarations::box_def::members::postfix::try_parse_method_postfix_after_last_method(
                self, &mut methods, &last_method_name,
            )? { continue; }

            // RBRACEに到達していればループを抜ける
            if self.match_token(&TokenType::RBRACE) {
                break;
            }

            // 🔥 static 初期化子の処理（厳密ゲート互換）
            if let Some(body) = crate::parser::declarations::static_def::members::parse_static_initializer_if_any(self)? {
                static_init = Some(body);
                continue;
            } else if self.match_token(&TokenType::STATIC) {
                // STRICT で top-level seam を検出した場合は while を抜ける
                break;
            }

            // initブロックの処理（共通ヘルパに委譲）
            if crate::parser::declarations::box_def::members::fields::parse_init_block_if_any(
                self, &mut init_fields, &mut weak_fields,
            )? { continue; }

            if let TokenType::IDENTIFIER(field_or_method) = &self.current_token().token_type {
                let field_or_method = field_or_method.clone();
                self.advance();
                crate::parser::declarations::static_def::members::try_parse_method_or_field(
                    self, field_or_method, &mut methods, &mut fields, &mut last_method_name,
                )?;
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "method or field name".to_string(),
                    found: self.current_token().token_type.clone(),
                    line: self.current_token().line,
                });
            }
        }

        self.consume(TokenType::RBRACE)?;

        // 🔥 Static初期化ブロックから依存関係を抽出
        if let Some(ref init_stmts) = static_init {
            let dependencies = self.extract_dependencies_from_statements(init_stmts);
            self.static_box_dependencies
                .insert(name.clone(), dependencies);
        } else {
            self.static_box_dependencies
                .insert(name.clone(), std::collections::HashSet::new());
        }

        Ok(ASTNode::BoxDeclaration {
            name,
            fields,
            public_fields: vec![],
            private_fields: vec![],
            methods,
            constructors,
            init_fields,
            weak_fields, // 🔗 Add weak fields to static box construction
            is_interface: false,
            extends,
            implements,
            type_parameters,
            is_static: true, // 🔥 static boxフラグを設定
            static_init,     // 🔥 static初期化ブロック
            span: Span::unknown(),
        })
    }
}
