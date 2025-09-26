/*!
 * Box Definition Parser Module
 *
 * Box宣言（box, interface box, static box）の解析を担当
 * Nyashの中核概念「Everything is Box」を実現する重要モジュール
 */

use crate::ast::{ASTNode, Span};
use crate::parser::declarations::box_def::header as box_header;
use crate::parser::common::ParserUtils;
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;
use std::collections::HashMap;

impl NyashParser {
    /// Thin wrappers to keep the main loop tidy (behavior-preserving)
    fn box_try_block_first_property(
        &mut self,
        methods: &mut HashMap<String, ASTNode>,
        birth_once_props: &mut Vec<String>,
    ) -> Result<bool, ParseError> {
        crate::parser::declarations::box_def::members::properties::try_parse_block_first_property(
            self, methods, birth_once_props,
        )
    }

    fn box_try_method_postfix_after_last(
        &mut self,
        methods: &mut HashMap<String, ASTNode>,
        last_method_name: &Option<String>,
    ) -> Result<bool, ParseError> {
        crate::parser::declarations::box_def::members::postfix::try_parse_method_postfix_after_last_method(
            self, methods, last_method_name,
        )
    }

    fn box_try_init_block(
        &mut self,
        init_fields: &mut Vec<String>,
        weak_fields: &mut Vec<String>,
    ) -> Result<bool, ParseError> {
        crate::parser::declarations::box_def::members::fields::parse_init_block_if_any(
            self, init_fields, weak_fields,
        )
    }

    fn box_try_constructor(
        &mut self,
        is_override: bool,
        constructors: &mut HashMap<String, ASTNode>,
    ) -> Result<bool, ParseError> {
        if let Some((key, node)) = crate::parser::declarations::box_def::members::constructors::try_parse_constructor(self, is_override)? {
            constructors.insert(key, node);
            return Ok(true);
        }
        Ok(false)
    }

    fn box_try_visibility(
        &mut self,
        visibility: &str,
        methods: &mut HashMap<String, ASTNode>,
        fields: &mut Vec<String>,
        public_fields: &mut Vec<String>,
        private_fields: &mut Vec<String>,
        last_method_name: &mut Option<String>,
    ) -> Result<bool, ParseError> {
        crate::parser::declarations::box_def::members::fields::try_parse_visibility_block_or_single(
            self,
            visibility,
            methods,
            fields,
            public_fields,
            private_fields,
            last_method_name,
        )
    }

    /// Parse either a method or a header-first field/property starting with `name`.
    /// Updates `methods`/`fields` and `last_method_name` as appropriate.
    fn box_try_method_or_field(
        &mut self,
        name: String,
        is_override: bool,
        methods: &mut HashMap<String, ASTNode>,
        fields: &mut Vec<String>,
        birth_once_props: &Vec<String>,
        last_method_name: &mut Option<String>,
    ) -> Result<bool, ParseError> {
        if let Some(method) = crate::parser::declarations::box_def::members::methods::try_parse_method(
            self,
            name.clone(),
            is_override,
            birth_once_props,
        )? {
            *last_method_name = Some(name.clone());
            methods.insert(name, method);
            return Ok(true);
        }
        // Fallback: header-first field/property (computed/once/birth_once handled inside)
        crate::parser::declarations::box_def::members::fields::try_parse_header_first_field_or_property(
            self,
            name,
            methods,
            fields,
        )
    }
    // parse_unified_member_block_first moved to members::properties

    // parse_method_postfix_after_last_method moved to members::postfix

