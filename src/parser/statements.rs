/*!
 * Nyash Parser - Statement Parsing Module
 * 
 * 文（Statement）の解析を担当するモジュール
 * if, loop, break, return, print等の制御構文を処理
 */

use crate::tokenizer::TokenType;
use crate::ast::{ASTNode, CatchClause, Span};
use super::{NyashParser, ParseError};

impl NyashParser {
    /// 文をパース
    pub(super) fn parse_statement(&mut self) -> Result<ASTNode, ParseError> {
        
        let result = match &self.current_token().token_type {
            TokenType::BOX => {
                self.parse_box_declaration()
            },
            TokenType::INTERFACE => {
                self.parse_interface_box_declaration()
            },
            TokenType::GLOBAL => {
                self.parse_global_var()
            },
            TokenType::FUNCTION => {
                self.parse_function_declaration()
            },
            TokenType::STATIC => {
                self.parse_static_declaration()  // 🔥 静的宣言 (function/box)
            },
            TokenType::IF => {
                self.parse_if()
            },
            TokenType::LOOP => {
                self.parse_loop()
            },
            TokenType::BREAK => {
                self.parse_break()
            },
            TokenType::RETURN => {
                self.parse_return()
            },
            TokenType::PRINT => {
                self.parse_print()
            },
            TokenType::NOWAIT => {
                self.parse_nowait()
            },
            TokenType::INCLUDE => {
                self.parse_include()
            },
            TokenType::LOCAL => {
                self.parse_local()
            },
            TokenType::OUTBOX => {
                self.parse_outbox()
            },
            TokenType::TRY => {
                self.parse_try_catch()
            },
            TokenType::THROW => {
                self.parse_throw()
            },
            TokenType::FROM => {
                // 🔥 from構文: from Parent.method(args) または from Parent.constructor(args)
                self.parse_from_call_statement()
            },
            TokenType::IDENTIFIER(name) => {
                // function宣言 または 代入文 または 関数呼び出し
                self.parse_assignment_or_function_call()
            }
            TokenType::THIS | TokenType::ME => {
                // this/me で始まる文も通常の代入文または関数呼び出しとして処理
                self.parse_assignment_or_function_call()
            }
            _ => {
                let line = self.current_token().line;
                Err(ParseError::InvalidStatement { line })
            }
        };
        
        result
    }
    
    /// if文をパース: if (condition) { body } else if ... else { body }
    pub(super) fn parse_if(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'if'
        
        // 条件部分を取得
        let condition = Box::new(self.parse_expression()?);
        
        // then部分を取得
        self.consume(TokenType::LBRACE)?;
        let mut then_body = Vec::new();
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines();
            if !self.match_token(&TokenType::RBRACE) {
                then_body.push(self.parse_statement()?);
            }
        }
        self.consume(TokenType::RBRACE)?;
        
        // else if/else部分を処理
        let else_body = if self.match_token(&TokenType::ELSE) {
            self.advance(); // consume 'else'
            
            if self.match_token(&TokenType::IF) {
                // else if を ネストしたifとして処理
                let nested_if = self.parse_if()?;
                Some(vec![nested_if])
            } else {
                // plain else
                self.consume(TokenType::LBRACE)?;
                let mut else_stmts = Vec::new();
                while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                    self.skip_newlines();
                    if !self.match_token(&TokenType::RBRACE) {
                        else_stmts.push(self.parse_statement()?);
                    }
                }
                self.consume(TokenType::RBRACE)?;
                Some(else_stmts)
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
        
        // 条件部分を取得
        self.consume(TokenType::LPAREN)?;
        let condition = Some(Box::new(self.parse_expression()?));
        self.consume(TokenType::RPAREN)?;
        
        // body部分を取得
        self.consume(TokenType::LBRACE)?;
        let mut body = Vec::new();
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines();
            if !self.match_token(&TokenType::RBRACE) {
                body.push(self.parse_statement()?);
            }
        }
        self.consume(TokenType::RBRACE)?;
        
