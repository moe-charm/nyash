use crate::tokenizer::{Token, TokenType};

/// トークンカーソル - 改行処理を一元管理
#[derive(Debug)]
pub struct TokenCursor<'a> {
    tokens: &'a [Token],
    idx: usize,
    mode: NewlineMode,
    paren_depth: usize,   // ()
    brace_depth: usize,   // {}
    bracket_depth: usize, // []
}

/// 改行処理モード
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NewlineMode {
    /// 文モード：改行は文の区切り
    Stmt,
    /// 式モード：改行を自動スキップ
    Expr,
}

impl<'a> TokenCursor<'a> {
    /// 新しいTokenCursorを作成
    pub fn new(tokens: &'a [Token]) -> Self {
        Self {
            tokens,
            idx: 0,
            mode: NewlineMode::Stmt,
            paren_depth: 0,
            brace_depth: 0,
            bracket_depth: 0,
        }
    }

    /// 現在のトークンを取得
    pub fn current(&self) -> &Token {
        self.tokens.get(self.idx).unwrap_or(&Token {
            token_type: TokenType::EOF,
            line: 0,
            column: 0,
        })
    }

    /// 次のトークンをピーク
    pub fn peek(&self) -> &Token {
        self.tokens.get(self.idx + 1).unwrap_or(&Token {
            token_type: TokenType::EOF,
            line: 0,
            column: 0,
        })
    }

    /// N番目のトークンをピーク
    pub fn peek_nth(&self, n: usize) -> &Token {
        self.tokens.get(self.idx + n).unwrap_or(&Token {
            token_type: TokenType::EOF,
            line: 0,
            column: 0,
        })
    }

    /// 次のトークンに進む（改行を考慮）
    pub fn advance(&mut self) {
        if self.idx < self.tokens.len() {
            // 深度を更新
            match &self.tokens[self.idx].token_type {
                TokenType::LPAREN => self.paren_depth += 1,
                TokenType::RPAREN => self.paren_depth = self.paren_depth.saturating_sub(1),
                TokenType::LBRACE => self.brace_depth += 1,
                TokenType::RBRACE => self.brace_depth = self.brace_depth.saturating_sub(1),
                TokenType::LBRACK => self.bracket_depth += 1,
                TokenType::RBRACK => self.bracket_depth = self.bracket_depth.saturating_sub(1),
                _ => {}
            }

            self.idx += 1;

            // 改行を自動的にスキップするかチェック
            while self.should_skip_newline() && self.idx < self.tokens.len() {
                if matches!(self.tokens[self.idx].token_type, TokenType::NEWLINE) {
                    self.idx += 1;
                } else {
                    break;
                }
            }
        }
    }

    /// 明示的に改行をスキップ
    pub fn skip_newlines(&mut self) {
        while self.idx < self.tokens.len()
            && matches!(self.tokens[self.idx].token_type, TokenType::NEWLINE) {
            self.idx += 1;
        }
    }

    /// トークンが期待した型かチェック
    pub fn match_token(&self, token_type: &TokenType) -> bool {
        std::mem::discriminant(&self.current().token_type) == std::mem::discriminant(token_type)
    }

    /// 期待したトークンを消費
    pub fn consume(&mut self, expected: TokenType) -> Result<(), crate::parser::ParseError> {
        if self.match_token(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(crate::parser::ParseError::UnexpectedToken {
                found: self.current().token_type.clone(),
                expected: format!("{:?}", expected),
                line: self.current().line,
            })
        }
    }

    /// ファイル終端かチェック
    pub fn is_at_end(&self) -> bool {
        matches!(self.current().token_type, TokenType::EOF)
    }

