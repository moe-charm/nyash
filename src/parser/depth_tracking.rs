#![allow(dead_code)]
/*!
 * 深度追跡機能 - Smart advance用
 *
 * 括弧の深度を追跡し、改行の自動スキップを判定
 *
 * LEGACY (Phase 15.5):
 * - 改行/深度の判定は TokenCursor に一元化していく方針。
 * - 互換維持のため当面残置（参照ゼロ後に撤去予定）。
 */

use super::{NyashParser, ParserUtils};
use crate::tokenizer::TokenType;

impl NyashParser {
    /// 現在の括弧深度を取得（デバッグ用）
    #[allow(dead_code)]
    pub fn get_depths(&self) -> (usize, usize, usize) {
        (self.paren_depth, self.brace_depth, self.bracket_depth)
    }

    /// 括弧の深度が0以上か（何かの括弧内にいるか）
    pub fn in_brackets(&self) -> bool {
        self.paren_depth > 0 || self.brace_depth > 0 || self.bracket_depth > 0
    }
}

impl ParserUtils for NyashParser {
    fn tokens(&self) -> &Vec<crate::tokenizer::Token> {
        &self.tokens
    }

    fn current(&self) -> usize {
        self.current
    }

    fn current_mut(&mut self) -> &mut usize {
        &mut self.current
    }

    /// advance前の深度更新（現在のトークンを処理）
    fn update_depth_before_advance(&mut self) {
        if std::env::var("NYASH_DEBUG_DEPTH").ok().as_deref() == Some("1") {
            eprintln!("🔍 BEFORE advance: token={:?}, depths=({},{},{})",
                self.current_token().token_type, self.paren_depth, self.brace_depth, self.bracket_depth);
        }
        // 開き括弧の場合は深度を増やす（進む前に）
        match &self.current_token().token_type {
            TokenType::LPAREN => {
                self.paren_depth += 1;
            }
            TokenType::LBRACE => {
                self.brace_depth += 1;
            }
            TokenType::LBRACK => {
                self.bracket_depth += 1;
            }
            _ => {}
        }
    }

    /// advance後の深度更新（新しいトークンを処理）
    fn update_depth_after_advance(&mut self) {
        if !self.is_at_end() {
            // 閉じ括弧の場合は深度を減らす（進んだ後）
            match &self.current_token().token_type {
                TokenType::RPAREN => {
                    self.paren_depth = self.paren_depth.saturating_sub(1);
                }
                TokenType::RBRACE => {
                    self.brace_depth = self.brace_depth.saturating_sub(1);
                }
                TokenType::RBRACK => {
                    self.bracket_depth = self.bracket_depth.saturating_sub(1);
                }
                _ => {}
            }
            if std::env::var("NYASH_DEBUG_DEPTH").ok().as_deref() == Some("1") {
                eprintln!("🔍 AFTER advance: token={:?}, depths=({},{},{})",
                    self.current_token().token_type, self.paren_depth, self.brace_depth, self.bracket_depth);
            }
        }
    }

    /// 改行を自動スキップすべきか判定（NyashParser版）
    fn should_auto_skip_newlines(&self) -> bool {
        // Smart advanceをデフォルトで有効化（NYASH_SMART_ADVANCE=0で無効化可能）
        if std::env::var("NYASH_SMART_ADVANCE").ok().as_deref() == Some("0") {
            return false;
        }

        // 括弧内では常に改行をスキップ
        if self.in_brackets() {
            return true;
        }

        // 行継続判定
        // 1. 直前のトークンが演算子等の場合
        if self.current() > 0 {
            let prev_token = &self.tokens[self.current() - 1].token_type;
            match prev_token {
                // 演算子の後（行継続）
                TokenType::PLUS | TokenType::MINUS | TokenType::MULTIPLY |
                TokenType::DIVIDE | TokenType::MODULO |
                TokenType::AND | TokenType::OR |
                TokenType::DOT | TokenType::DoubleColon |
                TokenType::COMMA | TokenType::FatArrow |
                TokenType::ASSIGN | TokenType::COLON => return true,
                _ => {}
            }
        }

        // 2. 現在のトークンが改行で、次のトークンが行継続演算子の場合
        if matches!(self.current_token().token_type, TokenType::NEWLINE) {
            if self.current() + 1 < self.tokens.len() {
                let next_token = &self.tokens[self.current() + 1].token_type;
                match next_token {
                    // 次の行が演算子で始まる場合も行継続
                    TokenType::DOT | TokenType::PLUS | TokenType::MINUS |
                    TokenType::MULTIPLY | TokenType::DIVIDE | TokenType::MODULO |
                    TokenType::AND | TokenType::OR | TokenType::DoubleColon => return true,
                    _ => {}
                }
            }
        }

        false
    }
}
