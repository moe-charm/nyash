# Phase 16 Implementation Guide: Box-Based Macro System

Date: 2025-09-19  
Version: 0.1.0  
Status: **READY** - 実装開始準備完了  

## 🚀 **実装開始チェックリスト**

### **前提条件確認**
- [x] CLI統合完了（--expand, --run-tests既に実装済み）
- [x] 環境変数対応（NYASH_MACRO_ENABLE等）
- [x] AST基盤（既存のASTNode構造体）
- [x] パーサー基盤（NyashParser実装済み）
- [ ] **開始準備**: Phase 1実装開始

## 🎯 **Phase 1: AST Pattern Matching 実装**

### **Step 1.1: AST構造体拡張**

#### **ファイル**: `src/ast.rs`
```rust
// 既存のASTNodeに追加
#[derive(Debug, Clone)]
pub enum ASTNode {
    // ... 既存のVariant ...
    
    // 新規追加: パターンマッチング
    Match {
        target: Box<ASTNode>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    
    // パターン表現
    Pattern(PatternAst),
}

// 新規追加: パターンAST
#[derive(Debug, Clone)]
pub enum PatternAst {
    // 基本パターン
    Wildcard { span: Span },
    Identifier { name: String, span: Span },
    Literal { value: LiteralValue, span: Span },
    
    // 構造パターン
    BoxPattern {
        name: String,
        fields: Vec<FieldPattern>,
        rest: Option<String>,  // ..rest
        span: Span,
    },
    
    // 配列パターン
    ArrayPattern {
        elements: Vec<PatternAst>,
        rest: Option<String>,  // ...rest
        span: Span,
    },
    
    // OR パターン
    OrPattern {
        patterns: Vec<PatternAst>,
        span: Span,
    },
    
    // バインドパターン
    BindPattern {
        name: String,           // @variable
        pattern: Box<PatternAst>,
        span: Span,
    },
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: PatternAst,
    pub guard: Option<Box<ASTNode>>,
    pub body: Vec<ASTNode>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FieldPattern {
    pub name: String,
    pub pattern: PatternAst,
    pub span: Span,
}
```

**実装タスク**:
```bash
# 1. ast.rsに上記の定義を追加
# 2. 既存のコンパイルエラーを修正
# 3. 基本的なDebug traitの動作確認
```

### **Step 1.2: Tokenizer拡張**

#### **ファイル**: `src/tokenizer.rs`
```rust
// TokenTypeに追加
#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // ... 既存のToken ...
    
    // パターンマッチング用
    MATCH,              // match
    PIPE,               // |
    AT,                 // @
    DOTDOT,             // ..
    DOTDOTDOT,          // ...
}

// Tokenizerに追加
impl Tokenizer {
    fn read_word(&mut self) -> TokenType {
        match word.as_str() {
            // ... 既存のキーワード ...
            "match" => TokenType::MATCH,
            _ => TokenType::IDENTIFIER(word),
        }
    }
    
    fn read_symbol(&mut self) -> TokenType {
        match self.current_char() {
            // ... 既存のシンボル ...
            '|' => TokenType::PIPE,
            '@' => TokenType::AT,
            '.' => {
                if self.peek_char() == Some('.') {
                    self.advance(); // consume second '.'
                    if self.peek_char() == Some('.') {
                        self.advance(); // consume third '.'
                        TokenType::DOTDOTDOT
                    } else {
                        TokenType::DOTDOT
                    }
                } else {
                    TokenType::DOT
                }
            }
            _ => // ... 既存の処理 ...
        }
    }
}
```

**実装タスク**:
```bash
# 1. TokenTypeに新しいトークンを追加  
# 2. Tokenizerの辞書登録
# 3. 基本的なトークン化テスト
```

### **Step 1.3: Parser拡張（match式）**

