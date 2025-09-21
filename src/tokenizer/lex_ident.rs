use super::{NyashTokenizer, TokenType};
use crate::grammar::engine;

impl NyashTokenizer {
    /// キーワードまたは識別子を読み取り
    pub(crate) fn read_keyword_or_identifier(&mut self) -> TokenType {
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
        let mut tok = match identifier.as_str() {
            "box" => TokenType::BOX,
            "global" => TokenType::GLOBAL,
            "singleton" => TokenType::SINGLETON,
            "new" => TokenType::NEW,
            "match" => TokenType::MATCH,
            "if" => TokenType::IF,
            "else" => TokenType::ELSE,
            "loop" => TokenType::LOOP,
            "break" => TokenType::BREAK,
            "continue" => TokenType::CONTINUE,
            "return" => TokenType::RETURN,
            "function" => TokenType::FUNCTION,
            "fn" => TokenType::FN,
            "print" => TokenType::PRINT,
            "this" => TokenType::THIS,
            "me" => TokenType::ME,
            "init" => TokenType::INIT,
            "pack" => TokenType::PACK,
            "birth" => TokenType::BIRTH,
            "nowait" => TokenType::NOWAIT,
            "await" => TokenType::AWAIT,
            "interface" => TokenType::INTERFACE,
            "include" => TokenType::INCLUDE,
            "import" => TokenType::IMPORT,
            "try" => TokenType::TRY,
            "catch" => TokenType::CATCH,
            "cleanup" => TokenType::CLEANUP,
            "throw" => TokenType::THROW,
            "local" => TokenType::LOCAL,
            "static" => TokenType::STATIC,
            "outbox" => TokenType::OUTBOX,
            "not" => TokenType::NOT,
            "override" => TokenType::OVERRIDE,
            "from" => TokenType::FROM,
            "weak" => TokenType::WEAK,
            "using" => TokenType::USING,
            "and" => TokenType::AND,
            "or" => TokenType::OR,
            "true" => TokenType::TRUE,
            "false" => TokenType::FALSE,
            "null" => TokenType::NULL,
            _ => TokenType::IDENTIFIER(identifier.clone()),
        };

        // 12.7 Strict mode: fallback extended keywords to IDENTIFIER
        if Self::strict_12_7() {
            let is_extended = matches!(
                tok,
                TokenType::INTERFACE
                    | TokenType::USING
                    | TokenType::INCLUDE
                    | TokenType::OUTBOX
                    | TokenType::NOWAIT
                    | TokenType::OVERRIDE
                    | TokenType::WEAK
                    | TokenType::PACK
            );
            if is_extended {
                tok = TokenType::IDENTIFIER(identifier.clone());
            }
        }

        // 統一文法エンジンとの差分チェック（動作は変更しない）
        if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
            if let Some(kw) = engine::get().is_keyword_str(&identifier) {
                if let TokenType::IDENTIFIER(_) = tok {
                    eprintln!(
                        "[GRAMMAR-DIFF] tokenizer=IDENT, grammar=KEYWORD({}) word='{}'",
                        kw, identifier
                    );
                }
            } else if !matches!(tok, TokenType::IDENTIFIER(_)) {
                eprintln!(
                    "[GRAMMAR-DIFF] tokenizer=KEYWORD, grammar=IDENT word='{}'",
                    identifier
                );
            }
        }

        tok
    }
}

