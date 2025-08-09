/*!
 * Nyash Tokenizer - .nyashソースコードをトークン列に変換
 * 
 * Python版nyashc_v4.pyのNyashTokenizerをRustで完全再実装
 * 正規表現ベース → 高速なcharレベル処理に最適化
 */

use thiserror::Error;

/// トークンの種類を表すenum
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // リテラル
    STRING(String),
    NUMBER(i64),
    FLOAT(f64),  // 浮動小数点数サポート追加
    TRUE,
    FALSE,
    
    // キーワード
    BOX,
    GLOBAL,
    SINGLETON,
    NEW,
    IF,
    ELSE,
    LOOP,
    BREAK,
    RETURN,
    FUNCTION,
    PRINT,
    THIS,
    ME,
    INIT,            // init (初期化ブロック)
    NOWAIT,          // nowait
    AWAIT,           // await
    INTERFACE,       // interface
    FROM,            // from (継承用)
    INCLUDE,         // include (ファイル読み込み)
    TRY,             // try
    CATCH,           // catch
    FINALLY,         // finally
    THROW,           // throw
    LOCAL,           // local (一時変数宣言)
    STATIC,          // static (静的メソッド)
    OUTBOX,          // outbox (所有権移転変数)
    NOT,             // not (否定演算子)
    
    // 演算子 (長いものから先に定義)
    ARROW,           // >>
    EQUALS,          // ==
    NotEquals,       // !=
    LessEquals,      // <=
    GreaterEquals,   // >=
    AND,             // && または and
    OR,              // || または or
    LESS,            // <
    GREATER,         // >
    ASSIGN,          // =
    PLUS,            // +
    MINUS,           // -
    MULTIPLY,        // *
    DIVIDE,          // /
    
    // 記号
    DOT,             // .
    LPAREN,          // (
    RPAREN,          // )
    LBRACE,          // {
    RBRACE,          // }
    COMMA,           // ,
    NEWLINE,         // \n
    
    // 識別子
    IDENTIFIER(String),
    
    // 特殊
    EOF,
}

/// トークンの位置情報を含む構造体
#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, line: usize, column: usize) -> Self {
        Self { token_type, line, column }
    }
}

/// トークナイズエラー
#[derive(Error, Debug)]
pub enum TokenizeError {
    #[error("Unexpected character '{char}' at line {line}, column {column}")]
    UnexpectedCharacter { char: char, line: usize, column: usize },
    
    #[error("Unterminated string literal at line {line}")]
    UnterminatedString { line: usize },
    
    #[error("Invalid number format at line {line}")]
    InvalidNumber { line: usize },
    
    #[error("Comment not closed at line {line}")]
    UnterminatedComment { line: usize },
}

/// Nyashトークナイザー
pub struct NyashTokenizer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl NyashTokenizer {
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
            // 空白をスキップ
            self.skip_whitespace();
            
            if self.is_at_end() {
                break;
            }
            
            // 次のトークンを読み取り
            let token = self.tokenize_next()?;
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
            Some('"') => {
                let string_value = self.read_string()?;
                Ok(Token::new(TokenType::STRING(string_value), start_line, start_column))
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
            Some('>') if self.peek_char() == Some('>') => {
                self.advance();
                self.advance();
                Ok(Token::new(TokenType::ARROW, start_line, start_column))
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
            Some('&') if self.peek_char() == Some('&') => {
                self.advance();
                self.advance();
                Ok(Token::new(TokenType::AND, start_line, start_column))
            }
            Some('|') if self.peek_char() == Some('|') => {
                self.advance();
                self.advance();
                Ok(Token::new(TokenType::OR, start_line, start_column))
            }
            Some('<') => {
                self.advance();
                Ok(Token::new(TokenType::LESS, start_line, start_column))
            }
            Some('>') => {
                self.advance();
                Ok(Token::new(TokenType::GREATER, start_line, start_column))
            }
            Some('=') => {
                self.advance();
                Ok(Token::new(TokenType::ASSIGN, start_line, start_column))
            }
            Some('+') => {
                self.advance();
                Ok(Token::new(TokenType::PLUS, start_line, start_column))
            }
            Some('-') => {
                self.advance();
                Ok(Token::new(TokenType::MINUS, start_line, start_column))
            }
            Some('*') => {
                self.advance();
                Ok(Token::new(TokenType::MULTIPLY, start_line, start_column))
            }
            Some('/') => {
                self.advance();
                Ok(Token::new(TokenType::DIVIDE, start_line, start_column))
            }
            Some('.') => {
                self.advance();
                Ok(Token::new(TokenType::DOT, start_line, start_column))
            }
            Some('(') => {
                self.advance();
                Ok(Token::new(TokenType::LPAREN, start_line, start_column))
            }
            Some(')') => {
                self.advance();
                Ok(Token::new(TokenType::RPAREN, start_line, start_column))
            }
            Some('{') => {
                self.advance();
                Ok(Token::new(TokenType::LBRACE, start_line, start_column))
            }
            Some('}') => {
                self.advance();
                Ok(Token::new(TokenType::RBRACE, start_line, start_column))
            }
            Some(',') => {
                self.advance();
                Ok(Token::new(TokenType::COMMA, start_line, start_column))
            }
            Some('\n') => {
                self.advance();
                Ok(Token::new(TokenType::NEWLINE, start_line, start_column))
            }
            Some(c) => {
                Err(TokenizeError::UnexpectedCharacter {
                    char: c,
                    line: self.line,
                    column: self.column,
                })
            }
            None => {
                Ok(Token::new(TokenType::EOF, self.line, self.column))
            }
        }
    }
    