#### **ファイル**: `src/parser/expressions.rs` (新規作成)
```rust
use crate::ast::{ASTNode, PatternAst, MatchArm, FieldPattern};
use crate::parser::{NyashParser, ParseError};
use crate::tokenizer::TokenType;

impl NyashParser {
    /// match式のパース
    pub fn parse_match_expression(&mut self) -> Result<ASTNode, ParseError> {
        let start_line = self.current_token().line;
        self.consume(TokenType::MATCH)?;
        
        // match対象の式
        let target = Box::new(self.parse_expression()?);
        
        self.consume(TokenType::LBRACE)?;
        self.skip_newlines();
        
        // match arms
        let mut arms = Vec::new();
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            arms.push(self.parse_match_arm()?);
            self.skip_newlines();
        }
        
        if arms.is_empty() {
            return Err(ParseError::UnexpectedToken {
                expected: "at least one match arm".to_string(),
                found: self.current_token().token_type.clone(),
                line: start_line,
            });
        }
        
        self.consume(TokenType::RBRACE)?;
        
        Ok(ASTNode::Match {
            target,
            arms,
            span: crate::ast::Span::unknown(),
        })
    }
    
    /// マッチアームのパース
    fn parse_match_arm(&mut self) -> Result<MatchArm, ParseError> {
        // パターン
        let pattern = self.parse_pattern()?;
        
        // ガード（if condition）
        let guard = if self.match_token(&TokenType::IF) {
            self.advance(); // consume 'if'
            Some(Box::new(self.parse_expression()?))
        } else {
            None
        };
        
        self.consume(TokenType::ARROW)?; // =>
        
        // ボディ
        let body = if self.match_token(&TokenType::LBRACE) {
            self.parse_block_statements()?
        } else {
            vec![self.parse_statement()?]
        };
        
        Ok(MatchArm {
            pattern,
            guard,
            body,
            span: crate::ast::Span::unknown(),
        })
    }
    
    /// パターンのパース
    pub fn parse_pattern(&mut self) -> Result<PatternAst, ParseError> {
        self.parse_or_pattern()
    }
    
    /// ORパターン（最低優先度）
    fn parse_or_pattern(&mut self) -> Result<PatternAst, ParseError> {
        let mut pattern = self.parse_bind_pattern()?;
        
        if self.match_token(&TokenType::PIPE) {
            let mut patterns = vec![pattern];
            
            while self.match_token(&TokenType::PIPE) {
                self.advance(); // consume '|'
                patterns.push(self.parse_bind_pattern()?);
            }
            
            pattern = PatternAst::OrPattern {
                patterns,
                span: crate::ast::Span::unknown(),
            };
        }
        
        Ok(pattern)
    }
    
    /// バインドパターン（@variable pattern）
    fn parse_bind_pattern(&mut self) -> Result<PatternAst, ParseError> {
        if self.match_token(&TokenType::AT) {
            self.advance(); // consume '@'
            
            let name = if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "identifier after '@'".to_string(),
                    found: self.current_token().token_type.clone(),
                    line: self.current_token().line,
                });
            };
            
            let pattern = Box::new(self.parse_primary_pattern()?);
            
            Ok(PatternAst::BindPattern {
                name,
                pattern,
                span: crate::ast::Span::unknown(),
            })
        } else {
            self.parse_primary_pattern()
        }
    }
    
    /// 基本パターン
    fn parse_primary_pattern(&mut self) -> Result<PatternAst, ParseError> {
        match &self.current_token().token_type {
            // ワイルドカード
            TokenType::UNDERSCORE => {
                self.advance();
                Ok(PatternAst::Wildcard {
                    span: crate::ast::Span::unknown(),
                })
            }
            
            // 識別子（変数バインドまたは構造体パターン）
            TokenType::IDENTIFIER(name) => {
                let name = name.clone();
                self.advance();
                
                if self.match_token(&TokenType::LBRACE) {
                    // 構造パターン: TypeName { field1, field2, .. }
                    self.parse_box_pattern(name)
                } else {
                    // 変数バインド
                    Ok(PatternAst::Identifier {
                        name,
                        span: crate::ast::Span::unknown(),
                    })
                }
            }
            
            // リテラル
            TokenType::INTEGER(value) => {
                let value = *value;
                self.advance();
                Ok(PatternAst::Literal {
                    value: crate::ast::LiteralValue::Integer(value),
                    span: crate::ast::Span::unknown(),
                })
            }
            
            TokenType::STRING(value) => {
                let value = value.clone();
                self.advance();
                Ok(PatternAst::Literal {
                    value: crate::ast::LiteralValue::String(value),
                    span: crate::ast::Span::unknown(),
                })
            }
            
            // 配列パターン
            TokenType::LBRACKET => self.parse_array_pattern(),
            
            _ => Err(ParseError::UnexpectedToken {
                expected: "pattern".to_string(),
                found: self.current_token().token_type.clone(),
                line: self.current_token().line,
            }),
        }
    }
    
    /// Box構造パターン: TypeName { field1, field2, .. }
    fn parse_box_pattern(&mut self, type_name: String) -> Result<PatternAst, ParseError> {
        self.consume(TokenType::LBRACE)?;
        self.skip_newlines();
        
        let mut fields = Vec::new();
        let mut rest = None;
        
        while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
            // rest pattern: ..rest
            if self.match_token(&TokenType::DOTDOT) {
                self.advance(); // consume '..'
                
                if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
                    rest = Some(name.clone());
                    self.advance();
                } else {
                    return Err(ParseError::UnexpectedToken {
                        expected: "identifier after '..'".to_string(),
                        found: self.current_token().token_type.clone(),
                        line: self.current_token().line,
                    });
                }
                break; // rest patternは最後でなければならない
            }
            
            // フィールドパターン
            let field_name = if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
                let name = name.clone();
                self.advance();
                name
            } else {
                return Err(ParseError::UnexpectedToken {
                    expected: "field name".to_string(),
                    found: self.current_token().token_type.clone(),
                    line: self.current_token().line,
                });
            };
            
            let pattern = if self.match_token(&TokenType::COLON) {
                self.advance(); // consume ':'
                self.parse_pattern()?
            } else {
                // 短縮形: field は field: field と同じ
                PatternAst::Identifier {
                    name: field_name.clone(),
                    span: crate::ast::Span::unknown(),
                }
            };
            
            fields.push(FieldPattern {
                name: field_name,
                pattern,
                span: crate::ast::Span::unknown(),
            });
            
            if self.match_token(&TokenType::COMMA) {
                self.advance();
                self.skip_newlines();
            } else if !self.match_token(&TokenType::RBRACE) {
                return Err(ParseError::UnexpectedToken {
                    expected: "',' or '}'".to_string(),
                    found: self.current_token().token_type.clone(),
                    line: self.current_token().line,
                });
            }
        }
        
        self.consume(TokenType::RBRACE)?;
        
        Ok(PatternAst::BoxPattern {
            name: type_name,
            fields,
            rest,
            span: crate::ast::Span::unknown(),
        })
    }
    
    /// 配列パターン: [first, second, ...rest]
    fn parse_array_pattern(&mut self) -> Result<PatternAst, ParseError> {
        self.consume(TokenType::LBRACKET)?;
        self.skip_newlines();
        
        let mut elements = Vec::new();
        let mut rest = None;
        
        while !self.match_token(&TokenType::RBRACKET) && !self.is_at_end() {
            // rest pattern: ...rest
            if self.match_token(&TokenType::DOTDOTDOT) {
                self.advance(); // consume '...'
                
                if let TokenType::IDENTIFIER(name) = &self.current_token().token_type {
                    rest = Some(name.clone());
                    self.advance();
                } else {
                    return Err(ParseError::UnexpectedToken {
                        expected: "identifier after '...'".to_string(),
                        found: self.current_token().token_type.clone(),
                        line: self.current_token().line,
                    });
                }
                break; // rest patternは最後でなければならない
            }
            
            elements.push(self.parse_pattern()?);
            
            if self.match_token(&TokenType::COMMA) {
                self.advance();
                self.skip_newlines();
            } else if !self.match_token(&TokenType::RBRACKET) {
                return Err(ParseError::UnexpectedToken {
                    expected: "',' or ']'".to_string(),
                    found: self.current_token().token_type.clone(),
                    line: self.current_token().line,
                });
            }
        }
        
        self.consume(TokenType::RBRACKET)?;
        
        Ok(PatternAst::ArrayPattern {
            elements,
            rest,
            span: crate::ast::Span::unknown(),
        })
    }
}
```

