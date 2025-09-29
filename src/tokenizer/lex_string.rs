use super::{NyashTokenizer, TokenizeError};

impl NyashTokenizer {
    /// Raw string literal reader: r"..." or r#"..."# or r##"..."##
    pub(crate) fn read_raw_string(&mut self) -> Result<String, TokenizeError> {
        // current is 'r' and next is '"' or '#'
        self.advance(); // consume 'r'
        // count leading '#'
        let mut hashes = 0usize;
        while let Some('#') = self.current_char() {
            hashes += 1;
            self.advance();
        }
        // next must be '"'
        if self.current_char() != Some('"') {
            return Err(TokenizeError::UnexpectedCharacter { char: self.current_char().unwrap_or('\0'), line: self.line, column: self.column });
        }
        self.advance(); // consume opening '"'

        let mut out = String::new();
        loop {
            match self.current_char() {
                None => return Err(TokenizeError::UnterminatedString { line: self.line }),
                Some('"') => {
                    // check for matching number of hashes
                    // lookahead sequence: '"' + hashes times '#'
                    let mut ok = true;
                    for i in 1..=hashes {
                        if self.peek_char_n(i) != Some('#') { ok = false; break; }
                    }
                    if ok {
                        self.advance(); // consume '"'
                        for _ in 0..hashes { self.advance(); }
                        break;
                    } else {
                        out.push('"');
                        self.advance();
                    }
                }
                Some(c) => { out.push(c); self.advance(); }
            }
        }
        Ok(out)
    }
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
