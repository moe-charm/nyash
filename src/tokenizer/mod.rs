/*!
 * Nyash Tokenizer — split modules (kinds/cursor/whitespace/lexers/engine)
 */

mod kinds;
mod cursor;
mod whitespace;
mod lex_string;
mod lex_number;
mod lex_ident;
mod engine;

pub use kinds::{Token, TokenType, TokenizeError};

/// Nyashトークナイザー
pub struct NyashTokenizer {
    pub(crate) input: Vec<char>,
    pub(crate) position: usize,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

// Public API and core logic are implemented in submodules via impl NyashTokenizer

