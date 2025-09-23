/*!
 * Parser Enhanced - 既存パーサーの改行処理自動化
 *
 * 既存のNyashParserを拡張し、advance()で自動的に改行をスキップ
 * skip_newlines()の明示的呼び出しを不要にする
 */

use crate::tokenizer::{Token, TokenType};
use std::cell::Cell;

/// パーサーコンテキスト（改行処理のモード管理）
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ParserContext {
    /// 文コンテキスト（改行は文の区切り）
    Statement,
    /// 式コンテキスト（改行を自動スキップ）
    Expression,
    /// ブロック内（改行を自動スキップ）
    Block,
}

thread_local! {
    /// 現在のパーサーコンテキスト
    static PARSER_CONTEXT: Cell<ParserContext> = Cell::new(ParserContext::Statement);

    /// 括弧の深度（自動改行スキップの判定用）
    static PAREN_DEPTH: Cell<usize> = Cell::new(0);
    static BRACE_DEPTH: Cell<usize> = Cell::new(0);
    static BRACKET_DEPTH: Cell<usize> = Cell::new(0);
}

/// コンテキストガード（RAIIパターンで自動復元）
pub struct ContextGuard {
    prev_context: ParserContext,
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        PARSER_CONTEXT.with(|c| c.set(self.prev_context));
    }
}

/// 式コンテキストで実行
pub fn with_expr_context<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    let prev = PARSER_CONTEXT.with(|c| c.get());
    PARSER_CONTEXT.with(|c| c.set(ParserContext::Expression));
    let result = f();
    PARSER_CONTEXT.with(|c| c.set(prev));
    result
}

/// ブロックコンテキストで実行
pub fn with_block_context<F, T>(f: F) -> T
where
    F: FnOnce() -> T,
{
    let prev = PARSER_CONTEXT.with(|c| c.get());
    PARSER_CONTEXT.with(|c| c.set(ParserContext::Block));
    let result = f();
    PARSER_CONTEXT.with(|c| c.set(prev));
    result
}

/// 改行をスキップすべきか判定
pub fn should_skip_newlines() -> bool {
    // 括弧内では常にスキップ
    if PAREN_DEPTH.with(|d| d.get()) > 0
        || BRACE_DEPTH.with(|d| d.get()) > 0
        || BRACKET_DEPTH.with(|d| d.get()) > 0
    {
        return true;
    }

    // コンテキストによる判定
    match PARSER_CONTEXT.with(|c| c.get()) {
        ParserContext::Expression | ParserContext::Block => true,
        ParserContext::Statement => false,
    }
}

/// トークンタイプによる深度更新
pub fn update_depth(token_type: &TokenType, advancing: bool) {
    match token_type {
        TokenType::LPAREN => {
            if advancing {
                PAREN_DEPTH.with(|d| d.set(d.get() + 1));
            }
        }
        TokenType::RPAREN => {
            if !advancing {
                PAREN_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
            }
        }
        TokenType::LBRACE => {
            if advancing {
                BRACE_DEPTH.with(|d| d.set(d.get() + 1));
            }
        }
        TokenType::RBRACE => {
            if !advancing {
                BRACE_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
            }
        }
        TokenType::LBRACK => {
            if advancing {
                BRACKET_DEPTH.with(|d| d.set(d.get() + 1));
            }
        }
        TokenType::RBRACK => {
            if !advancing {
                BRACKET_DEPTH.with(|d| d.set(d.get().saturating_sub(1)));
            }
        }
        _ => {}
    }
}

/// 改良されたadvance実装（自動改行スキップ付き）
pub fn smart_advance(
    tokens: &[Token],
    current: &mut usize,
    prev_token: Option<&TokenType>,
) {
    if *current >= tokens.len() {
        return;
    }

    // 現在のトークンで深度を更新
    let current_token = &tokens[*current].token_type;
    update_depth(current_token, true);

    // 位置を進める
    *current += 1;

    // 改行を自動的にスキップ
    while *current < tokens.len() {
        let token_type = &tokens[*current].token_type;

        // 改行判定
        if matches!(token_type, TokenType::NEWLINE) {
            // スキップすべきか判定
            if should_skip_newlines() || is_line_continuation(prev_token) {
                *current += 1;
                continue;
            }
        }

        // セミコロンも同様に処理
        if matches!(token_type, TokenType::SEMICOLON) {
            if std::env::var("NYASH_PARSER_ALLOW_SEMICOLON").ok().as_deref() == Some("1") {
                if should_skip_newlines() {
                    *current += 1;
                    continue;
                }
            }
        }

        break;
    }
}

/// 行継続判定（直前のトークンから判断）
fn is_line_continuation(prev_token: Option<&TokenType>) -> bool {
    match prev_token {
        Some(token) => matches!(
            token,
            TokenType::PLUS
                | TokenType::MINUS
                | TokenType::MULTIPLY
                | TokenType::DIVIDE
                | TokenType::MODULO
                | TokenType::AND
                | TokenType::OR
                | TokenType::DOT
                | TokenType::DoubleColon
                | TokenType::COMMA
                | TokenType::FatArrow
        ),
        None => false,
    }
}

/// 既存のParserUtilsトレイトを拡張
pub trait EnhancedParserUtils {
    /// 改良版advance（改行自動処理）
    fn advance_smart(&mut self);

    /// 式コンテキストでパース
    fn parse_in_expr_context<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T;

    /// ブロックコンテキストでパース
    fn parse_in_block_context<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T;
}