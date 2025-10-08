/*!
 * Statement Parser Module Organization
 *
 * Refactored from monolithic statements.rs (723 lines)
 * Split into focused modules following Single Responsibility Principle
 */

// Helper functions
pub mod helpers;

// Control flow statements
pub mod control_flow;

// Declaration statements
pub mod declarations;

// Variable declarations and assignments
pub mod variables;

// I/O and async statements
pub mod io_async;

// Exception handling
pub mod exceptions;

// Module system
pub mod modules;

use crate::ast::{ASTNode, CatchClause, Span};
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;

impl NyashParser {
    /// Parse a standalone block `{ ... }` and optional postfix `catch/cleanup` sequence
    pub(super) fn parse_standalone_block_statement(&mut self) -> Result<ASTNode, ParseError> {
        // Parse the block body first
        let try_body = self.parse_block_statements()?;

        if crate::config::env::block_postfix_catch()
            && (self.match_token(&TokenType::CATCH) || self.match_token(&TokenType::CLEANUP))
        {
            // Parse at most one catch, then optional cleanup
            let mut catch_clauses: Vec<CatchClause> = Vec::new();
            if self.match_token(&TokenType::CATCH) {
                self.advance(); // consume 'catch'
                self.consume(TokenType::LPAREN)?;
                let (exception_type, exception_var) = self.parse_catch_param()?;
                self.consume(TokenType::RPAREN)?;
                let catch_body = self.parse_block_statements()?;

                catch_clauses.push(CatchClause {
                    exception_type,
                    variable_name: exception_var,
                    body: catch_body,
                    span: Span::unknown(),
                });
            }

            // Optional cleanup
            let finally_body = if self.match_token(&TokenType::CLEANUP) {
                self.advance(); // consume 'cleanup'
                Some(self.parse_block_statements()?)
            } else {
                None
            };

            // Return TryCatch with the standalone block as try_body
            Ok(ASTNode::TryCatch {
                try_body,
                catch_clauses,
                finally_body,
                span: Span::unknown(),
            })
        } else {
            // No postfix catch/cleanup - return as Program
            Ok(ASTNode::Program {
                statements: try_body,
                span: Span::unknown(),
            })
        }
    }

    /// Parse block statements: { statement* }
    pub(super) fn parse_block_statements(&mut self) -> Result<Vec<ASTNode>, ParseError> {
        let trace_blocks = std::env::var("NYASH_PARSER_TRACE_BLOCKS").ok().as_deref() == Some("1");
        if trace_blocks {
            eprintln!(
                "[parser][block] enter '{{' at line {}",
                self.current_token().line
            );
        }
        self.consume(TokenType::LBRACE)?;
        let mut statements = Vec::new();

        // Be tolerant to blank lines within blocks: skip NEWLINE tokens between statements
        while !self.is_at_end() {
            while self.match_token(&TokenType::NEWLINE) { self.advance(); }
            if self.match_token(&TokenType::RBRACE) { break; }
            statements.push(self.parse_statement()?);
        }
        if trace_blocks {
            eprintln!(
                "[parser][block] exit '}}' at line {}",
                self.current_token().line
            );
        }
        self.consume(TokenType::RBRACE)?;
        Ok(statements)
    }

