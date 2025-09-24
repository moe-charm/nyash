use nyash_rust::tokenizer::{NyashTokenizer, Token, TokenType, TokenizeError};

#[test]
fn test_basic_tokens() {
    let mut tokenizer = NyashTokenizer::new("box new = + - *");
    let tokens = tokenizer.tokenize().unwrap();
    assert!(matches!(tokens[0].token_type, TokenType::BOX));
    assert!(matches!(tokens[1].token_type, TokenType::NEW));
    assert!(matches!(tokens[2].token_type, TokenType::ASSIGN));
    assert!(matches!(tokens[3].token_type, TokenType::PLUS));
    assert!(matches!(tokens[4].token_type, TokenType::MINUS));
    assert!(matches!(tokens[5].token_type, TokenType::MULTIPLY));
}

#[test]
fn test_string_literal() {
    let mut tokenizer = NyashTokenizer::new(r#""Hello, World!""#);
    let tokens = tokenizer.tokenize().unwrap();
    assert!(matches!(tokens[0].token_type, TokenType::STRING(_)));
}

#[test]
fn test_numeric_literals() {
    let mut tokenizer = NyashTokenizer::new("42 123 0");
    let tokens = tokenizer.tokenize().unwrap();
    assert!(matches!(tokens[0].token_type, TokenType::NUMBER(42)));
    assert!(matches!(tokens[1].token_type, TokenType::NUMBER(123)));
    assert!(matches!(tokens[2].token_type, TokenType::NUMBER(0)));
}

#[test]
fn test_identifiers_and_keywords() {
    let mut tokenizer = NyashTokenizer::new("test_var myBox getValue");
    let tokens = tokenizer.tokenize().unwrap();
    assert!(matches!(tokens[0].token_type, TokenType::IDENTIFIER(_)));
    assert!(matches!(tokens[1].token_type, TokenType::IDENTIFIER(_)));
    assert!(matches!(tokens[2].token_type, TokenType::IDENTIFIER(_)));
}

#[test]
fn test_operators() {
    let mut tokenizer = NyashTokenizer::new(">> == != <= >= < >");
    let tokens = tokenizer.tokenize().unwrap();
    assert_eq!(tokens[0].token_type, TokenType::ShiftRight);
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

    // NEWLINEトークンを除外して確認
    let non_newline: Vec<&Token> = tokens
        .iter()
        .filter(|t| !matches!(t.token_type, TokenType::NEWLINE))
        .collect();
    assert_eq!(non_newline[0].line, 1); // box
    assert_eq!(non_newline[1].line, 2); // test
    assert_eq!(non_newline[2].line, 3); // value
}

#[test]
fn test_comments() {
    let code = r#"box Test // this is a comment
# this is also a comment
value"#;

    let mut tokenizer = NyashTokenizer::new(code);
    let tokens = tokenizer.tokenize().unwrap();

    // コメントは除外されている
    let token_types: Vec<_> = tokens
        .iter()
        .filter(|t| !matches!(t.token_type, TokenType::NEWLINE))
        .map(|t| &t.token_type)
        .collect();
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

#[test]
fn test_basic_sugar_tokens() {
    let mut t2 = NyashTokenizer::new("|> ?.? ?? += -= *= /= ..");
    let toks = t2.tokenize().unwrap();
    assert!(toks
        .iter()
        .any(|k| matches!(k.token_type, TokenType::PipeForward)));
    assert!(toks
        .iter()
        .any(|k| matches!(k.token_type, TokenType::QmarkDot)));
    assert!(toks
        .iter()
        .any(|k| matches!(k.token_type, TokenType::QmarkQmark)));
    assert!(toks
        .iter()
        .any(|k| matches!(k.token_type, TokenType::PlusAssign)));
    assert!(toks
        .iter()
        .any(|k| matches!(k.token_type, TokenType::MinusAssign)));
    assert!(toks
        .iter()
        .any(|k| matches!(k.token_type, TokenType::MulAssign)));
    assert!(toks
        .iter()
        .any(|k| matches!(k.token_type, TokenType::DivAssign)));
    assert!(toks
        .iter()
        .any(|k| matches!(k.token_type, TokenType::RANGE)));
}

#[test]
fn test_longest_match_sequences() {
    // '??' は '?' より優先、'?.' は '.' より優先、'..' は '.' より優先
    let mut t = NyashTokenizer::new("?? ? ?. .. .");
    let toks = t.tokenize().unwrap();
    let kinds: Vec<&TokenType> = toks.iter().map(|k| &k.token_type).collect();
    assert!(matches!(kinds[0], TokenType::QmarkQmark));
    assert!(matches!(kinds[1], TokenType::QUESTION));
    assert!(matches!(kinds[2], TokenType::QmarkDot));
    assert!(matches!(kinds[3], TokenType::RANGE));
    assert!(matches!(kinds[4], TokenType::DOT));
}
