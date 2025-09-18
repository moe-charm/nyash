/*!
 * Box Definition Parser Module
 *
 * Box宣言（box, interface box, static box）の解析を担当
 * Nyashの中核概念「Everything is Box」を実現する重要モジュール
 */

use crate::ast::{ASTNode, Span};
use crate::must_advance;
use crate::parser::common::ParserUtils;
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;
use std::collections::HashMap;

impl NyashParser {
    /// box宣言をパース: box Name { fields... methods... }
    pub fn parse_box_declaration(&mut self) -> Result<ASTNode, ParseError> {
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

        // 🔥 ジェネリクス型パラメータのパース (<T, U>)
        let type_parameters = if self.match_token(&TokenType::LESS) {
            self.advance(); // consume '<'
            let mut params = Vec::new();

            while !self.match_token(&TokenType::GREATER) && !self.is_at_end() {
                must_advance!(self, _unused, "generic type parameter parsing");

                if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                    params.push(param.clone());
                    self.advance();

                    if self.match_token(&TokenType::COMMA) {
                        self.advance();
                        self.skip_newlines();
                    }
                } else {
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "type parameter name".to_string(),
                        line: self.current_token().line,
                    });
                }
            }

            self.consume(TokenType::GREATER)?; // consume '>'
            params
        } else {
            Vec::new()
        };

        // 🚀 Multi-delegation support: "from Parent1, Parent2, ..."
        let extends = if self.match_token(&TokenType::FROM) {
            self.advance(); // consume 'from'
            let mut parents = Vec::new();

            // Parse first parent (required)
            if let TokenType::IDENTIFIER(parent) = &self.current_token().token_type {
                parents.push(parent.clone());
                self.advance();
            } else {
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "parent box name after 'from'".to_string(),
                    line: self.current_token().line,
                });
            }

            // Parse additional parents (optional)
            while self.match_token(&TokenType::COMMA) {
                self.advance(); // consume ','
                self.skip_newlines();

                if let TokenType::IDENTIFIER(parent) = &self.current_token().token_type {
                    parents.push(parent.clone());
                    self.advance();
                } else {
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "parent box name after comma".to_string(),
                        line: self.current_token().line,
                    });
                }
            }

            parents
        } else {
            Vec::new()
        };

        // implementsキーワードのチェック
        // TODO: TokenType::IMPLEMENTS is not defined in current version
        let implements = if false {
            // self.match_token(&TokenType::IMPLEMENTS) {
            self.advance(); // consume 'implements'
            let mut interfaces = Vec::new();

            loop {
                must_advance!(self, _unused, "interface implementation parsing");

                if let TokenType::IDENTIFIER(interface) = &self.current_token().token_type {
                    interfaces.push(interface.clone());
                    self.advance();
                } else {
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "interface name".to_string(),
                        line: self.current_token().line,
                    });
                }

                if self.match_token(&TokenType::COMMA) {
                    self.advance();
                } else {
                    break;
                }
            }

            interfaces
        } else {
            Vec::new()
        };

        self.consume(TokenType::LBRACE)?;
        self.skip_newlines(); // ブレース後の改行をスキップ

        let mut fields = Vec::new();
        let mut methods = HashMap::new();
        let mut public_fields: Vec<String> = Vec::new();
        let mut private_fields: Vec<String> = Vec::new();
        let mut constructors = HashMap::new();
        let mut init_fields = Vec::new();
        let mut weak_fields = Vec::new(); // 🔗 Track weak fields

        let mut last_method_name: Option<String> = None;
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines(); // ループ開始時に改行をスキップ

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

            // initトークンをメソッド名として特別処理
            if self.match_token(&TokenType::INIT) && self.peek_token() == &TokenType::LPAREN {
                let field_or_method = "init".to_string();
                self.advance(); // consume 'init'

                // コンストラクタとして処理
                if self.match_token(&TokenType::LPAREN) {
                    // initは常にコンストラクタ
                    if is_override {
                        return Err(ParseError::UnexpectedToken {
                            expected: "method definition, not constructor after override keyword"
                                .to_string(),
                            found: TokenType::INIT,
                            line: self.current_token().line,
                        });
                    }
                    // コンストラクタの処理
                    self.advance(); // consume '('

                    let mut params = Vec::new();
                    while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                        must_advance!(self, _unused, "constructor parameter parsing");

                        if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                            params.push(param.clone());
                            self.advance();
                        }

                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    }

                    self.consume(TokenType::RPAREN)?;
                    let mut body = self.parse_block_statements()?;
                    self.skip_newlines();

                    // Method-level postfix catch/cleanup (gate)
                    if self.match_token(&TokenType::CATCH) || self.match_token(&TokenType::CLEANUP)
                    {
                        let mut catch_clauses: Vec<crate::ast::CatchClause> = Vec::new();
                        if self.match_token(&TokenType::CATCH) {
                            self.advance(); // consume 'catch'
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
                        // Wrap original body with TryCatch
                        body = vec![ASTNode::TryCatch {
                            try_body: body,
                            catch_clauses,
                            finally_body,
                            span: crate::ast::Span::unknown(),
                        }];
                    }

                    let constructor = ASTNode::FunctionDeclaration {
                        name: field_or_method.clone(),
                        params: params.clone(),
                        body,
                        is_static: false,
                        is_override: false, // コンストラクタは常に非オーバーライド
                        span: Span::unknown(),
                    };

                    // 🔥 init/引数数 形式でキーを作成（インタープリターと一致させる）
                    let constructor_key = format!("{}/{}", field_or_method, params.len());
                    constructors.insert(constructor_key, constructor);
                    continue;
                }
            }

            // packキーワードの処理（ビルトインBox継承用）
            if self.match_token(&TokenType::PACK) && self.peek_token() == &TokenType::LPAREN {
                let field_or_method = "pack".to_string();
                self.advance(); // consume 'pack'

                // packは常にコンストラクタ
                if is_override {
                    return Err(ParseError::UnexpectedToken {
                        expected: "method definition, not constructor after override keyword"
                            .to_string(),
                        found: TokenType::PACK,
                        line: self.current_token().line,
                    });
                }
                // packコンストラクタの処理
                self.advance(); // consume '('

                let mut params = Vec::new();
                while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                    must_advance!(self, _unused, "pack parameter parsing");

                    if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                        params.push(param.clone());
                        self.advance();
                    }

                    if self.match_token(&TokenType::COMMA) {
                        self.advance();
                    }
                }

                self.consume(TokenType::RPAREN)?;
                let body = self.parse_block_statements()?;

                let constructor = ASTNode::FunctionDeclaration {
                    name: field_or_method.clone(),
                    params: params.clone(),
                    body,
                    is_static: false,
                    is_override: false, // packは常に非オーバーライド
                    span: Span::unknown(),
                };

                // 🔥 pack/引数数 形式でキーを作成（インタープリターと一致させる）
                let constructor_key = format!("{}/{}", field_or_method, params.len());
                constructors.insert(constructor_key, constructor);
                continue;
            }

            // birthキーワードの処理（生命を与えるコンストラクタ）
            if self.match_token(&TokenType::BIRTH) && self.peek_token() == &TokenType::LPAREN {
                let field_or_method = "birth".to_string();
                self.advance(); // consume 'birth'

                // birthは常にコンストラクタ
                if is_override {
                    return Err(ParseError::UnexpectedToken {
                        expected: "method definition, not constructor after override keyword"
                            .to_string(),
                        found: TokenType::BIRTH,
                        line: self.current_token().line,
                    });
                }
                // birthコンストラクタの処理
                self.advance(); // consume '('

                let mut params = Vec::new();
                while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                    must_advance!(self, _unused, "birth parameter parsing");

                    if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                        params.push(param.clone());
                        self.advance();
                    }

                    if self.match_token(&TokenType::COMMA) {
                        self.advance();
                    }
                }

                self.consume(TokenType::RPAREN)?;
                let body = self.parse_block_statements()?;

                let constructor = ASTNode::FunctionDeclaration {
                    name: field_or_method.clone(),
                    params: params.clone(),
                    body,
                    is_static: false,
                    is_override: false, // birthは常に非オーバーライド
                    span: Span::unknown(),
                };

                // 🔥 birth/引数数 形式でキーを作成（インタープリターと一致させる）
                let constructor_key = format!("{}/{}", field_or_method, params.len());
                constructors.insert(constructor_key, constructor);
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

            // 通常のフィールド名またはメソッド名を読み取り
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
                        // 単行形式: public name[: Type]
                        let fname =
                            if let TokenType::IDENTIFIER(n) = &self.current_token().token_type {
                                n.clone()
                            } else {
                                unreachable!()
                            };
                        self.advance();
                        if self.match_token(&TokenType::COLON) {
                            self.advance(); // consume ':'
                                            // 型名（識別子）を受理して破棄（P0）
                            if let TokenType::IDENTIFIER(_ty) = &self.current_token().token_type {
                                self.advance();
                            } else {
                                return Err(ParseError::UnexpectedToken {
                                    found: self.current_token().token_type.clone(),
                                    expected: "type name".to_string(),
                                    line: self.current_token().line,
                                });
                            }
                        }
                        if field_or_method == "public" {
                            public_fields.push(fname.clone());
                        } else {
                            private_fields.push(fname.clone());
                        }
                        fields.push(fname);
                        self.skip_newlines();
                        continue;
                    } else {
                        // public/private の後に '{' でも識別子でもない
                        return Err(ParseError::UnexpectedToken {
                            found: self.current_token().token_type.clone(),
                            expected: "'{' or field name".to_string(),
                            line: self.current_token().line,
                        });
                    }
                }

                // メソッドかフィールドかを判定
                if self.match_token(&TokenType::LPAREN) {
                    // メソッド定義
                    self.advance(); // consume '('

                    let mut params = Vec::new();
                    while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
                        must_advance!(self, _unused, "method parameter parsing");

                        if let TokenType::IDENTIFIER(param) = &self.current_token().token_type {
                            params.push(param.clone());
                            self.advance();
                        }

                        if self.match_token(&TokenType::COMMA) {
                            self.advance();
                        }
                    }

                    self.consume(TokenType::RPAREN)?;
                    let body = self.parse_block_statements()?;

                    let method = ASTNode::FunctionDeclaration {
                        name: field_or_method.clone(),
                        params,
                        body,
                        is_static: false,
                        is_override,
                        span: Span::unknown(),
                    };

                    last_method_name = Some(field_or_method.clone());
                    methods.insert(field_or_method, method);
                } else {
                    // フィールド定義（P0: 型注釈 name: Type を受理して破棄）
                    let fname = field_or_method;
                    if self.match_token(&TokenType::COLON) {
                        self.advance(); // consume ':'
                                        // 型名（識別子）を許可（P0は保持せず破棄）
                        if let TokenType::IDENTIFIER(_ty) = &self.current_token().token_type {
                            self.advance();
                        }
                    }
                    fields.push(fname);
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
