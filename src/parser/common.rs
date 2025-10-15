/*!
 * Parser Common Utilities
 *
 * パーサーモジュール間で共有されるヘルパー関数や型定義
 * Extracted from parser/mod.rs as part of modularization
 */

use super::ParseError;
use crate::ast::Span;
use crate::tokenizer::{Token, TokenType};

/// Parser utility methods
pub trait ParserUtils {
    fn tokens(&self) -> &Vec<Token>;
    fn current(&self) -> usize;
    fn current_mut(&mut self) -> &mut usize;

    /// 現在のトークンを取得
    fn current_token(&self) -> &Token {
        self.tokens().get(self.current()).unwrap_or(&Token {
            token_type: TokenType::EOF,
            line: 0,
            column: 0,
        })
    }

    /// 次のトークンを先読み（位置を進めない）
    fn peek_token(&self) -> &TokenType {
        if self.current() + 1 < self.tokens().len() {
            &self.tokens()[self.current() + 1].token_type
        } else {
            &TokenType::EOF
        }
    }

    /// N個先のトークンを先読み
    #[allow(dead_code)]
    fn peek_nth_token(&self, n: usize) -> &TokenType {
        if self.current() + n < self.tokens().len() {
            &self.tokens()[self.current() + n].token_type
        } else {
            &TokenType::EOF
        }
    }

    /// 位置を1つ進める（改行スキップ：Cursor無効時のみ最小限）
    fn advance(&mut self) {
        if !self.is_at_end() {
            // 現在のトークンで深度を更新（進める前）
            self.update_depth_before_advance();

            *self.current_mut() += 1;

            // 新しいトークンで深度を更新（進めた後）
            self.update_depth_after_advance();

            // 改行スキップは Cursor 無効時のみ最小限で行う（互換用）。
            // 環境変数 NYASH_PARSER_TOKEN_CURSOR=1 の場合は Cursor 側で一元管理する。
            let cursor_on = std::env::var("NYASH_PARSER_TOKEN_CURSOR").ok().as_deref() == Some("1");
            if !cursor_on {
                let allow_sc = std::env::var("NYASH_PARSER_ALLOW_SEMICOLON").ok().map(|v| {
                    let lv = v.to_ascii_lowercase();
                    lv == "1" || lv == "true" || lv == "on"
                }).unwrap_or(true);
                loop {
                    let is_nl = matches!(self.current_token().token_type, TokenType::NEWLINE);
                    let is_sc = allow_sc && matches!(self.current_token().token_type, TokenType::SEMICOLON);
                    if (is_nl || is_sc) && !self.is_at_end() {
                        *self.current_mut() += 1; // 非再帰的に前進
                        continue;
                    }
                    break;
                }
            }
        }
    }

    /// advance前の深度更新（閉じ括弧の処理）
    fn update_depth_before_advance(&mut self) {
        // デフォルト実装は何もしない（NyashParserでオーバーライド）
    }

    /// advance後の深度更新（開き括弧の処理）
    fn update_depth_after_advance(&mut self) {
        // デフォルト実装は何もしない（NyashParserでオーバーライド）
    }

    // 旧来の should_auto_skip_newlines / skip_newlines 系は撤去（Cursor に集約）

    /// 指定されたトークンタイプを消費 (期待通りでなければエラー)
    fn consume(&mut self, expected: TokenType) -> Result<Token, ParseError> {
        if std::mem::discriminant(&self.current_token().token_type)
            == std::mem::discriminant(&expected)
        {
            let token = self.current_token().clone();
            self.advance();
            Ok(token)
        } else {
            let line = self.current_token().line;
            Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: format!("{:?}", expected),
                line,
            })
        }
    }

    /// 現在のトークンが指定されたタイプかチェック
    fn match_token(&self, token_type: &TokenType) -> bool {
        std::mem::discriminant(&self.current_token().token_type)
            == std::mem::discriminant(token_type)
    }

    /// 複数のトークンタイプのいずれかにマッチするかチェック
    #[allow(dead_code)]
    fn match_any_token(&self, token_types: &[TokenType]) -> bool {
        let current_discriminant = std::mem::discriminant(&self.current_token().token_type);
        token_types
            .iter()
            .any(|tt| std::mem::discriminant(tt) == current_discriminant)
    }

    /// 終端に達したかチェック
    fn is_at_end(&self) -> bool {
        self.current() >= self.tokens().len()
            || matches!(self.current_token().token_type, TokenType::EOF)
    }

    /// 現在のトークンが行の終わり（NEWLINE or EOF）かチェック
    #[allow(dead_code)]
    fn is_line_end(&self) -> bool {
        matches!(
            self.current_token().token_type,
            TokenType::NEWLINE | TokenType::EOF
        )
    }

    /// エラー報告用の現在位置情報を取得
    #[allow(dead_code)]
    fn current_position(&self) -> (usize, usize) {
        let token = self.current_token();
        (token.line, token.column)
    }

    /// 現在のトークンからSpanを作成
    #[allow(dead_code)]
    fn current_span(&self) -> Span {
        let token = self.current_token();
        Span {
            start: 0, // Token doesn't have byte offset, so using 0
            end: 0,
            line: token.line,
            column: token.column,
        }
    }
}

/// Helper function to create unknown span
#[allow(dead_code)]
pub fn unknown_span() -> Span {
    Span::unknown()
}