    /// Parse method body statements: { statement* }
    /// Optional seam-guard (env-gated via NYASH_PARSER_METHOD_BODY_STRICT=1) is applied
    /// conservatively at top-level only, and only right after a nested block '}' was
    /// just consumed, to avoid false positives inside method bodies.
    pub(super) fn parse_method_body_statements(&mut self) -> Result<Vec<ASTNode>, ParseError> {
        // Reuse block entry tracing
        let trace_blocks = std::env::var("NYASH_PARSER_TRACE_BLOCKS").ok().as_deref() == Some("1");
        if trace_blocks {
            eprintln!(
                "[parser][block] enter '{{' (method) at line {}",
                self.current_token().line
            );
        }
        self.consume(TokenType::LBRACE)?;
        let mut statements = Vec::new();

        // Helper: lookahead for `ident '(' ... ')' [NEWLINE*] '{'`
        let looks_like_method_head = |this: &Self| -> bool {
            // Only meaningful when starting at a new statement head
            match &this.current_token().token_type {
                TokenType::IDENTIFIER(_) => {
                    // Expect '(' after optional NEWLINE
                    let mut k = 1usize;
                    while matches!(this.peek_nth_token(k), TokenType::NEWLINE) { k += 1; }
                    if !matches!(this.peek_nth_token(k), TokenType::LPAREN) { return false; }
                    // Walk to matching ')'
                    k += 1; // after '('
                    let mut depth: i32 = 1;
                    while !matches!(this.peek_nth_token(k), TokenType::EOF) {
                        match this.peek_nth_token(k) {
                            TokenType::LPAREN => depth += 1,
                            TokenType::RPAREN => { depth -= 1; if depth == 0 { k += 1; break; } },
                            _ => {}
                        }
                        k += 1;
                    }
                    // Allow NEWLINE(s) between ')' and '{'
                    while matches!(this.peek_nth_token(k), TokenType::NEWLINE) { k += 1; }
                    matches!(this.peek_nth_token(k), TokenType::LBRACE)
                }
                _ => false,
            }
        };

        while !self.is_at_end() {
            // Skip blank lines at method body top-level
            while self.match_token(&TokenType::NEWLINE) { self.advance(); }
            // Stop at end of current method body
            if self.match_token(&TokenType::RBRACE) { break; }
            // Optional seam guard: if the upcoming tokens form a method head
            // like `ident '(' ... ')' NEWLINE* '{'`, bail out so the caller
            // (static box member parser) can handle it as a declaration, not
            // as a function call expression inside this body.
            if std::env::var("NYASH_PARSER_METHOD_BODY_STRICT").ok().as_deref() == Some("1") {
                if looks_like_method_head(self) {
                    break;
                }
            }
            statements.push(self.parse_statement()?);
        }
        if trace_blocks {
            eprintln!(
                "[parser][block] exit '}}' (method) at line {}",
                self.current_token().line
            );
        }
        self.consume(TokenType::RBRACE)?;
        Ok(statements)
    }

    /// Main statement parser dispatch
    pub(super) fn parse_statement(&mut self) -> Result<ASTNode, ParseError> {
        // For grammar diff: capture starting token to classify statement keyword
        let start_tok = self.current_token().token_type.clone();

        let result = match &start_tok {
            TokenType::LBRACE => self.parse_standalone_block_statement(),

            // Declarations
            TokenType::BOX
            | TokenType::IMPORT
            | TokenType::INTERFACE
            | TokenType::GLOBAL
            | TokenType::FUNCTION
            | TokenType::STATIC
            | TokenType::AT => self.parse_declaration_statement(),

            // Control flow
            TokenType::IF
            | TokenType::LOOP
            | TokenType::BREAK
            | TokenType::CONTINUE
            | TokenType::RETURN => self.parse_control_flow_statement(),

            // I/O and async
            TokenType::PRINT | TokenType::NOWAIT => self.parse_io_module_statement(),

            // Variables
            TokenType::LOCAL | TokenType::OUTBOX => self.parse_variable_declaration_statement(),

            // Exceptions
            TokenType::TRY | TokenType::THROW => self.parse_exception_statement(),
            TokenType::CATCH | TokenType::CLEANUP => self.parse_postfix_catch_cleanup_error(),

            // Module system
            TokenType::USING => self.parse_using(),
            TokenType::FROM => self.parse_from_call_statement(),

            // Flow declaration (staged) — treat 'flow Name { ... }' as a declaration when enabled
            TokenType::IDENTIFIER(s) if s == "flow" && crate::config::env::parser_flow_enabled() => {
                self.parse_flow_declaration()
            }
            // Assignment or function call
            TokenType::IDENTIFIER(_) | TokenType::THIS | TokenType::ME => {
                self.parse_assignment_or_function_call()
            }

            // Fallback: expression statement
            _ => {
                // Thin-adapt with Cursor in stmt mode to normalize leading newlines
                self.with_stmt_cursor(|p| Ok(p.parse_expression()?))
            }
        };

        // Non-invasive syntax rule check
        if std::env::var("NYASH_GRAMMAR_DIFF").ok().as_deref() == Some("1") {
            if let Some(k) = Self::grammar_keyword_for(&start_tok) {
                let ok = crate::grammar::engine::get().syntax_is_allowed_statement(k);
                if !ok {
                    eprintln!(
                        "[GRAMMAR-DIFF][Parser] statement '{}' not allowed by syntax rules",
                        k
                    );
                }
            }
        }

        result
    }
}
