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
    /// Validate that birth_once properties do not have cyclic dependencies via me.<prop> references
    fn validate_birth_once_cycles(
        &mut self,
        methods: &HashMap<String, ASTNode>,
    ) -> Result<(), ParseError> {
        if !crate::config::env::unified_members() {
            return Ok(());
        }
        use std::collections::{HashMap, HashSet};
        // Collect birth_once compute bodies
        let mut birth_bodies: HashMap<String, Vec<ASTNode>> = HashMap::new();
        for (mname, mast) in methods {
            if let Some(prop) = mname.strip_prefix("__compute_birth_") {
                if let ASTNode::FunctionDeclaration { body, .. } = mast {
                    birth_bodies.insert(prop.to_string(), body.clone());
                }
            }
        }
        if birth_bodies.is_empty() {
            return Ok(());
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
        Ok(())
    }
    /// Extracted: parse block-first unified member: `{ body } as [once|birth_once]? name : Type [postfix]`
    /// Returns true if a member was parsed and emitted into `methods`.
    fn parse_unified_member_block_first(
        &mut self,
        methods: &mut HashMap<String, ASTNode>,
        birth_once_props: &mut Vec<String>,
    ) -> Result<bool, ParseError> {
        if !(crate::config::env::unified_members() && self.match_token(&TokenType::LBRACE)) {
            return Ok(false);
        }
        // 1) Parse block body first
        let mut final_body = self.parse_block_statements()?;
        self.skip_newlines();

        // 2) Expect 'as'
        if let TokenType::IDENTIFIER(kw) = &self.current_token().token_type {
            if kw != "as" {
                let line = self.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "'as' after block for block-first member".to_string(),
                    line,
                });
            }
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "'as' after block for block-first member".to_string(),
                line,
            });
        }
        self.advance(); // consume 'as'

        // 3) Optional kind keyword: once | birth_once
        let mut kind = "computed".to_string();
        if let TokenType::IDENTIFIER(k) = &self.current_token().token_type {
            if k == "once" || k == "birth_once" {
                kind = k.clone();
                self.advance();
            }
        }

        // 4) Name : Type
        let name = if let TokenType::IDENTIFIER(n) = &self.current_token().token_type {
            let s = n.clone();
            self.advance();
            s
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "identifier for member name".to_string(),
                line,
            });
        };
        if self.match_token(&TokenType::COLON) {
            self.advance();
            if let TokenType::IDENTIFIER(_ty) = &self.current_token().token_type {
                self.advance();
            } else {
                let line = self.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "type name after ':'".to_string(),
                    line,
                });
            }
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: ": type".to_string(),
                line,
            });
        }

        // 5) Optional postfix handlers (Stage‑3) directly after block (shared helper)
        final_body = crate::parser::declarations::box_def::members::postfix::wrap_with_optional_postfix(self, final_body)?;

        // 6) Generate methods per kind (fully equivalent to former inline branch)
        if kind == "once" {
            // __compute_once_<name>
            let compute_name = format!("__compute_once_{}", name);
            let compute = ASTNode::FunctionDeclaration {
                name: compute_name.clone(),
                params: vec![],
                body: final_body,
                is_static: false,
                is_override: false,
                span: Span::unknown(),
            };
            methods.insert(compute_name.clone(), compute);

            // Getter with cache + poison handling
            let key = format!("__once_{}", name);
            let poison_key = format!("__once_poison_{}", name);
            let cached_local = format!("__ny_cached_{}", name);
            let poison_local = format!("__ny_poison_{}", name);
            let val_local = format!("__ny_val_{}", name);
            let me_node = ASTNode::Me { span: Span::unknown() };
            let get_cached = ASTNode::MethodCall {
                object: Box::new(me_node.clone()),
                method: "getField".to_string(),
                arguments: vec![ASTNode::Literal {
                    value: crate::ast::LiteralValue::String(key.clone()),
                    span: Span::unknown(),
                }],
                span: Span::unknown(),
            };
            let local_cached = ASTNode::Local {
                variables: vec![cached_local.clone()],
                initial_values: vec![Some(Box::new(get_cached))],
                span: Span::unknown(),
            };
            let cond_cached = ASTNode::BinaryOp {
                operator: crate::ast::BinaryOperator::NotEqual,
                left: Box::new(ASTNode::Variable { name: cached_local.clone(), span: Span::unknown() }),
                right: Box::new(ASTNode::Literal { value: crate::ast::LiteralValue::Null, span: Span::unknown() }),
                span: Span::unknown(),
            };
            let then_ret_cached = vec![ASTNode::Return {
                value: Some(Box::new(ASTNode::Variable { name: cached_local.clone(), span: Span::unknown() })),
                span: Span::unknown(),
            }];
            let if_cached = ASTNode::If { condition: Box::new(cond_cached), then_body: then_ret_cached, else_body: None, span: Span::unknown() };

            let get_poison = ASTNode::MethodCall {
                object: Box::new(me_node.clone()),
                method: "getField".to_string(),
                arguments: vec![ASTNode::Literal { value: crate::ast::LiteralValue::String(poison_key.clone()), span: Span::unknown() }],
                span: Span::unknown(),
            };
            let local_poison = ASTNode::Local {
                variables: vec![poison_local.clone()],
                initial_values: vec![Some(Box::new(get_poison))],
                span: Span::unknown(),
            };
            let cond_poison = ASTNode::BinaryOp {
                operator: crate::ast::BinaryOperator::NotEqual,
                left: Box::new(ASTNode::Variable { name: poison_local.clone(), span: Span::unknown() }),
                right: Box::new(ASTNode::Literal { value: crate::ast::LiteralValue::Null, span: Span::unknown() }),
                span: Span::unknown(),
            };
            let then_throw = vec![ASTNode::Throw {
                expression: Box::new(ASTNode::Literal { value: crate::ast::LiteralValue::String(format!("once '{}' previously failed", name)), span: Span::unknown() }),
                span: Span::unknown(),
            }];
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
        Ok(true)
    }

    /// Extracted: method-level postfix catch/cleanup after a method
    fn parse_method_postfix_after_last_method(
        &mut self,
        methods: &mut HashMap<String, ASTNode>,
        last_method_name: &Option<String>,
    ) -> Result<bool, ParseError> {
        if !(self.match_token(&TokenType::CATCH) || self.match_token(&TokenType::CLEANUP))
            || last_method_name.is_none()
        {
            return Ok(false);
        }
        let mname = last_method_name.clone().unwrap();
        let mut catch_clauses: Vec<crate::ast::CatchClause> = Vec::new();
        if self.match_token(&TokenType::CATCH) {
            self.advance();
            self.consume(TokenType::LPAREN)?;
            let (exc_ty, exc_var) = self.parse_catch_param()?;
            self.consume(TokenType::RPAREN)?;
            let catch_body = self.parse_block_statements()?;
            catch_clauses.push(crate::ast::CatchClause {
                exception_type: exc_ty,
                variable_name: exc_var,
                body: catch_body,
                span: crate::ast::Span::unknown(),
            });
            self.skip_newlines();
            if self.match_token(&TokenType::CATCH) {
                let line = self.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "single catch only after method body".to_string(),
                    line,
                });
            }
        }
        let finally_body = if self.match_token(&TokenType::CLEANUP) {
            self.advance();
            Some(self.parse_block_statements()?)
        } else {
            None
        };
        if let Some(mnode) = methods.get_mut(&mname) {
            if let crate::ast::ASTNode::FunctionDeclaration { body, .. } = mnode {
                let already = body
                    .iter()
                    .any(|n| matches!(n, crate::ast::ASTNode::TryCatch { .. }));
                if already {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "duplicate postfix catch/cleanup after method".to_string(),
                        line,
                    });
                }
                let old = std::mem::take(body);
                *body = vec![crate::ast::ASTNode::TryCatch {
                    try_body: old,
                    catch_clauses,
                    finally_body,
                    span: crate::ast::Span::unknown(),
                }];
            }
        }
        Ok(true)
    }

    /// Extracted: init { ... } block (non-call form)
    fn parse_init_block_if_any(
        &mut self,
        init_fields: &mut Vec<String>,
        weak_fields: &mut Vec<String>,
    ) -> Result<bool, ParseError> {
        if !(self.match_token(&TokenType::INIT) && self.peek_token() != &TokenType::LPAREN) {
            return Ok(false);
        }
        self.advance(); // consume 'init'
        self.consume(TokenType::LBRACE)?;
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines();
            if self.match_token(&TokenType::RBRACE) {
                break;
            }
            let is_weak = if self.match_token(&TokenType::WEAK) {
                self.advance();
                true
            } else {
                false
            };
            if let TokenType::IDENTIFIER(field_name) = &self.current_token().token_type {
                init_fields.push(field_name.clone());
                if is_weak {
                    weak_fields.push(field_name.clone());
                }
                self.advance();
                if self.match_token(&TokenType::COMMA) {
                    self.advance();
                }
            } else {
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
        Ok(true)
    }

    /// Extracted: visibility block or single property header-first
    fn parse_visibility_block_or_single(
        &mut self,
        visibility: &str,
        methods: &mut HashMap<String, ASTNode>,
        fields: &mut Vec<String>,
        public_fields: &mut Vec<String>,
        private_fields: &mut Vec<String>,
        last_method_name: &mut Option<String>,
    ) -> Result<bool, ParseError> {
        if visibility != "public" && visibility != "private" {
            return Ok(false);
        }
        if self.match_token(&TokenType::LBRACE) {
            self.advance();
            self.skip_newlines();
            while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                if let TokenType::IDENTIFIER(fname) = &self.current_token().token_type {
                    let fname = fname.clone();
                    if visibility == "public" {
                        public_fields.push(fname.clone());
                    } else {
                        private_fields.push(fname.clone());
                    }
                    fields.push(fname);
                    self.advance();
                    if self.match_token(&TokenType::COMMA) {
                        self.advance();
                    }
                    self.skip_newlines();
                    continue;
                }
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier in visibility block".to_string(),
                    found: self.current_token().token_type.clone(),
                    line: self.current_token().line,
                });
            }
            self.consume(TokenType::RBRACE)?;
            self.skip_newlines();
            return Ok(true);
        }
        if let TokenType::IDENTIFIER(n) = &self.current_token().token_type {
            let fname = n.clone();
            self.advance();
            if crate::parser::declarations::box_def::members::fields::try_parse_header_first_field_or_property(
                self,
                fname.clone(),
                methods,
                fields,
            )? {
                if visibility == "public" {
                    public_fields.push(fname.clone());
                } else {
                    private_fields.push(fname.clone());
                }
                *last_method_name = None;
                self.skip_newlines();
                return Ok(true);
            } else {
                // Fallback: record visibility + field name for compatibility
                if visibility == "public" {
                    public_fields.push(fname.clone());
                } else {
                    private_fields.push(fname.clone());
                }
                fields.push(fname);
                self.skip_newlines();
                return Ok(true);
            }
        }
        Ok(false)
    }
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
            if self.parse_unified_member_block_first(&mut methods, &mut birth_once_props)? { continue; }

            // Fallback: method-level postfix catch/cleanup after a method (non-static box)
            if self.parse_method_postfix_after_last_method(&mut methods, &last_method_name)? { continue; }

            // RBRACEに到達していればループを抜ける
            if self.match_token(&TokenType::RBRACE) {
                break;
            }

            // initブロックの処理（initメソッドではない場合のみ）
            if self.parse_init_block_if_any(&mut init_fields, &mut weak_fields)? { continue; }

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

                // 可視性: public/private ブロック/単行
                if self.parse_visibility_block_or_single(
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
        self.validate_birth_once_cycles(&methods)?;

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
