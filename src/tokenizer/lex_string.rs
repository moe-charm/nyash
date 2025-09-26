use super::{NyashTokenizer, TokenizeError};

impl NyashTokenizer {
    /// 文字列リテラルを読み取り
    pub(crate) fn read_string(&mut self) -> Result<String, TokenizeError> {
        let start_line = self.line;
        self.advance(); // 開始の '"' をスキップ

        let mut string_value = String::new();

        while let Some(c) = self.current_char() {
            if c == '"' {
                self.advance(); // 終了の '"' をスキップ
                return Ok(string_value);
            }

            // エスケープ文字の処理
            if c == '\\' {
                self.advance();
                match self.current_char() {
                    Some('n') => string_value.push('\n'),
                    Some('t') => string_value.push('\t'),
                    Some('r') => string_value.push('\r'),
                    Some('\\') => string_value.push('\\'),
                    Some('"') => string_value.push('"'),
                    Some(c) => {
                        string_value.push('\\');
                        string_value.push(c);
                    }
                    None => break,
                }
            } else {
                string_value.push(c);
            }

            self.advance();
        }

        Err(TokenizeError::UnterminatedString { line: start_line })
    }
}