    /// 文字列リテラルを読み取り
    fn read_string(&mut self) -> Result<String, TokenizeError> {
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
    
    /// 数値リテラル（整数または浮動小数点数）を読み取り
    fn read_numeric_literal(&mut self) -> Result<TokenType, TokenizeError> {
        let start_line = self.line;
        let mut number_str = String::new();
        let mut has_dot = false;
        
        // 整数部分を読み取り
        while let Some(c) = self.current_char() {
            if c.is_ascii_digit() {
                number_str.push(c);
                self.advance();
            } else if c == '.' && !has_dot && self.peek_char().map_or(false, |ch| ch.is_ascii_digit()) {
                // 小数点の後に数字が続く場合のみ受け入れる
                has_dot = true;
                number_str.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        if has_dot {
            // 浮動小数点数として解析
            number_str.parse::<f64>()
                .map(TokenType::FLOAT)
                .map_err(|_| TokenizeError::InvalidNumber { line: start_line })
        } else {
            // 整数として解析
            number_str.parse::<i64>()
                .map(TokenType::NUMBER)
                .map_err(|_| TokenizeError::InvalidNumber { line: start_line })
        }
    }
    
    /// キーワードまたは識別子を読み取り
    fn read_keyword_or_identifier(&mut self) -> TokenType {
        let mut identifier = String::new();
        
        while let Some(c) = self.current_char() {
            if c.is_alphanumeric() || c == '_' {
                identifier.push(c);
                self.advance();
            } else {
                break;
            }
        }
        
        // キーワードチェック
        match identifier.as_str() {
            "box" => TokenType::BOX,
            "global" => TokenType::GLOBAL,
            "singleton" => TokenType::SINGLETON,
            "new" => TokenType::NEW,
            "if" => TokenType::IF,
            "else" => TokenType::ELSE,
            "loop" => TokenType::LOOP,
            "break" => TokenType::BREAK,
            "return" => TokenType::RETURN,
            "function" => TokenType::FUNCTION,
            "print" => TokenType::PRINT,
            "this" => TokenType::THIS,
            "me" => TokenType::ME,
            "init" => TokenType::INIT,
            "nowait" => TokenType::NOWAIT,
            "await" => TokenType::AWAIT,
            "interface" => TokenType::INTERFACE,
            "from" => TokenType::FROM,
            "include" => TokenType::INCLUDE,
            "try" => TokenType::TRY,
            "catch" => TokenType::CATCH,
            "finally" => TokenType::FINALLY,
            "throw" => TokenType::THROW,
            "local" => TokenType::LOCAL,
            "static" => TokenType::STATIC,
            "outbox" => TokenType::OUTBOX,
            "not" => TokenType::NOT,
            "and" => TokenType::AND,
            "or" => TokenType::OR,
            "true" => TokenType::TRUE,
            "false" => TokenType::FALSE,
            _ => TokenType::IDENTIFIER(identifier),
        }
    }
    
    /// 行コメントをスキップ
    fn skip_line_comment(&mut self) {
        while let Some(c) = self.current_char() {
            if c == '\n' {
                break; // 改行文字は消費せずに残す
            }
            self.advance();
        }
    }
    
    /// 空白文字をスキップ（改行は除く）
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current_char() {
            if c.is_whitespace() && c != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }
    
    /// 現在の文字を取得
    fn current_char(&self) -> Option<char> {
        self.input.get(self.position).copied()
    }
    
    /// 次の文字を先読み
    fn peek_char(&self) -> Option<char> {
        self.input.get(self.position + 1).copied()
    }
    
    /// 位置を1つ進める
    fn advance(&mut self) {
        if let Some(c) = self.current_char() {
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
    }
    
    /// 入力の終端に達したかチェック
    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_simple_tokens() {
        let mut tokenizer = NyashTokenizer::new("box new = + - *");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 7); // 6 tokens + EOF
        assert_eq!(tokens[0].token_type, TokenType::BOX);
        assert_eq!(tokens[1].token_type, TokenType::NEW);
        assert_eq!(tokens[2].token_type, TokenType::ASSIGN);
        assert_eq!(tokens[3].token_type, TokenType::PLUS);
        assert_eq!(tokens[4].token_type, TokenType::MINUS);
        assert_eq!(tokens[5].token_type, TokenType::MULTIPLY);
        assert_eq!(tokens[6].token_type, TokenType::EOF);
    }
    
    #[test]
    fn test_string_literal() {
        let mut tokenizer = NyashTokenizer::new(r#""Hello, World!""#);
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 2); // STRING + EOF
        match &tokens[0].token_type {
            TokenType::STRING(s) => assert_eq!(s, "Hello, World!"),
            _ => panic!("Expected STRING token"),
        }
    }
    
    #[test]
    fn test_number_literal() {
        let mut tokenizer = NyashTokenizer::new("42 123 0");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 4); // 3 numbers + EOF
        match &tokens[0].token_type {
            TokenType::NUMBER(n) => assert_eq!(*n, 42),
            _ => panic!("Expected NUMBER token"),
        }
        match &tokens[1].token_type {
            TokenType::NUMBER(n) => assert_eq!(*n, 123),
            _ => panic!("Expected NUMBER token"),
        }
        match &tokens[2].token_type {
            TokenType::NUMBER(n) => assert_eq!(*n, 0),
            _ => panic!("Expected NUMBER token"),
        }
    }
    
    #[test]
    fn test_identifier() {
        let mut tokenizer = NyashTokenizer::new("test_var myBox getValue");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens.len(), 4); // 3 identifiers + EOF
        match &tokens[0].token_type {
            TokenType::IDENTIFIER(s) => assert_eq!(s, "test_var"),
            _ => panic!("Expected IDENTIFIER token"),
        }
        match &tokens[1].token_type {
            TokenType::IDENTIFIER(s) => assert_eq!(s, "myBox"),
            _ => panic!("Expected IDENTIFIER token"),
        }
        match &tokens[2].token_type {
            TokenType::IDENTIFIER(s) => assert_eq!(s, "getValue"),
            _ => panic!("Expected IDENTIFIER token"),
        }
    }
    
    #[test]
    fn test_operators() {
        let mut tokenizer = NyashTokenizer::new(">> == != <= >= < >");
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::ARROW);
        assert_eq!(tokens[1].token_type, TokenType::EQUALS);
        assert_eq!(tokens[2].token_type, TokenType::NotEquals);
        assert_eq!(tokens[3].token_type, TokenType::LessEquals);
        assert_eq!(tokens[4].token_type, TokenType::GreaterEquals);
        assert_eq!(tokens[5].token_type, TokenType::LESS);
        assert_eq!(tokens[6].token_type, TokenType::GREATER);
    }
    
