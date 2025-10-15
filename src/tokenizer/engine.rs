use super::{NyashTokenizer, Token, TokenType, TokenizeError};

impl NyashTokenizer {
    #[inline]
    pub(crate) fn allow_semicolon() -> bool {
        match std::env::var("NYASH_PARSER_ALLOW_SEMICOLON").ok() {
            Some(v) => {
                let lv = v.to_ascii_lowercase();
                lv == "1" || lv == "true" || lv == "on"
            }
            // Default ON: semicolons are accepted as statement separators
            None => true,
        }
    }

    #[inline]
    pub(crate) fn strict_12_7() -> bool {
        std::env::var("NYASH_STRICT_12_7").ok().as_deref() == Some("1")
    }

    /// 新しいトークナイザーを作成
    pub fn new(input: impl Into<String>) -> Self {
        let input_string = input.into();
        Self {
            input: input_string.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    /// 完全なトークナイズを実行
    pub fn tokenize(&mut self) -> Result<Vec<Token>, TokenizeError> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            // 空白・コメントをスキップ
            self.skip_whitespace();
            // 連続するブロックコメントや行コメントもまとめてスキップ
            loop {
                // block comment: /* ... */
                if self.current_char() == Some('/') && self.peek_char() == Some('*') {
                    self.skip_block_comment()?;
                    self.skip_whitespace();
                    continue;
                }
                // line comments: // ... or # ...
                if (self.current_char() == Some('/') && self.peek_char() == Some('/'))
                    || self.current_char() == Some('#')
                {
                    self.skip_line_comment();
                    self.skip_whitespace();
                    continue;
                }
                break;
            }

            if self.is_at_end() {
                break;
            }

            // 次のトークンを読み取り
            let token = self.tokenize_next()?;
            if std::env::var("NYASH_TOK_TRACE").ok().as_deref() == Some("1") {
                eprintln!("[tok] {:?}", token.token_type);
            }
            tokens.push(token);
        }

        // EOF トークンを追加
        tokens.push(Token::new(TokenType::EOF, self.line, self.column));

        Ok(tokens)
    }

