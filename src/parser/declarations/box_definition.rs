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
    /// box宣言をパース: box Name { fields... methods... }
    pub fn parse_box_declaration(&mut self) -> Result<ASTNode, ParseError> {
        self.consume(TokenType::BOX)?;
        let (name, type_parameters, extends, implements) =
            box_header::parse_header(self)?;

        self.consume(TokenType::LBRACE)?;
        self.skip_newlines(); // ブレース後の改行をスキップ

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
            self.skip_newlines(); // ループ開始時に改行をスキップ
            // 分類（段階移行用の観測）: 将来の分岐移譲のための前処理
            if crate::config::env::parser_stage3() {
                if let Ok(kind) = crate::parser::declarations::box_def::members::common::classify_member(self) {
                    let _ = kind; // 現段階では観測のみ（無副作用）
                }
            }

            // nyashモード（block-first）: { body } as (once|birth_once)? name : Type
            if crate::config::env::unified_members() && self.match_token(&TokenType::LBRACE) {
                // Parse block body first
                let mut final_body = self.parse_block_statements()?;
                self.skip_newlines();
                // Expect 'as'
                if let TokenType::IDENTIFIER(kw) = &self.current_token().token_type {
                    if kw != "as" {
                        // Not a block-first member; treat as standalone block statement in box (unsupported)
                        let line = self.current_token().line;
                        return Err(ParseError::UnexpectedToken { found: self.current_token().token_type.clone(), expected: "'as' after block for block-first member".to_string(), line });
                    }
                } else {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken { found: self.current_token().token_type.clone(), expected: "'as' after block for block-first member".to_string(), line });
                }
                self.advance(); // consume 'as'
                // Optional kind keyword
                let mut kind = "computed".to_string();
                if let TokenType::IDENTIFIER(k) = &self.current_token().token_type {
                    if k == "once" || k == "birth_once" {
                        kind = k.clone();
                        self.advance();
                    }
                }
                // Name : Type
                let name = if let TokenType::IDENTIFIER(n) = &self.current_token().token_type {
                    let s = n.clone();
                    self.advance();
                    s
                } else {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken { found: self.current_token().token_type.clone(), expected: "identifier for member name".to_string(), line });
                };
                if self.match_token(&TokenType::COLON) {
                    self.advance();
                    if let TokenType::IDENTIFIER(_ty) = &self.current_token().token_type { self.advance(); } else {
                        let line = self.current_token().line;
                        return Err(ParseError::UnexpectedToken { found: self.current_token().token_type.clone(), expected: "type name after ':'".to_string(), line });
                    }
                } else {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken { found: self.current_token().token_type.clone(), expected: ": type".to_string(), line });
                }
                // Optional postfix handlers (Stage-3) directly after block
                if crate::config::env::parser_stage3() && (self.match_token(&TokenType::CATCH) || self.match_token(&TokenType::CLEANUP)) {
                    let mut catch_clauses: Vec<crate::ast::CatchClause> = Vec::new();
                    if self.match_token(&TokenType::CATCH) {
                        self.advance();
                        self.consume(TokenType::LPAREN)?;
                        let (exc_ty, exc_var) = self.parse_catch_param()?;
                        self.consume(TokenType::RPAREN)?;
                        let catch_body = self.parse_block_statements()?;
                        catch_clauses.push(crate::ast::CatchClause { exception_type: exc_ty, variable_name: exc_var, body: catch_body, span: crate::ast::Span::unknown() });
                        self.skip_newlines();
                        if self.match_token(&TokenType::CATCH) {
                            let line = self.current_token().line;
                            return Err(ParseError::UnexpectedToken { found: self.current_token().token_type.clone(), expected: "single catch only after member block".to_string(), line });
                        }
                    }
                    let finally_body = if self.match_token(&TokenType::CLEANUP) { self.advance(); Some(self.parse_block_statements()?) } else { None };
                    final_body = vec![ASTNode::TryCatch { try_body: final_body, catch_clauses, finally_body, span: crate::ast::Span::unknown() }];
                }

                // Generate methods per kind
                if kind == "once" {
                    let compute_name = format!("__compute_once_{}", name);
                    let compute = ASTNode::FunctionDeclaration { name: compute_name.clone(), params: vec![], body: final_body, is_static: false, is_override: false, span: Span::unknown() };
                    methods.insert(compute_name.clone(), compute);
                    let key = format!("__once_{}", name);
                    let poison_key = format!("__once_poison_{}", name);
                    let cached_local = format!("__ny_cached_{}", name);
                    let poison_local = format!("__ny_poison_{}", name);
                    let val_local = format!("__ny_val_{}", name);
                    let me_node = ASTNode::Me { span: Span::unknown() };
                    let get_cached = ASTNode::MethodCall { object: Box::new(me_node.clone()), method: "getField".to_string(), arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::String(key.clone()), span: Span::unknown() }], span: Span::unknown() };
                    let local_cached = ASTNode::Local { variables: vec![cached_local.clone()], initial_values: vec![Some(Box::new(get_cached))], span: Span::unknown() };
                    let cond_cached = ASTNode::BinaryOp { operator: crate::ast::BinaryOperator::NotEqual, left: Box::new(ASTNode::Variable { name: cached_local.clone(), span: Span::unknown() }), right: Box::new(ASTNode::Literal { value: crate::ast::LiteralValue::Null, span: Span::unknown() }), span: Span::unknown() };
                    let then_ret_cached = vec![ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: cached_local.clone(), span: Span::unknown() })), span: Span::unknown() }];
                    let if_cached = ASTNode::If { condition: Box::new(cond_cached), then_body: then_ret_cached, else_body: None, span: Span::unknown() };
                    let get_poison = ASTNode::MethodCall { object: Box::new(me_node.clone()), method: "getField".to_string(), arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::String(poison_key.clone()), span: Span::unknown() }], span: Span::unknown() };
                    let local_poison = ASTNode::Local { variables: vec![poison_local.clone()], initial_values: vec![Some(Box::new(get_poison))], span: Span::unknown() };
                    let cond_poison = ASTNode::BinaryOp { operator: crate::ast::BinaryOperator::NotEqual, left: Box::new(ASTNode::Variable { name: poison_local.clone(), span: Span::unknown() }), right: Box::new(ASTNode::Literal { value: crate::ast::LiteralValue::Null, span: Span::unknown() }), span: Span::unknown() };
                    let then_throw = vec![ASTNode::Throw { expression: Box::new(ASTNode::Literal { value: crate::ast::LiteralValue::String(format!("once '{}' previously failed", name)), span: Span::unknown() }), span: Span::unknown() }];
                    let if_poison = ASTNode::If { condition: Box::new(cond_poison), then_body: then_throw, else_body: None, span: Span::unknown() };
                    let call_compute = ASTNode::MethodCall { object: Box::new(me_node.clone()), method: compute_name.clone(), arguments: vec![], span: Span::unknown() };
                    let local_val = ASTNode::Local { variables: vec![val_local.clone()], initial_values: vec![Some(Box::new(call_compute))], span: Span::unknown() };
                    let set_call = ASTNode::MethodCall { object: Box::new(me_node.clone()), method: "setField".to_string(), arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::String(key.clone()), span: Span::unknown() }, ASTNode::Variable { name: val_local.clone(), span: Span::unknown() }], span: Span::unknown() };
                    let ret_stmt = ASTNode::Return { value: Some(Box::new(ASTNode::Variable { name: val_local.clone(), span: Span::unknown() })), span: Span::unknown() };
                    let try_body = vec![local_val, set_call, ret_stmt];
                    let catch_body = vec![
                        ASTNode::MethodCall { object: Box::new(me_node.clone()), method: "setField".to_string(), arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::String(poison_key.clone()), span: Span::unknown() }, ASTNode::Literal { value: crate::ast::LiteralValue::Bool(true), span: Span::unknown() }], span: Span::unknown() },
                        ASTNode::Throw { expression: Box::new(ASTNode::Literal { value: crate::ast::LiteralValue::String(format!("once '{}' init failed", name)), span: Span::unknown() }), span: Span::unknown() }
                    ];
                    let trycatch = ASTNode::TryCatch { try_body, catch_clauses: vec![crate::ast::CatchClause { exception_type: None, variable_name: None, body: catch_body, span: Span::unknown() }], finally_body: None, span: Span::unknown() };
                    let getter_body = vec![local_cached, if_cached, local_poison, if_poison, trycatch];
                    let getter_name = format!("__get_once_{}", name);
                    let getter = ASTNode::FunctionDeclaration { name: getter_name.clone(), params: vec![], body: getter_body, is_static: false, is_override: false, span: Span::unknown() };
                    methods.insert(getter_name, getter);
                } else if kind == "birth_once" {
                    // Self-cycle guard: birth_once cannot reference itself via me.<name>
                    if self.ast_contains_me_field(&final_body, &name) {
                        let line = self.current_token().line;
                        return Err(ParseError::UnexpectedToken { found: self.current_token().token_type.clone(), expected: format!("birth_once '{}' must not reference itself", name), line });
                    }
                    birth_once_props.push(name.clone());
                    let compute_name = format!("__compute_birth_{}", name);
                    let compute = ASTNode::FunctionDeclaration { name: compute_name.clone(), params: vec![], body: final_body, is_static: false, is_override: false, span: Span::unknown() };
                    methods.insert(compute_name.clone(), compute);
                    let key = format!("__birth_{}", name);
                    let me_node = ASTNode::Me { span: Span::unknown() };
                    let get_call = ASTNode::MethodCall { object: Box::new(me_node.clone()), method: "getField".to_string(), arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::String(key.clone()), span: Span::unknown() }], span: Span::unknown() };
                    let getter_body = vec![ASTNode::Return { value: Some(Box::new(get_call)), span: Span::unknown() }];
                    let getter_name = format!("__get_birth_{}", name);
                    let getter = ASTNode::FunctionDeclaration { name: getter_name.clone(), params: vec![], body: getter_body, is_static: false, is_override: false, span: Span::unknown() };
                    methods.insert(getter_name, getter);
                } else {
                    // computed
                    let getter_name = format!("__get_{}", name);
                    let getter = ASTNode::FunctionDeclaration { name: getter_name.clone(), params: vec![], body: final_body, is_static: false, is_override: false, span: Span::unknown() };
                    methods.insert(getter_name, getter);
                }
                self.skip_newlines();
                continue;
            }

            // Fallback: method-level postfix catch/cleanup after a method (non-static box)
            if (self.match_token(&TokenType::CATCH) || self.match_token(&TokenType::CLEANUP)) && last_method_name.is_some() {
                let mname = last_method_name.clone().unwrap();
                let mut catch_clauses: Vec<crate::ast::CatchClause> = Vec::new();
                if self.match_token(&TokenType::CATCH) {
                    self.advance();
                    self.consume(TokenType::LPAREN)?;
                    let (exc_ty, exc_var) = self.parse_catch_param()?;
                    self.consume(TokenType::RPAREN)?;
                    let catch_body = self.parse_block_statements()?;
                    catch_clauses.push(crate::ast::CatchClause { exception_type: exc_ty, variable_name: exc_var, body: catch_body, span: crate::ast::Span::unknown() });
                    self.skip_newlines();
                    if self.match_token(&TokenType::CATCH) {
                        let line = self.current_token().line;
                        return Err(ParseError::UnexpectedToken { found: self.current_token().token_type.clone(), expected: "single catch only after method body".to_string(), line });
                    }
                }
                let finally_body = if self.match_token(&TokenType::CLEANUP) { self.advance(); Some(self.parse_block_statements()?) } else { None };
                if let Some(mnode) = methods.get_mut(&mname) {
                    if let crate::ast::ASTNode::FunctionDeclaration { body, .. } = mnode {
                        let already = body.iter().any(|n| matches!(n, crate::ast::ASTNode::TryCatch{..}));
                        if already {
                            let line = self.current_token().line;
                            return Err(ParseError::UnexpectedToken { found: self.current_token().token_type.clone(), expected: "duplicate postfix catch/cleanup after method".to_string(), line });
                        }
                        let old = std::mem::take(body);
                        *body = vec![crate::ast::ASTNode::TryCatch { try_body: old, catch_clauses, finally_body, span: crate::ast::Span::unknown() }];
                        continue;
                    }
                }
            }

            // RBRACEに到達していればループを抜ける
            if self.match_token(&TokenType::RBRACE) {
                break;
            }

            // initブロックの処理（initメソッドではない場合のみ）
            if self.match_token(&TokenType::INIT) && self.peek_token() != &TokenType::LPAREN {
                self.advance(); // consume 'init'
                self.consume(TokenType::LBRACE)?;

                // initブロック内のフィールド定義を読み込み
                while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                    self.skip_newlines();

                    if self.match_token(&TokenType::RBRACE) {
                        break;
                    }

                    // Check for weak modifier
                    let is_weak = if self.match_token(&TokenType::WEAK) {
                        self.advance(); // consume 'weak'
                        true
                    } else {
                        false
                    };

                    if let TokenType::IDENTIFIER(field_name) = &self.current_token().token_type {
                        init_fields.push(field_name.clone());
                        if is_weak {
                            weak_fields.push(field_name.clone()); // 🔗 Add to weak fields list
                        }
                        self.advance();

                        // カンマがあればスキップ
                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    } else {
                        // 不正なトークンがある場合はエラー
                        return Err(ParseError::UnexpectedToken {
                            expected: if is_weak {
                                "field name after 'weak'"
                            } else {
                                "field name"
                            }
                            .to_string(),
                            found: self.current_token().token_type.clone(),
                            line: self.current_token().line,
                        });
                    }
                }

                self.consume(TokenType::RBRACE)?;
                continue;
            }

            // overrideキーワードをチェック
            let mut is_override = false;
            if self.match_token(&TokenType::OVERRIDE) {
                is_override = true;
                self.advance();
            }

            // constructor parsing moved to members::constructors
            if let Some((ctor_key, ctor_node)) = crate::parser::declarations::box_def::members::constructors::try_parse_constructor(self, is_override)? {
                constructors.insert(ctor_key, ctor_node);
                continue;
            }

            // 🚨 birth()統一システム: Box名コンストラクタ無効化
            // Box名と同じ名前のコンストラクタは禁止（birth()のみ許可）
            if let TokenType::IDENTIFIER(id) = &self.current_token().token_type {
                if id == &name && self.peek_token() == &TokenType::LPAREN {
                    return Err(ParseError::UnexpectedToken {
                        expected: format!("birth() constructor instead of {}(). Nyash uses birth() for unified constructor syntax.", name),
                        found: TokenType::IDENTIFIER(name.clone()),
                        line: self.current_token().line,
                    });
                }
            }

            // 通常のフィールド名またはメソッド名、または unified members の先頭キーワードを読み取り
            if let TokenType::IDENTIFIER(field_or_method) = &self.current_token().token_type {
                let field_or_method = field_or_method.clone();
                self.advance();

                // 可視性:
                // - public { ... } / private { ... } ブロック
                // - public name: Type 単行（P0: 型はパースのみ、意味付けは後段）
                if field_or_method == "public" || field_or_method == "private" {
                    if self.match_token(&TokenType::LBRACE) {
                        // ブロック形式
                        self.advance(); // consume '{'
                        self.skip_newlines();
                        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                            if let TokenType::IDENTIFIER(fname) = &self.current_token().token_type {
                                let fname = fname.clone();
                                // ブロックに追加
                                if field_or_method == "public" {
                                    public_fields.push(fname.clone());
                                } else {
                                    private_fields.push(fname.clone());
                                }
                                // 互換性のため、全体fieldsにも追加
                                fields.push(fname);
                                self.advance();
                                // カンマ/改行をスキップ
                                if self.match_token(&TokenType::COMMA) {
                                    self.advance();
                                }
                                self.skip_newlines();
                                continue;
                            }
                            // 予期しないトークン
                            return Err(ParseError::UnexpectedToken {
                                expected: "identifier in visibility block".to_string(),
                                found: self.current_token().token_type.clone(),
                                line: self.current_token().line,
                            });
                        }
                        self.consume(TokenType::RBRACE)?;
                        self.skip_newlines();
                        continue;
                    } else if matches!(self.current_token().token_type, TokenType::IDENTIFIER(_)) {
                        // 単行形式: public/private name[: Type] (= init | => expr | { ... }[postfix]) を委譲
                        let fname = if let TokenType::IDENTIFIER(n) = &self.current_token().token_type {
                            n.clone()
                        } else {
                            unreachable!()
                        };
                        self.advance();
                        if crate::parser::declarations::box_def::members::fields::try_parse_header_first_field_or_property(
                            self,
                            fname.clone(),
                            &mut methods,
                            &mut fields,
                        )? {
                            if field_or_method == "public" {
                                public_fields.push(fname.clone());
                            } else {
                                private_fields.push(fname.clone());
                            }
                            // プロパティ経路ではメソッド後置（catch/cleanup）を無効化する
                            last_method_name = None;
                            self.skip_newlines();
                            continue;
                        } else {
                            // 解析不能な場合は従来どおりフィールド扱い（後方互換の保険）
                            if field_or_method == "public" { public_fields.push(fname.clone()); } else { private_fields.push(fname.clone()); }
                            fields.push(fname);
                            self.skip_newlines();
                            continue;
                        }
                    } else {
                        // public/private の後に '{' でも識別子でもない
                        return Err(ParseError::UnexpectedToken {
                            found: self.current_token().token_type.clone(),
                            expected: "'{' or field name".to_string(),
                            line: self.current_token().line,
                        });
                    }
                }

                // Unified Members (header-first) gate: support once/birth_once via members::properties
                if crate::config::env::unified_members() && (field_or_method == "once" || field_or_method == "birth_once") {
                    if crate::parser::declarations::box_def::members::properties::try_parse_unified_property(
                        self,
                        &field_or_method,
                        &mut methods,
                        &mut birth_once_props,
                    )? {
                        last_method_name = None; // do not attach method-level postfix here
                        self.skip_newlines();
                        continue;
                    }
                }

                // メソッド or フィールド（委譲）
                if let Some(method) = crate::parser::declarations::box_def::members::methods::try_parse_method(
                    self,
                    field_or_method.clone(),
                    is_override,
                    &birth_once_props,
                )? {
                    last_method_name = Some(field_or_method.clone());
                    methods.insert(field_or_method, method);
                } else {
                    // フィールド or 統一メンバ（computed/once/birth_once header-first）
                    let fname = field_or_method;
                    if crate::parser::declarations::box_def::members::fields::try_parse_header_first_field_or_property(
                        self,
                        fname,
                        &mut methods,
                        &mut fields,
                    )? {
                        continue;
                    }
                }
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "method or field name".to_string(),
                    found: self.current_token().token_type.clone(),
                    line: self.current_token().line,
                });
            }
        }

        self.consume(TokenType::RBRACE)?;

        // 🔥 Override validation
        for parent in &extends {
            self.validate_override_methods(&name, parent, &methods)?;
        }

        // birth_once 相互依存の簡易検出（宣言間の循環）
        if crate::config::env::unified_members() {
            // Collect birth_once compute bodies
            use std::collections::{HashMap, HashSet};
            let mut birth_bodies: HashMap<String, Vec<ASTNode>> = HashMap::new();
            for (mname, mast) in &methods {
                if let Some(prop) = mname.strip_prefix("__compute_birth_") {
                    if let ASTNode::FunctionDeclaration { body, .. } = mast {
                        birth_bodies.insert(prop.to_string(), body.clone());
                    }
                }
            }
            // Build dependency graph: A -> {B | me.B used inside A}
            let mut deps: HashMap<String, HashSet<String>> = HashMap::new();
            let props: HashSet<String> = birth_bodies.keys().cloned().collect();
            for (p, body) in &birth_bodies {
                let used = self.ast_collect_me_fields(body);
                let mut set = HashSet::new();
                for u in used {
                    if props.contains(&u) && u != *p {
                        set.insert(u);
                    }
                }
                deps.insert(p.clone(), set);
            }
            // Detect cycle via DFS
            fn has_cycle(
                node: &str,
                deps: &HashMap<String, HashSet<String>>,
                temp: &mut HashSet<String>,
                perm: &mut HashSet<String>,
            ) -> bool {
                if perm.contains(node) {
                    return false;
                }
                if !temp.insert(node.to_string()) {
                    return true; // back-edge
                }
                if let Some(ns) = deps.get(node) {
                    for n in ns {
                        if has_cycle(n, deps, temp, perm) {
                            return true;
                        }
                    }
                }
                temp.remove(node);
                perm.insert(node.to_string());
                false
            }
            let mut perm = HashSet::new();
            let mut temp = HashSet::new();
            for p in deps.keys() {
                if has_cycle(p, &deps, &mut temp, &mut perm) {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "birth_once declarations must not have cyclic dependencies".to_string(),
                        line,
                    });
                }
            }
        }

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
        self.consume(TokenType::INTERFACE)?;
        self.consume(TokenType::BOX)?;

        let name = if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            let name = name.clone();
            self.advance();
            name
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "identifier".to_string(),
                line,
            });
        };

        self.consume(TokenType::LBRACE)?;
        self.skip_newlines(); // ブレース後の改行をスキップ

        let mut methods = HashMap::new();

        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines(); // ループ開始時に改行をスキップ
            if let TokenType::IDENTIFIER(method_name) = &self.current_token().token_type {
                let method_name = method_name.clone();
                self.advance();

                // インターフェースメソッドはシグネチャのみ
                if self.match_token(&TokenType::LPAREN) {
                    self.advance(); // consume '('

                    let mut params = Vec::new();
                    while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                        if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                            params.push(param.clone());
                            self.advance();
                        }

                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    }

                    self.consume(TokenType::RPAREN)?;

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

                    // メソッド宣言後の改行をスキップ
                    self.skip_newlines();
                } else {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "(".to_string(),
                        line,
                    });
                }
            } else {
                let line = self.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "method name".to_string(),
                    line,
                });
            }
        }

        self.consume(TokenType::RBRACE)?;

        Ok(ASTNode::BoxDeclaration {
            name,
            fields: vec![], // インターフェースはフィールドなし
            public_fields: vec![],
            private_fields: vec![],
            methods,
            constructors: HashMap::new(), // インターフェースにコンストラクタなし
            init_fields: vec![],          // インターフェースにinitブロックなし
            weak_fields: vec![],          // 🔗 インターフェースにweak fieldsなし
            is_interface: true,           // インターフェースフラグ
            extends: vec![],              // 🚀 Multi-delegation: Changed from None to vec![]
            implements: vec![],
            type_parameters: Vec::new(), // 🔥 インターフェースではジェネリクス未対応
            is_static: false,            // インターフェースは非static
            static_init: None,           // インターフェースにstatic initなし
            span: Span::unknown(),
        })
    }
}

