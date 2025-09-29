use super::NyashTokenizer;

impl NyashTokenizer {
    /// 現在の文字を取得
    pub(crate) fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }

    /// 次の文字を先読み
    pub(crate) fn peek_char(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }

    /// n文字先を先読み（n>=1）
    pub(crate) fn peek_char_n(&self, n: usize) -> Option<char> {
        if n == 0 { return self.current_char(); }
        self.input.get(self.position + n).copied()
    }

    /// 1文字進める（行/列も更新）
    pub(crate) fn advance(&mut self) {
        if let Some(c) = self.current_char() {
            self.position += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
    }

    /// 入力の終端に到達しているか
    pub(crate) fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}
