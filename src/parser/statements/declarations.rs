/*!
 * Declaration Statement Parsers
 *
 * Dispatcher for declaration statements
 * Actual implementations are in other specialized modules
 */

use crate::ast::ASTNode;
use crate::parser::{NyashParser, ParseError};
use crate::parser::common::ParserUtils;
use crate::tokenizer::TokenType;

impl NyashParser {
    /// Parse declaration statement dispatch
    pub(super) fn parse_declaration_statement(&mut self) -> Result<ASTNode, ParseError> {
        match &self.current_token().token_type {
            TokenType::BOX => self.parse_box_declaration(),
            TokenType::IMPORT => self.parse_import(),
            TokenType::INTERFACE => self.parse_interface_box_declaration(),
            TokenType::GLOBAL => self.parse_global_var(),
            TokenType::FUNCTION => self.parse_function_declaration(),
            TokenType::STATIC => self.parse_static_declaration(),
            TokenType::AT => {
                // Consume @
                self.advance();
                // Dispatch macro name
                match &self.current_token().token_type {
                    TokenType::IDENTIFIER(s) if s == "enum" => {
                        self.advance(); // consume "enum"
                        self.parse_enum_declaration()
                    }
                    TokenType::IDENTIFIER(s) if s == "derive" => {
                        // Gate: require macros enabled (planned default ON in dev/ci)
                        if std::env::var("NYASH_MACRO_ENABLE").ok().map(|v| v != "0" && v != "false" && v != "off").unwrap_or(false) {
                            self.parse_derive_then_box()
                        } else {
                            Err(ParseError::UnexpectedToken { found: TokenType::IDENTIFIER("derive".into()), expected: "macro feature enabled (NYASH_MACRO_ENABLE=1)".to_string(), line: self.current_token().line })
                        }
                    }
                    _ => Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "macro name (enum|derive)".to_string(),
                        line: self.current_token().line,
                    }),
                }
            }
            TokenType::IDENTIFIER(s) if s == "flow" => {
                if crate::config::env::parser_flow_enabled() {
                    self.parse_flow_declaration()
                } else {
                    Err(ParseError::UnexpectedToken {
                        found: self.current_token().token_type.clone(),
                        expected: "declaration statement".to_string(),
                        line: self.current_token().line,
                    })
                }
            }
            _ => Err(ParseError::UnexpectedToken {
                found: self.current_token().token_type.clone(),
                expected: "declaration statement".to_string(),
                line: self.current_token().line,
            }),
        }
    }
}
