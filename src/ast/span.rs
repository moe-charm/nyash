use std::fmt;

/// ソースコード位置情報 - エラー報告とデバッグの革命
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Span {
    pub start: usize,  // 開始位置（バイトオフセット）
    pub end: usize,    // 終了位置（バイトオフセット）
    pub line: usize,   // 行番号（1から開始）
    pub column: usize, // 列番号（1から開始）
}

impl Span {
    /// 新しいSpanを作成
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self {
            start,
            end,
            line,
            column,
        }
    }

    /// デフォルトのSpan（不明な位置）
    pub fn unknown() -> Self {
        Self {
            start: 0,
            end: 0,
            line: 1,
            column: 1,
        }
    }

    /// 2つのSpanを結合（開始位置から終了位置まで）
    pub fn merge(&self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
            line: self.line,
            column: self.column,
        }
    }

    /// ソースコードから該当箇所を抽出してエラー表示用文字列を生成
    pub fn error_context(&self, source: &str) -> String {
        let lines: Vec<&str> = source.lines().collect();
        if self.line == 0 || self.line > lines.len() {
            return format!("line {}, column {}", self.line, self.column);
        }

        let line_content = lines[self.line - 1];
        let mut context = String::new();

        // 行番号とソース行を表示
        context.push_str(&format!("   |\n{:3} | {}\n", self.line, line_content));

        // カーソル位置を表示（簡易版）
        if self.column > 0 && self.column <= line_content.len() + 1 {
            context.push_str("   | ");
            for _ in 1..self.column {
                context.push(' ');
            }
            let span_length = if self.end > self.start {
                (self.end - self.start).min(line_content.len() - self.column + 1)
            } else {
                1
            };
            for _ in 0..span_length.max(1) {
                context.push('^');
            }
            context.push('\n');
        }

        context
    }

    /// 位置情報の文字列表現
    pub fn location_string(&self) -> String {
        format!("line {}, column {}", self.line, self.column)
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}, column {}", self.line, self.column)
    }
}