    /// box宣言をパース: box Name { fields... methods... }
    pub fn parse_box_declaration(&mut self) -> Result<ASTNode, ParseError> {
        self.consume(TokenType::BOX)?;
        let (name, type_parameters, extends, implements) =
            box_header::parse_header(self)?;

        self.consume(TokenType::LBRACE)?;

        let mut fields = Vec::new();
        let mut methods = HashMap::new();
        let mut public_fields: Vec<String> = Vec::new();
        let mut private_fields: Vec<String> = Vec::new();
        let mut constructors = HashMap::new();
        let mut init_fields = Vec::new();
        let mut weak_fields = Vec::new(); // 🔗 Track weak fields
        // Track birth_once properties to inject eager init into birth()
        let mut birth_once_props: Vec<String> = Vec::new();

        let mut last_method_name: Option<String> = None;
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            // 分類（段階移行用の観測）: 将来の分岐移譲のための前処理
            if crate::config::env::parser_stage3() {
                if let Ok(kind) = crate::parser::declarations::box_def::members::common::classify_member(self) {
                    let _ = kind; // 現段階では観測のみ（無副作用）
                }
            }

            // nyashモード（block-first）: { body } as (once|birth_once)? name : Type
            if self.box_try_block_first_property(&mut methods, &mut birth_once_props)? { continue; }

            // Fallback: method-level postfix catch/cleanup after a method (non-static box)
            if self.box_try_method_postfix_after_last(&mut methods, &last_method_name)? { continue; }

            // RBRACEに到達していればループを抜ける
            if self.match_token(&TokenType::RBRACE) {
                break;
            }

            // initブロックの処理（initメソッドではない場合のみ）
            if self.box_try_init_block(&mut init_fields, &mut weak_fields)? { continue; }

            // overrideキーワードをチェック
            let mut is_override = false;
            if self.match_token(&TokenType::OVERRIDE) {
                is_override = true;
                self.advance();
            }

            // constructor parsing moved to members::constructors
            if self.box_try_constructor(is_override, &mut constructors)? { continue; }

            // 🚨 birth()統一システム: Box名コンストラクタ無効化
            crate::parser::declarations::box_def::validators::forbid_box_named_constructor(self, &name)?;

            // 通常のフィールド名またはメソッド名、または unified members の先頭キーワードを読み取り
            if let TokenType::IDENTIFIER(field_or_method) = &self.current_token().token_type {
                let field_or_method = field_or_method.clone();
                self.advance();

                // 可視性: public/private ブロック/単行
                if self.box_try_visibility(
                    &field_or_method,
                    &mut methods,
                    &mut fields,
                    &mut public_fields,
                    &mut private_fields,
                    &mut last_method_name,
                )? { continue; }

                // Unified Members (header-first) gate: support once/birth_once via members::properties
                if crate::config::env::unified_members() && (field_or_method == "once" || field_or_method == "birth_once") {
                    if crate::parser::declarations::box_def::members::properties::try_parse_unified_property(
                        self,
                        &field_or_method,
                        &mut methods,
                        &mut birth_once_props,
                    )? {
                        last_method_name = None; // do not attach method-level postfix here
                        continue;
                    }
                }

                // メソッド or フィールド（委譲）
                if self.box_try_method_or_field(
                    field_or_method,
                    is_override,
                    &mut methods,
                    &mut fields,
                    &birth_once_props,
                    &mut last_method_name,
                )? { continue; }
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "method or field name".to_string(),
                    found: self.current_token().token_type.clone(),
                    line: self.current_token().line,
                });
            }
        }

        self.consume(TokenType::RBRACE)?;
        // 🚫 Disallow method named same as the box (constructor-like confusion)
        crate::parser::declarations::box_def::validators::validate_no_ctor_like_name(self, &name, &methods)?;

        // 🔥 Override validation
        for parent in &extends {
            self.validate_override_methods(&name, parent, &methods)?;
        }

        // birth_once 相互依存の簡易検出（宣言間の循環）
        crate::parser::declarations::box_def::validators::validate_birth_once_cycles(self, &methods)?;

        Ok(ASTNode::BoxDeclaration {
            name,
            fields,
            public_fields,
            private_fields,
            methods,
            constructors,
            init_fields,
            weak_fields, // 🔗 Add weak fields to AST
            is_interface: false,
            extends,
            implements,
            type_parameters,
            is_static: false,  // 通常のboxはnon-static
            static_init: None, // 通常のboxはstatic初期化ブロックなし
            span: Span::unknown(),
        })
    }

    /// interface box宣言をパース: interface box Name { methods... }
    pub fn parse_interface_box_declaration(&mut self) -> Result<ASTNode, ParseError> {
        crate::parser::declarations::box_def::interface::parse_interface_box(self)
    }
}

// ast_collect_me_fields moved into box_def::validators (private helper)