impl NyashParser {
    /// Minimal scan: does body contain `me.<field>` access (direct self-cycle guard)
    fn ast_contains_me_field(&self, nodes: &Vec<ASTNode>, field: &str) -> bool {
        fn scan(nodes: &Vec<ASTNode>, field: &str) -> bool {
            for n in nodes {
                match n {
                    ASTNode::FieldAccess { object, field: f, .. } => {
                        if f == field {
                            if let ASTNode::Me { .. } = object.as_ref() {
                                return true;
                            }
                        }
                    }
                    ASTNode::Program { statements, .. } => {
                        if scan(statements, field) { return true; }
                    }
                    ASTNode::If { then_body, else_body, .. } => {
                        if scan(then_body, field) { return true; }
                        if let Some(eb) = else_body { if scan(eb, field) { return true; } }
                    }
                    ASTNode::TryCatch { try_body, catch_clauses, finally_body, .. } => {
                        if scan(try_body, field) { return true; }
                        for c in catch_clauses { if scan(&c.body, field) { return true; } }
                        if let Some(fb) = finally_body { if scan(fb, field) { return true; } }
                    }
                    ASTNode::FunctionDeclaration { body, .. } => {
                        if scan(body, field) { return true; }
                    }
                    _ => {}
                }
            }
            false
        }
        scan(nodes, field)
    }

    /// Collect all `me.<field>` accessed in nodes (flat set)
    fn ast_collect_me_fields(&self, nodes: &Vec<ASTNode>) -> std::collections::HashSet<String> {
        use std::collections::HashSet;
        fn scan(nodes: &Vec<ASTNode>, out: &mut HashSet<String>) {
            for n in nodes {
                match n {
                    ASTNode::FieldAccess { object, field, .. } => {
                        if let ASTNode::Me { .. } = object.as_ref() {
                            out.insert(field.clone());
                        }
                    }
                    ASTNode::Program { statements, .. } => scan(statements, out),
                    ASTNode::If { then_body, else_body, .. } => {
                        scan(then_body, out);
                        if let Some(eb) = else_body { scan(eb, out); }
                    }
                    ASTNode::TryCatch { try_body, catch_clauses, finally_body, .. } => {
                        scan(try_body, out);
                        for c in catch_clauses { scan(&c.body, out); }
                        if let Some(fb) = finally_body { scan(fb, out); }
                    }
                    ASTNode::FunctionDeclaration { body, .. } => scan(body, out),
                    _ => {}
                }
            }
        }
        let mut hs = HashSet::new();
        scan(nodes, &mut hs);
        hs
    }
}
