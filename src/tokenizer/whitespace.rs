use super::{NyashTokenizer, TokenizeError};

impl NyashTokenizer {
    /// 行コメントをスキップ
    pub(crate) fn skip_line_comment(&mut self) {
        while let Some(c) = self.current_char() {
            if c == '\n' {
                break; // 改行文字は消費せずに残す
            }
            self.advance();
        }
    }

    /// ブロックコメントをスキップ: /* ... */（ネスト非対応）
    pub(crate) fn skip_block_comment(&mut self) -> Result<(), TokenizeError> {
        // Assume current position is at '/' and next is '*'
        self.advance(); // '/'
        self.advance(); // '*'
        while let Some(c) = self.current_char() {
            // detect end '*/'
            if c == '*' && self.peek_char() == Some('/') {
                self.advance(); // '*'
                self.advance(); // '/'
                return Ok(());
            }
            self.advance();
        }
        // EOF reached without closing */
        Err(TokenizeError::UnterminatedComment { line: self.line })
    }

    /// 空白文字をスキップ（改行は除く：改行はNEWLINEトークンとして扱う）
    pub(crate) fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c.is_whitespace() && c != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
}