        Ok(ASTNode::Loop {
            condition: condition.unwrap(),
            body,
            span: Span::unknown(),
        })
    }
    
    /// break文をパース
    pub(super) fn parse_break(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'break'
        Ok(ASTNode::Break { span: Span::unknown() })
    }
    
    /// return文をパース
    pub(super) fn parse_return(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'return'
        
        // returnの後に式があるかチェック
        let value = if self.is_at_end() || self.match_token(&TokenType::NEWLINE) {
            // return単体の場合はvoidを返す
            None
        } else {
            // 式をパースして返す
            Some(Box::new(self.parse_expression()?))
        };
        
        Ok(ASTNode::Return { value, span: Span::unknown() })
    }
    
    /// print文をパース
    pub(super) fn parse_print(&mut self) -> Result<ASTNode, ParseError> {
        self.advance(); // consume 'print'
        self.consume(TokenType::LPAREN)?;
        let value = Box::new(self.parse_expression()?);
        self.consume(TokenType::RPAREN)?;
        
        Ok(ASTNode::Print { expression: value, span: Span::unknown() })
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
        
        Ok(ASTNode::Include { filename: path, span: Span::unknown() })
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
        self.consume(TokenType::LBRACE)?;
        
        let mut try_body = Vec::new();
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            self.skip_newlines();
            if !self.match_token(&TokenType::RBRACE) {
                try_body.push(self.parse_statement()?);
            }
        }
        
        self.consume(TokenType::RBRACE)?;
        
        let mut catch_clauses = Vec::new();
        
        // catch節をパース
        while self.match_token(&TokenType::CATCH) {
            self.advance(); // consume 'catch'
            self.consume(TokenType::LPAREN)?;
            
            // 例外型 (オプション)
            let exception_type = if let TokenType::IDENTIFIER(type_name) = &self.current_token().token_type {
                let type_name = type_name.clone();
                self.advance();
                Some(type_name)
            } else {
                None
            };
            
            // 例外変数名
            let exception_var = if let TokenType::IDENTIFIER(var_name) = &self.current_token().token_type {
                let var_name = var_name.clone();
                self.advance();
                var_name
            } else {
                let line = self.current_token().line;
                return Err(ParseError::UnexpectedToken {
                    found: self.current_token().token_type.clone(),
                    expected: "exception variable name".to_string(),
                    line,
                });
            };
            
            self.consume(TokenType::RPAREN)?;
            self.consume(TokenType::LBRACE)?;
            
            let mut catch_body = Vec::new();
            while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                self.skip_newlines();
                if !self.match_token(&TokenType::RBRACE) {
                    catch_body.push(self.parse_statement()?);
                }
            }
            
            self.consume(TokenType::RBRACE)?;
            
            catch_clauses.push(CatchClause {
                exception_type,
                variable_name: Some(exception_var),
                body: catch_body,
                span: Span::unknown(),
            });
        }
        
        // finally節をパース (オプション)
        let finally_body = if self.match_token(&TokenType::FINALLY) {
            self.advance(); // consume 'finally'
            self.consume(TokenType::LBRACE)?;
            
            let mut body = Vec::new();
            while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
                self.skip_newlines();
                if !self.match_token(&TokenType::RBRACE) {
                    body.push(self.parse_statement()?);
                }
            }
            
            self.consume(TokenType::RBRACE)?;
            Some(body)
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
        Ok(ASTNode::Throw { expression: value, span: Span::unknown() })
    }
    
    /// 🔥 from構文を文としてパース: from Parent.method(args)
    pub(super) fn parse_from_call_statement(&mut self) -> Result<ASTNode, ParseError> {
        // 既存のparse_from_call()を使用してFromCall ASTノードを作成
        let from_call_expr = self.parse_from_call()?;
        
        // FromCallは式でもあるが、文としても使用可能
        // 例: from Animal.constructor() （戻り値を使わない）
        Ok(from_call_expr)
    }
}