    /// 式モードで一時的に実行
    pub fn with_expr_mode<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let old_mode = self.mode;
        self.mode = NewlineMode::Expr;
        let result = f(self);
        self.mode = old_mode;
        result
    }

    /// 文モードで一時的に実行
    pub fn with_stmt_mode<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let old_mode = self.mode;
        self.mode = NewlineMode::Stmt;
        let result = f(self);
        self.mode = old_mode;
        result
    }

    /// 改行をスキップすべきか判定
    fn should_skip_newline(&self) -> bool {
        // ブレース/パーレン/ブラケット内では常にスキップ
        if self.brace_depth > 0 || self.paren_depth > 0 || self.bracket_depth > 0 {
            return true;
        }

        // 式モードでは改行をスキップ
        if self.mode == NewlineMode::Expr {
            return true;
        }

        // 行継続判定（直前のトークンを見る）
        if self.prev_is_line_continuation() {
            return true;
        }

        false
    }

    /// 直前のトークンが行継続を示すか判定
    fn prev_is_line_continuation(&self) -> bool {
        if self.idx == 0 {
            return false;
        }

        match &self.tokens[self.idx - 1].token_type {
            // 二項演算子
            TokenType::PLUS | TokenType::MINUS | TokenType::MULTIPLY | TokenType::DIVIDE |
            TokenType::MODULO | TokenType::AND | TokenType::OR |
            TokenType::BitOr | TokenType::BitAnd | TokenType::BitXor |
            // メンバアクセス
            TokenType::DOT | TokenType::DoubleColon |
            // Optional系
            TokenType::QUESTION |
            // Arrow
            TokenType::FatArrow |
            // カンマ
            TokenType::COMMA => true,
            _ => false,
        }
    }

    /// 現在の位置を取得
    pub fn position(&self) -> usize {
        self.idx
    }

    /// 位置を設定（バックトラック用）
    pub fn set_position(&mut self, pos: usize) {
        if pos <= self.tokens.len() {
            self.idx = pos;
            // 深度を再計算
            self.recalculate_depths();
        }
    }

    /// 深度を再計算
    fn recalculate_depths(&mut self) {
        self.paren_depth = 0;
        self.brace_depth = 0;
        self.bracket_depth = 0;

        for i in 0..self.idx {
            match &self.tokens[i].token_type {
                TokenType::LPAREN => self.paren_depth += 1,
                TokenType::RPAREN => self.paren_depth = self.paren_depth.saturating_sub(1),
                TokenType::LBRACE => self.brace_depth += 1,
                TokenType::RBRACE => self.brace_depth = self.brace_depth.saturating_sub(1),
                TokenType::LBRACK => self.bracket_depth += 1,
                TokenType::RBRACK => self.bracket_depth = self.bracket_depth.saturating_sub(1),
                _ => {}
            }
        }
    }

    /// モードを取得
    pub fn get_mode(&self) -> NewlineMode {
        self.mode
    }

    /// モードを設定
    pub fn set_mode(&mut self, mode: NewlineMode) {
        self.mode = mode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_cursor_operations() {
        let tokens = vec![
            Token { token_type: TokenType::LOCAL, line: 1, column: 1 },
            Token { token_type: TokenType::IDENTIFIER("x".to_string()), line: 1, column: 7 },
            Token { token_type: TokenType::ASSIGN, line: 1, column: 9 },
            Token { token_type: TokenType::NUMBER(42), line: 1, column: 11 },
            Token { token_type: TokenType::EOF, line: 1, column: 13 },
        ];

        let mut cursor = TokenCursor::new(&tokens);

        assert!(cursor.match_token(&TokenType::LOCAL));
        cursor.advance();

        assert!(matches!(cursor.current().token_type, TokenType::IDENTIFIER(_)));
        cursor.advance();

        assert!(cursor.match_token(&TokenType::ASSIGN));
        cursor.advance();

        assert!(matches!(cursor.current().token_type, TokenType::NUMBER(42)));
        cursor.advance();

        assert!(cursor.is_at_end());
    }

    #[test]
    fn test_newline_skipping_in_braces() {
        let tokens = vec![
            Token { token_type: TokenType::LBRACE, line: 1, column: 1 },
            Token { token_type: TokenType::NEWLINE, line: 1, column: 2 },
            Token { token_type: TokenType::IDENTIFIER("x".to_string()), line: 2, column: 1 },
            Token { token_type: TokenType::RBRACE, line: 2, column: 2 },
            Token { token_type: TokenType::EOF, line: 2, column: 3 },
        ];

        let mut cursor = TokenCursor::new(&tokens);

        cursor.advance(); // consume LBRACE, should skip NEWLINE
        assert!(matches!(cursor.current().token_type, TokenType::IDENTIFIER(_)));
    }

    #[test]
    fn test_expr_mode() {
        let tokens = vec![
            Token { token_type: TokenType::IDENTIFIER("x".to_string()), line: 1, column: 1 },
            Token { token_type: TokenType::NEWLINE, line: 1, column: 2 },
            Token { token_type: TokenType::PLUS, line: 2, column: 1 },
            Token { token_type: TokenType::NUMBER(1), line: 2, column: 3 },
            Token { token_type: TokenType::EOF, line: 2, column: 4 },
        ];

        let mut cursor = TokenCursor::new(&tokens);

        cursor.with_expr_mode(|c| {
            c.advance(); // consume IDENTIFIER, should skip NEWLINE in expr mode
            assert!(c.match_token(&TokenType::PLUS));
        });
    }
}