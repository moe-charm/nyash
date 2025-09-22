/*!
 * Nyash Parser - Statement Parsing Module
 *
 * 文（Statement）の解析を担当するモジュール
 * if, loop, break, return, print等の制御構文を処理
 */

use super::common::ParserUtils;
use super::{NyashParser, ParseError};
use crate::ast::{ASTNode, CatchClause, Span};
use crate::tokenizer::TokenType;

impl NyashParser {
    /// Parse a standalone block `{ ... }` and optional postfix `catch/cleanup` sequence.
    /// Returns Program(body) when no postfix keywords follow.
    fn parse_standalone_block_statement(&mut self) -> Result<ASTNode, ParseError> {
        // Parse the block body first
        let try_body = self.parse_block_statements()?;

        // Allow whitespace/newlines between block and postfix keywords
        self.skip_newlines();

        if crate::config::env::block_postfix_catch()
            && (self.match_token(&TokenType::CATCH) || self.match_token(&TokenType::CLEANUP))
        {
            // Parse at most one catch, then optional cleanup
            let mut catch_clauses: Vec<CatchClause> = Vec::new();
            if self.match_token(&TokenType::CATCH) {
                self.advance(); // consume 'catch'
                self.consume(TokenType::LPAREN)?;
                let (exception_type, exception_var) = self.parse_catch_param()?;
                self.consume(TokenType::RPAREN)?;
                let catch_body = self.parse_block_statements()?;
                catch_clauses.push(CatchClause {
                    exception_type,
                    variable_name: exception_var,
                    body: catch_body,
                    span: Span::unknown(),
                });

                // Single‑catch policy (MVP): disallow multiple catch in postfix form
                self.skip_newlines();
                if self.match_token(&TokenType::CATCH) {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "single catch only after standalone block".to_string(),
                        line,
                    });
                }
            }

            // Optional cleanup
            let finally_body = if self.match_token(&TokenType::CLEANUP) {
                self.advance(); // consume 'cleanup'
                Some(self.parse_block_statements()?)
            } else {
                None
            };