**実装タスク**:
```bash
# 1. src/parser/expressions.rs を作成
# 2. src/parser/mod.rs に追加
# 3. 基本的なmatch式のパーステスト
```

### **Step 1.4: パターンマッチング実行エンジン**

#### **ファイル**: `src/macro_system/pattern_matcher.rs` (新規作成)
```rust
use crate::ast::{ASTNode, PatternAst, FieldPattern, LiteralValue};
use std::collections::HashMap;

#[derive(Debug)]
pub struct PatternMatcher {
    bindings: HashMap<String, ASTNode>,
}

#[derive(Debug)]
pub enum MatchResult {
    Success,
    Failure,
}

impl PatternMatcher {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
        }
    }
    
    /// パターンマッチング実行
    pub fn match_pattern(&mut self, pattern: &PatternAst, value: &ASTNode) -> MatchResult {
        match (pattern, value) {
            // ワイルドカード: 常に成功
            (PatternAst::Wildcard { .. }, _) => MatchResult::Success,
            
            // 識別子: 変数バインド
            (PatternAst::Identifier { name, .. }, _) => {
                self.bindings.insert(name.clone(), value.clone());
                MatchResult::Success
            }
            
            // リテラル: 値比較
            (PatternAst::Literal { value: pattern_val, .. }, 
             ASTNode::Literal { value: node_val, .. }) => {
                if self.literal_equals(pattern_val, node_val) {
                    MatchResult::Success
                } else {
                    MatchResult::Failure
                }
            }
            
            // Box構造パターン
            (PatternAst::BoxPattern { name: pattern_name, fields: pattern_fields, rest, .. },
             ASTNode::BoxDeclaration { name: box_name, fields: box_fields, .. }) => {
                if pattern_name == box_name {
                    self.match_box_fields(pattern_fields, box_fields, rest)
                } else {
                    MatchResult::Failure
                }
            }
            
            // 配列パターン
            (PatternAst::ArrayPattern { elements, rest, .. },
             ASTNode::Array { elements: array_elements, .. }) => {
                self.match_array_elements(elements, array_elements, rest)
            }
            
            // ORパターン: いずれかが成功すれば成功
            (PatternAst::OrPattern { patterns, .. }, value) => {
                for pattern in patterns {
                    let mut temp_matcher = PatternMatcher::new();
                    if let MatchResult::Success = temp_matcher.match_pattern(pattern, value) {
                        // 成功したバインディングをマージ
                        self.bindings.extend(temp_matcher.bindings);
                        return MatchResult::Success;
                    }
                }
                MatchResult::Failure
            }
            
            // バインドパターン: 内部パターンマッチ + 変数バインド
            (PatternAst::BindPattern { name, pattern, .. }, value) => {
                if let MatchResult::Success = self.match_pattern(pattern, value) {
                    self.bindings.insert(name.clone(), value.clone());
                    MatchResult::Success
                } else {
                    MatchResult::Failure
                }
            }
            
            // その他: 失敗
            _ => MatchResult::Failure,
        }
    }
    
    /// リテラル値の比較
    fn literal_equals(&self, a: &LiteralValue, b: &LiteralValue) -> bool {
        match (a, b) {
            (LiteralValue::Integer(a), LiteralValue::Integer(b)) => a == b,
            (LiteralValue::String(a), LiteralValue::String(b)) => a == b,
            (LiteralValue::Bool(a), LiteralValue::Bool(b)) => a == b,
            (LiteralValue::Null, LiteralValue::Null) => true,
            _ => false,
        }
    }
    
    /// Boxフィールドのマッチング
    fn match_box_fields(
        &mut self,
        pattern_fields: &[FieldPattern],
        box_fields: &[String],
        rest: &Option<String>,
    ) -> MatchResult {
        // TODO: 実装
        // 現在は簡単のため、フィールド名のマッチングのみ
        if pattern_fields.len() <= box_fields.len() {
            MatchResult::Success
        } else {
            MatchResult::Failure
        }
    }
    
    /// 配列要素のマッチング
    fn match_array_elements(
        &mut self,
        pattern_elements: &[PatternAst],
        array_elements: &[ASTNode],
        rest: &Option<String>,
    ) -> MatchResult {
        // TODO: 実装
        // 現在は簡単のため、要素数のチェックのみ
        if pattern_elements.len() <= array_elements.len() {
            MatchResult::Success
        } else {
            MatchResult::Failure
        }
    }
    
    /// バインディング取得
    pub fn get_binding(&self, name: &str) -> Option<&ASTNode> {
        self.bindings.get(name)
    }
    
    /// すべてのバインディング取得
    pub fn bindings(&self) -> &HashMap<String, ASTNode> {
        &self.bindings
    }
}
```