    #[test]
    fn test_complex_code() {
        let code = r#"
        box TestBox {
            value
            
            getValue() {
                return this.value
            }
        }
        
        obj = new TestBox()
        obj.value = "test123"
        "#;
        
        let mut tokenizer = NyashTokenizer::new(code);
        let tokens = tokenizer.tokenize().unwrap();
        
        // 基本的なトークンがある事を確認
        let token_types: Vec<_> = tokens.iter().map(|t| &t.token_type).collect();
        assert!(token_types.contains(&&TokenType::BOX));
        assert!(token_types.contains(&&TokenType::NEW));
        assert!(token_types.contains(&&TokenType::THIS));
        assert!(token_types.contains(&&TokenType::RETURN));
        assert!(token_types.contains(&&TokenType::DOT));
    }
    
    #[test]
    fn test_line_numbers() {
        let code = "box\ntest\nvalue";
        let mut tokenizer = NyashTokenizer::new(code);
        let tokens = tokenizer.tokenize().unwrap();
        
        assert_eq!(tokens[0].line, 1); // box
        assert_eq!(tokens[1].line, 2); // test  
        assert_eq!(tokens[2].line, 3); // value
    }
    
    #[test]
    fn test_comments() {
        let code = r#"box Test // this is a comment
# this is also a comment
value"#;
        
        let mut tokenizer = NyashTokenizer::new(code);
        let tokens = tokenizer.tokenize().unwrap();
        
        // コメントは除外されている
        let token_types: Vec<_> = tokens.iter().map(|t| &t.token_type).collect();
        assert_eq!(token_types.len(), 4); // box, Test, value, EOF
    }
    
    #[test]
    fn test_error_handling() {
        let mut tokenizer = NyashTokenizer::new("@#$%");
        let result = tokenizer.tokenize();
        
        assert!(result.is_err());
        match result {
            Err(TokenizeError::UnexpectedCharacter { char, line, column }) => {
                assert_eq!(char, '@');
                assert_eq!(line, 1);
                assert_eq!(column, 1);
            }
            _ => panic!("Expected UnexpectedCharacter error"),
        }
    }
}