            Ok(ASTNode::TryCatch {
                try_body,
                catch_clauses,
                finally_body,
                span: Span::unknown(),
            })
        } else {
            // No postfix keywords. If gate is on, enforce MVP static check:
            // direct top-level `throw` inside the standalone block must be followed by catch
            if crate::config::env::block_postfix_catch()
                && try_body.iter().any(|n| matches!(n, ASTNode::Throw { .. }))
            {
                let line = self.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "block with direct 'throw' must be followed by 'catch'".to_string(),
                    line,
                });
            }
            Ok(ASTNode::Program {
                statements: try_body,
                span: Span::unknown(),
            })
        }
    }
    /// Helper: parse a block `{ stmt* }` and return its statements
    pub(super) fn parse_block_statements(&mut self) -> Result<Vec<ASTNode>, ParseError> {
        self.consume(TokenType::LBRACE)?;
        let mut body = Vec::new();
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines();
            if !self.match_token(&TokenType::RBRACE) {
                body.push(self.parse_statement()?);
            }
        }
        self.consume(TokenType::RBRACE)?;
        Ok(body)
    }

    /// Grouped: declarations (box/interface/global/function/static/import)
    fn parse_declaration_statement(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            TokenType::BOX => self.parse_box_declaration(),
            TokenType::IMPORT => self.parse_import(),
            TokenType::INTERFACE => self.parse_interface_box_declaration(),
            TokenType::GLOBAL => self.parse_global_var(),
            TokenType::FUNCTION => self.parse_function_declaration(),
            TokenType::STATIC => self.parse_static_declaration(),
            _ => {
                let line = self.current_token().line;
                Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "declaration statement".to_string(),
                    line,
                })
            }
        }
    }

    /// Grouped: control flow (if/loop/break/continue/return)
    fn parse_control_flow_statement(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            TokenType::IF => self.parse_if(),
            TokenType::LOOP => self.parse_loop(),
            TokenType::BREAK => self.parse_break(),
            TokenType::CONTINUE => self.parse_continue(),
            TokenType::RETURN => self.parse_return(),
            _ => {
                let line = self.current_token().line;
                Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "control-flow statement".to_string(),
                    line,
                })
            }
        }
    }

    /// Grouped: IO/module-ish (print/nowait/include)
    fn parse_io_module_statement(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            TokenType::PRINT => self.parse_print(),
            TokenType::NOWAIT => self.parse_nowait(),
            TokenType::INCLUDE => self.parse_include(),
            _ => {
                let line = self.current_token().line;
                Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "io/module statement".to_string(),
                    line,
                })
            }
        }
    }

    /// Grouped: variable-related (local/outbox)
    fn parse_variable_declaration_statement(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            TokenType::LOCAL => self.parse_local(),
            TokenType::OUTBOX => self.parse_outbox(),
            _ => {
                let line = self.current_token().line;
                Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "variable declaration".to_string(),
                    line,
                })
            }
        }
    }

    /// Grouped: exception (try/throw) with gate checks preserved
    fn parse_exception_statement(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            TokenType::TRY => {
                if crate::config::env::parser_stage3() {
                    self.parse_try_catch()
                } else {
                    Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "enable NYASH_PARSER_STAGE3=1 to use 'try'".to_string(),
                        line: self.current_token().line,
                    })
                }
            }
            TokenType::THROW => {
                if crate::config::env::parser_stage3() {
                    self.parse_throw()
                } else {
                    Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "enable NYASH_PARSER_STAGE3=1 to use 'throw'".to_string(),
                        line: self.current_token().line,
                    })
                }
            }
            _ => {
                let line = self.current_token().line;
                Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "try/throw".to_string(),
                    line,
                })
            }
        }
    }

    /// Error helpers for standalone postfix keywords (catch/cleanup)
    fn parse_postfix_catch_cleanup_error(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            TokenType::CATCH => {
                if crate::config::env::block_postfix_catch() {
                    Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "postfix 'catch' is only allowed immediately after a standalone block: { ... } catch (...) { ... } (wrap if/else/loop in a standalone block)".to_string(),
                        line: self.current_token().line,
                    })
                } else {
                    Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "enable NYASH_BLOCK_CATCH=1 (or NYASH_PARSER_STAGE3=1) to use postfix 'catch' after a standalone block".to_string(),
                        line: self.current_token().line,
                    })
                }
            }
            TokenType::CLEANUP => {
                if crate::config::env::block_postfix_catch() {
                    Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "postfix 'cleanup' is only allowed immediately after a standalone block: { ... } cleanup { ... }".to_string(),
                        line: self.current_token().line,
                    })
                } else {
                    Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "enable NYASH_BLOCK_CATCH=1 (or NYASH_PARSER_STAGE3=1) to use postfix 'cleanup' after a standalone block".to_string(),
                        line: self.current_token().line,
                    })
                }
            }
            _ => unreachable!(),
        }
    }

    /// Helper: parse catch parameter inside parentheses (after '(' consumed)
    /// Forms: (Type ident) | (ident) | ()
    pub(super) fn parse_catch_param(&mut self) -> Result<(Option<String>, Option<String>), ParseError> {
        if self.match_token(&TokenType::RPAREN) {
            return Ok((None, None));
        }
        match &self.current_token().token_type {
            TokenType::IDENTIFIER(first) => {
                let first_str = first.clone();
                let two_idents = matches!(self.peek_token(), TokenType::IDENTIFIER(_));
                if two_idents {
                    self.advance(); // consume type ident
                    if let TokenType::IDENTIFIER(var_name) = &self.current_token().token_type {
                        let var = var_name.clone();
                        self.advance();
                        Ok((Some(first_str), Some(var)))
                    } else {
                        let line = self.current_token().line;
                        Err(ParseError::UnexpectedToken { found: self.current_token().token_type.clone(), expected: "exception variable name".to_string(), line })
                    }
                } else {
                    self.advance();
                    Ok((None, Some(first_str)))
                }
            }
            _ => {
                if self.match_token(&TokenType::RPAREN) {
                    Ok((None, None))
                } else {
                    let line = self.current_token().line;
                    Err(ParseError::UnexpectedToken { found: self.current_token().token_type.clone(), expected: ") or identifier".to_string(), line })
                }
            }
        }
    }

    /// 文をパース
    pub(super) fn parse_statement(&mut self) -> Result<ASTNode, ParseError> {
        // For grammar diff: capture starting token to classify statement keyword
        let start_tok = self.current_token().token_type.clone();
        let result = match &start_tok {
            TokenType::LBRACE => self.parse_standalone_block_statement(),
            TokenType::BOX
            | TokenType::IMPORT
            | TokenType::INTERFACE
            | TokenType::GLOBAL
            | TokenType::FUNCTION
            | TokenType::STATIC => self.parse_declaration_statement(),
            TokenType::IF
            | TokenType::LOOP
            | TokenType::BREAK
            | TokenType::CONTINUE
            | TokenType::RETURN => self.parse_control_flow_statement(),
            TokenType::PRINT | TokenType::NOWAIT | TokenType::INCLUDE => self.parse_io_module_statement(),
            TokenType::LOCAL | TokenType::OUTBOX => self.parse_variable_declaration_statement(),
            TokenType::TRY | TokenType::THROW => self.parse_exception_statement(),
            TokenType::CATCH | TokenType::CLEANUP => self.parse_postfix_catch_cleanup_error(),
            TokenType::USING => self.parse_using(),
            TokenType::FROM => {
                // 🔥 from構文: from Parent.method(args) または from Parent.constructor(args)
                self.parse_from_call_statement()
            }
            TokenType::IDENTIFIER(_name) => {
                // function宣言 または 代入文 または 関数呼び出し
                self.parse_assignment_or_function_call()
            }
            TokenType::THIS | TokenType::ME => {
                // this/me で始まる文も通常の代入文または関数呼び出しとして処理
                self.parse_assignment_or_function_call()
            }
            _ => {
                // Fallback: treat as expression statement
                // Allows forms like: print("x")  or a bare literal as the last value in a block
                Ok(self.parse_expression()?)
            }
        };

        // Non-invasive syntax rule check
        if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
            let kw = match start_tok {
                TokenType::BOX => Some("box"),
                TokenType::GLOBAL => Some("global"),
                TokenType::FUNCTION => Some("function"),
                TokenType::STATIC => Some("static"),
                TokenType::IF => Some("if"),
                TokenType::LOOP => Some("loop"),
                TokenType::BREAK => Some("break"),
                TokenType::RETURN => Some("return"),
                TokenType::PRINT => Some("print"),
                TokenType::NOWAIT => Some("nowait"),
                TokenType::INCLUDE => Some("include"),
                TokenType::LOCAL => Some("local"),
                TokenType::OUTBOX => Some("outbox"),
                TokenType::TRY => Some("try"),
                TokenType::THROW => Some("throw"),
                TokenType::USING => Some("using"),
                TokenType::FROM => Some("from"),
                _ => None,
            };
            if let Some(k) = kw {
                let ok = crate::grammar::engine::get().syntax_is_allowed_statement(k);
                if !ok {
                    eprintln!(
                        "[GRAMMAR-DIFF][Parser] statement '{}' not allowed by syntax rules",
                        k
                    );
                }
            }
        }
        result
    }

    /// import文をパース: import "path" (as Alias)?
    pub(super) fn parse_import(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'import'
        let path = if let TokenType::STRING(s) = &self.current_token().token_type {
            let v = s.clone();
            self.advance();
            v
        } else {
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "string literal".to_string(),
                line: self.current_token().line,
            });
        };
        // Optional: 'as' Alias (treat 'as' as identifier literal)
        let mut alias: Option<String> = None;
        if let TokenType::IDENTIFIER(w) = &self.current_token().token_type {
            if w == "as" {
                self.advance();
                if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
                    alias = Some(name.clone());
                    self.advance();
                } else {
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "alias name".to_string(),
                        line: self.current_token().line,
                    });
                }
            }
        }
        Ok(ASTNode::ImportStatement {
            path,
            alias,
            span: Span::unknown(),
        })
    }

    /// if文をパース: if (condition) { body } else if ... else { body }
    pub(super) fn parse_if(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'if'

        // 条件部分を取得
        let condition = Box::new(self.parse_expression()?);

        // then部分を取得（共通ブロックヘルパー）
        let then_body = self.parse_block_statements()?;

        // else if/else部分を処理
        let else_body = if self.match_token(&TokenType::ELSE) {
            self.advance(); // consume 'else'

            if self.match_token(&TokenType::IF) {
                // else if を ネストしたifとして処理
                let nested_if = self.parse_if()?;
                Some(vec![nested_if])
            } else {
                // plain else（共通ブロックヘルパー）
                Some(self.parse_block_statements()?)
            }
        } else {
            None
        };

        Ok(ASTNode::If {
            condition,
            then_body,
            else_body,
            span: Span::unknown(),
        })
    }

    /// loop文をパース
    pub(super) fn parse_loop(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'loop'

        // 条件部分を取得（省略可: `loop { ... }` は無条件ループとして扱う）
        let condition = if self.match_token(&TokenType::LPAREN) {
            self.advance(); // consume '('
            let cond = Box::new(self.parse_expression()?);
            self.consume(TokenType::RPAREN)?;
            cond
        } else {
            // default: true
            Box::new(ASTNode::Literal {
                value: crate::ast::LiteralValue::Bool(true),
                span: Span::unknown(),
            })
        };

        // body部分を取得（共通ブロックヘルパー）
        let body = self.parse_block_statements()?;

        Ok(ASTNode::Loop {
            condition,
            body,
            span: Span::unknown(),
        })
    }

    /// break文をパース
    pub(super) fn parse_break(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'break'
        Ok(ASTNode::Break {
            span: Span::unknown(),
        })
    }

    /// continue文をパース
    pub(super) fn parse_continue(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'continue'
        Ok(ASTNode::Continue {
            span: Span::unknown(),
        })
    }

    /// return文をパース
    pub(super) fn parse_return(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'return'
                        // 許容: 改行をスキップしてから式有無を判定
        self.skip_newlines();
        // returnの後に式があるかチェック（RBRACE/EOFなら値なし）
        let value = if self.is_at_end() || self.match_token(&TokenType::RBRACE) {
            None
        } else {
            Some(Box::new(self.parse_expression()?))
        };

        Ok(ASTNode::Return {
            value,
            span: Span::unknown(),
        })
    }

    /// print文をパース
    pub(super) fn parse_print(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'print'
        self.consume(TokenType::LPAREN)?;
        let value = Box::new(self.parse_expression()?);
        self.consume(TokenType::RPAREN)?;

        Ok(ASTNode::Print {
            expression: value,
            span: Span::unknown(),
        })
    }

    /// nowait文をパース: nowait variable = expression
    pub(super) fn parse_nowait(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'nowait'

        // 変数名を取得
        let variable = if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            let name = name.clone();
            self.advance();
            name
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "variable name".to_string(),
                line,
            });
        };

        self.consume(TokenType::ASSIGN)?;
        let expression = Box::new(self.parse_expression()?);

        Ok(ASTNode::Nowait {
            variable,
            expression,
            span: Span::unknown(),
        })
    }

    /// include文をパース
    pub(super) fn parse_include(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'include'

        let path = if let TokenType::STRING(path) = &self.current_token().token_type {
            let path = path.clone();
            self.advance();
            path
        } else {
            let line = self.current_token().line;
            return Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "string literal".to_string(),
                line,
            });
        };

        Ok(ASTNode::Include {
            filename: path,
            span: Span::unknown(),
        })
    }

    /// local変数宣言をパース: local var1, var2, var3 または local x = 10
    pub(super) fn parse_local(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'local'

        let mut names = Vec::new();
        let mut initial_values = Vec::new();

        // 最初の変数名を取得
        if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            names.push(name.clone());
            self.advance();

            // = があれば初期値を設定
            if self.match_token(&TokenType::ASSIGN) {
                self.advance(); // consume '='
                initial_values.push(Some(Box::new(self.parse_expression()?)));

                // 初期化付きlocalは単一変数のみ（カンマ区切り不可）
                Ok(ASTNode::Local {
                    variables: names,
                    initial_values,
                    span: Span::unknown(),
                })
            } else {
                // 初期化なしの場合はカンマ区切りで複数変数可能
                initial_values.push(None);

                // カンマ区切りで追加の変数名を取得
                while self.match_token(&TokenType::COMMA) {
                    self.advance(); // consume ','

                    if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
                        names.push(name.clone());
                        initial_values.push(None);
                        self.advance();
                    } else {
                        let line = self.current_token().line;
                        return Err(ParseError::UnexpectedToken {
                            found: self.current_token().token_type.clone(),
                            expected: "identifier".to_string(),
                            line,
                        });
                    }
                }

                Ok(ASTNode::Local {
                    variables: names,
                    initial_values,
                    span: Span::unknown(),
                })
            }
        } else {
            let line = self.current_token().line;
            Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "identifier".to_string(),
                line,
            })
        }
    }

    /// outbox変数宣言をパース: outbox var1, var2, var3
    pub(super) fn parse_outbox(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'outbox'

        let mut names = Vec::new();

        // 最初の変数名を取得
        if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
            names.push(name.clone());
            self.advance();

            // カンマ区切りで追加の変数名を取得
            while self.match_token(&TokenType::COMMA) {
                self.advance(); // consume ','

                if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
                    names.push(name.clone());
                    self.advance();
                } else {
                    let line = self.current_token().line;
                    return Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "identifier".to_string(),
                        line,
                    });
                }
            }

            let num_vars = names.len();
            Ok(ASTNode::Outbox {
                variables: names,
                initial_values: vec![None; num_vars],
                span: Span::unknown(),
            })
        } else {
            let line = self.current_token().line;
            Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "identifier".to_string(),
                line,
            })
        }
    }

    /// try-catch文をパース
    pub(super) fn parse_try_catch(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'try'
        let try_body = self.parse_block_statements()?;

        let mut catch_clauses = Vec::new();

        // catch節をパース
        while self.match_token(&TokenType::CATCH) {
            self.advance(); // consume 'catch'
            self.consume(TokenType::LPAREN)?;
            let (exception_type, exception_var) = self.parse_catch_param()?;
            self.consume(TokenType::RPAREN)?;
            let catch_body = self.parse_block_statements()?;

            catch_clauses.push(CatchClause {
                exception_type,
                variable_name: exception_var,
                body: catch_body,
                span: Span::unknown(),
            });
        }

        // cleanup節をパース (オプション)
        let finally_body = if self.match_token(&TokenType::CLEANUP) {
            self.advance(); // consume 'cleanup'
            Some(self.parse_block_statements()?)
        } else {
            None
        };

        Ok(ASTNode::TryCatch {
            try_body,
            catch_clauses,
            finally_body,
            span: Span::unknown(),
        })
    }

    /// throw文をパース
    pub(super) fn parse_throw(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'throw'
        let value = Box::new(self.parse_expression()?);
        Ok(ASTNode::Throw {
            expression: value,
            span: Span::unknown(),
        })
    }

    /// 🔥 from構文を文としてパース: from Parent.method(args)
    pub(super) fn parse_from_call_statement(&mut self) -> Result<ASTNode, ParseError> {
        // 既存のparse_from_call()を使用してFromCall ASTノードを作成
        let from_call_expr = self.parse_from_call()?;

        // FromCallは式でもあるが、文としても使用可能
        // 例: from Animal.constructor() （戻り値を使わない）
        Ok(from_call_expr)
    }

    /// using文をパース: using namespace_name
    pub(super) fn parse_using(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'using'

        // 名前空間名を取得
        if let TokenType::IDENTIFIER(namespace_name) = &self.current_token().token_type {
            let name = namespace_name.clone();
            self.advance();

            // Phase 0では "nyashstd" のみ許可
            if name != "nyashstd" {
                return Err(ParseError::UnsupportedNamespace {
                    name,
                    line: self.current_token().line,
                });
            }

            Ok(ASTNode::UsingStatement {
                namespace_name: name,
                span: Span::unknown(),
            })
        } else {
            Err(ParseError::ExpectedIdentifier {
                line: self.current_token().line,
            })
        }
    }
}