**実装タスク**:
```bash
# 1. src/macro_system ディレクトリ作成
# 2. pattern_matcher.rs 作成
# 3. src/lib.rs にmacro_systemモジュール追加
# 4. 基本的なパターンマッチテスト
```

### **Step 1.5: 基本テスト**

#### **ファイル**: `src/tests/pattern_matching_tests.rs` (新規作成)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{ASTNode, PatternAst, LiteralValue};
    use crate::macro_system::pattern_matcher::{PatternMatcher, MatchResult};

    #[test]
    fn test_wildcard_pattern() {
        let mut matcher = PatternMatcher::new();
        let pattern = PatternAst::Wildcard { span: crate::ast::Span::unknown() };
        let value = ASTNode::Literal {
            value: LiteralValue::Integer(42),
            span: crate::ast::Span::unknown(),
        };
        
        let result = matcher.match_pattern(&pattern, &value);
        assert!(matches!(result, MatchResult::Success));
    }
    
    #[test]
    fn test_identifier_pattern() {
        let mut matcher = PatternMatcher::new();
        let pattern = PatternAst::Identifier {
            name: "x".to_string(),
            span: crate::ast::Span::unknown(),
        };
        let value = ASTNode::Literal {
            value: LiteralValue::Integer(42),
            span: crate::ast::Span::unknown(),
        };
        
        let result = matcher.match_pattern(&pattern, &value);
        assert!(matches!(result, MatchResult::Success));
        
        // バインディング確認
        let binding = matcher.get_binding("x");
        assert!(binding.is_some());
    }
    
    #[test]
    fn test_literal_pattern_success() {
        let mut matcher = PatternMatcher::new();
        let pattern = PatternAst::Literal {
            value: LiteralValue::Integer(42),
            span: crate::ast::Span::unknown(),
        };
        let value = ASTNode::Literal {
            value: LiteralValue::Integer(42),
            span: crate::ast::Span::unknown(),
        };
        
        let result = matcher.match_pattern(&pattern, &value);
        assert!(matches!(result, MatchResult::Success));
    }
    
    #[test]
    fn test_literal_pattern_failure() {
        let mut matcher = PatternMatcher::new();
        let pattern = PatternAst::Literal {
            value: LiteralValue::Integer(42),
            span: crate::ast::Span::unknown(),
        };
        let value = ASTNode::Literal {
            value: LiteralValue::Integer(99),
            span: crate::ast::Span::unknown(),
        };
        
        let result = matcher.match_pattern(&pattern, &value);
        assert!(matches!(result, MatchResult::Failure));
    }
}
```

**実装タスク**:
```bash
# 1. テストファイル作成
# 2. cargo test で動作確認
# 3. CI通過確認
```

## 🎯 **Phase 1 完成目標**

### **動作する最小例**
```nyash
// パース可能なmatch式
match some_value {
    42 => print("Found answer")
    x => print("Found: " + x.toString())
    _ => print("Unknown")
}

// パース可能なパターン
BoxDeclaration { name: "Person", fields: [first, ...rest] }
```

### **Phase 1 完了条件**
- [ ] すべてのパターン構文がパース可能
- [ ] 基本的なパターンマッチングが動作
- [ ] テストが緑色（通過）
- [ ] 既存機能に影響なし（回帰テスト通過）

---

**Phase 1が完了したら、Phase 2（Quote/Unquote）の実装に進む！** 🚀

**Next**: [Phase 2実装ガイド](./PHASE2_IMPLEMENTATION.md) | [テスト戦略](./TESTING_STRATEGY.md)