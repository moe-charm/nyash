use crate::syntax::sugar_config::{SugarConfig, SugarLevel};

#[test]
fn sugar_config_env_overrides_toml() {
    use std::{env, fs};
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("nyash.toml");
    fs::write(&file, "[syntax]\nsugar_level='full'\n").unwrap();
    env::set_var("NYASH_SYNTAX_SUGAR_LEVEL", "basic");
    let cfg = SugarConfig::from_env_or_toml(&file);
    env::remove_var("NYASH_SYNTAX_SUGAR_LEVEL");
    assert_eq!(cfg.level, SugarLevel::Basic);
}

#[test]
fn tokenizer_has_basic_sugar_tokens() {
    use crate::tokenizer::{NyashTokenizer, TokenType};
    let mut t = NyashTokenizer::new("|> ?.? ?? += -= *= /= ..");
    let toks = t.tokenize().unwrap();
    let has = |p: fn(&TokenType) -> bool| -> bool { toks.iter().any(|k| p(&k.token_type)) };
    assert!(has(|k| matches!(k, TokenType::PipeForward)));
    assert!(has(|k| matches!(k, TokenType::QmarkDot)));
    assert!(has(|k| matches!(k, TokenType::QmarkQmark)));
    assert!(has(|k| matches!(k, TokenType::PlusAssign)));
    assert!(has(|k| matches!(k, TokenType::MinusAssign)));
    assert!(has(|k| matches!(k, TokenType::MulAssign)));
    assert!(has(|k| matches!(k, TokenType::DivAssign)));
    assert!(has(|k| matches!(k, TokenType::RANGE)));
}
