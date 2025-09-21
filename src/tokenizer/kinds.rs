use thiserror::Error;

/// トークンの種類
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // リテラル
    STRING(String),
    NUMBER(i64),
    FLOAT(f64),
    TRUE,
    FALSE,
    NULL,

    // キーワード
    BOX,
    GLOBAL,
    SINGLETON,
    NEW,
    MATCH,
    IF,
    ELSE,
    LOOP,
    BREAK,
    CONTINUE,
    RETURN,
    FUNCTION,
    FN,
    PRINT,
    THIS,
    ME,
    INIT,
    PACK,
    BIRTH,
    NOWAIT,
    AWAIT,
    INTERFACE,
    COLON,
    INCLUDE,
    TRY,
    CATCH,
    CLEANUP,
    THROW,
    LOCAL,
    STATIC,
    OUTBOX,
    NOT,
    OVERRIDE,
    FROM,
    WEAK,
    USING,
    IMPORT,

    // 演算子
    ShiftLeft,
    ShiftRight,
    BitAnd,
    BitOr,
    BitXor,
    FatArrow,
    EQUALS,
    NotEquals,
    LessEquals,
    GreaterEquals,
    AND,
    OR,
    // 2文字演算子（最長一致）
    PipeForward,
    QmarkDot,
    QmarkQmark,
    PlusAssign,
    MinusAssign,
    MulAssign,
    DivAssign,
    RANGE,
    LESS,
    GREATER,
    ASSIGN,
    PLUS,
    MINUS,
    MULTIPLY,
    DIVIDE,
    MODULO,

    // 記号
    DOT,
    DoubleColon,
    LPAREN,
    RPAREN,
    LBRACK,
    RBRACK,
    LBRACE,
    RBRACE,
    COMMA,
    QUESTION,
    NEWLINE,
    SEMICOLON, // オプショナル区切り

    // 識別子
    IDENTIFIER(String),

    // 特殊
    EOF,
}

/// トークン（位置情報付き）
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