    /// 次の一つのトークンを読み取り
    fn tokenize_next(&mut self) -> Result<Token, TokenizeError> {
        let start_line = self.line;
        let start_column = self.column;

        match self.current_char() {
            // Optional statement separator ';' (gated)
            Some(';') if Self::allow_semicolon() => {
                self.advance();
                return Ok(Token::new(TokenType::SEMICOLON, start_line, start_column));
            }
            // Block comment should have been skipped by tokenize() pre-loop, but be defensive here
            Some('/') if self.peek_char() == Some('*') => {
                self.skip_block_comment()?;
                // After skipping, restart tokenization for next token
                return self.tokenize_next();
            }
            // 2文字（またはそれ以上）の演算子は最長一致で先に判定
            Some('|') if self.peek_char() == Some('|') => {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenType::OR, start_line, start_column));
            }
            Some('&') if self.peek_char() == Some('&') => {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenType::AND, start_line, start_column));
            }
            Some('|') if self.peek_char() == Some('>') => {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenType::PipeForward, start_line, start_column));
            }
            Some('|') if self.peek_char() == Some('?') && self.peek_char_n(2) == Some('>') => {
                // '|?>' tap operator (L2 sugar)
                self.advance(); // |
                self.advance(); // ?
                self.advance(); // >
                return Ok(Token::new(TokenType::PipeTap, start_line, start_column));
            }
            Some('?') if self.peek_char() == Some('.') => {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenType::QmarkDot, start_line, start_column));
            }
            Some('?') if self.peek_char() == Some('?') => {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenType::QmarkQmark, start_line, start_column));
            }
            Some('?') => {
                self.advance();
                return Ok(Token::new(TokenType::QUESTION, start_line, start_column));
            }
            Some('@') => {
                self.advance();
                return Ok(Token::new(TokenType::AT, start_line, start_column));
            }
            Some('+') if self.peek_char() == Some('=') => {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenType::PlusAssign, start_line, start_column));
            }
            Some('-') if self.peek_char() == Some('=') => {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenType::MinusAssign, start_line, start_column));
            }
            Some('*') if self.peek_char() == Some('=') => {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenType::MulAssign, start_line, start_column));
            }
            Some('/') if self.peek_char() == Some('=') => {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenType::DivAssign, start_line, start_column));
            }
            Some('.') if self.peek_char() == Some('.') => {
                self.advance();
                self.advance();
                return Ok(Token::new(TokenType::RANGE, start_line, start_column));
            }
            Some('"') => {
                let string_value = self.read_string()?;
                Ok(Token::new(
                    TokenType::STRING(string_value),
                    start_line,
                    start_column,
                ))
            }
            // Raw strings: r"..." or r#"..."# or r##"..."##
            Some('r') if self.peek_char() == Some('"') || self.peek_char() == Some('#') => {
                let s = self.read_raw_string()?;
                Ok(Token::new(TokenType::STRING(s), start_line, start_column))
            }
            Some(c) if c.is_ascii_digit() => {
                let token_type = self.read_numeric_literal()?;
                Ok(Token::new(token_type, start_line, start_column))
            }
            Some(c) if c.is_alphabetic() || c == '_' => {
                let token_type = self.read_keyword_or_identifier();
                Ok(Token::new(token_type, start_line, start_column))
            }
            Some('/') if self.peek_char() == Some('/') => {
                self.skip_line_comment();
                self.skip_whitespace(); // コメント後の空白もスキップ
                return self.tokenize_next();
            }
            Some('#') => {
                self.skip_line_comment();
                self.skip_whitespace(); // コメント後の空白もスキップ
                return self.tokenize_next();
            }
            Some('>') if self.peek_char() == Some('>') && !Self::strict_12_7() => {
                self.advance();
                self.advance();
                Ok(Token::new(TokenType::ShiftRight, start_line, start_column))
            }
            Some('<') if self.peek_char() == Some('<') && !Self::strict_12_7() => {
                self.advance();
                self.advance();
                Ok(Token::new(TokenType::ShiftLeft, start_line, start_column))
            }
            Some(':') if self.peek_char() == Some(':') => {
                self.advance();
                self.advance();
                Ok(Token::new(TokenType::DoubleColon, start_line, start_column))
            }
            Some(':') => {
                self.advance();
                Ok(Token::new(TokenType::COLON, start_line, start_column))
            }
            Some('=') if self.peek_char() == Some('>') => {
                self.advance();
                self.advance();
                Ok(Token::new(TokenType::FatArrow, start_line, start_column))
            }
            Some('=') if self.peek_char() == Some('=') => {
                self.advance();
                self.advance();
                Ok(Token::new(TokenType::EQUALS, start_line, start_column))
            }
            Some('!') if self.peek_char() == Some('=') => {
                self.advance();
                self.advance();
                Ok(Token::new(TokenType::NotEquals, start_line, start_column))
            }
            Some('<') if self.peek_char() == Some('=') => {
                self.advance();
                self.advance();
                Ok(Token::new(TokenType::LessEquals, start_line, start_column))
            }
            Some('>') if self.peek_char() == Some('=') => {
                self.advance();
                self.advance();
                Ok(Token::new(TokenType::GreaterEquals, start_line, start_column))
            }
            Some(c) => {
                if let Some(token) = self.single_char_token(c) {
                    self.advance();
                    Ok(Token::new(token, start_line, start_column))
                } else {
                    Err(TokenizeError::UnexpectedCharacter {
                        char: c,
                        line: start_line,
                        column: start_column,
                    })
                }
            }
            None => Ok(Token::new(TokenType::EOF, start_line, start_column)),
        }
    }

    // 単文字トークンのマップ（最長一致系は呼び出し元で処理済み）
    fn single_char_token(&self, c: char) -> Option<TokenType> {
        // '?' は上位で分岐済み、':' も同様。ここでは純粋な1文字を扱う。
        match c {
            '!' => Some(TokenType::NOT),
            '~' => Some(TokenType::BitNot),
            '<' => Some(TokenType::LESS),
            '>' => Some(TokenType::GREATER),
            '&' => Some(TokenType::BitAnd),
            '|' => Some(TokenType::BitOr),
            '^' => Some(TokenType::BitXor),
            '=' => Some(TokenType::ASSIGN),
            '+' => Some(TokenType::PLUS),
            '-' => Some(TokenType::MINUS),
            '*' => Some(TokenType::MULTIPLY),
            '/' => Some(TokenType::DIVIDE),
            '%' => Some(TokenType::MODULO),
            '.' => Some(TokenType::DOT),
            '(' => Some(TokenType::LPAREN),
            ')' => Some(TokenType::RPAREN),
            '[' => Some(TokenType::LBRACK),
            ']' => Some(TokenType::RBRACK),
            '{' => Some(TokenType::LBRACE),
            '}' => Some(TokenType::RBRACE),
            ',' => Some(TokenType::COMMA),
            '\n' => Some(TokenType::NEWLINE),
            _ => None,
        }
    }
}
