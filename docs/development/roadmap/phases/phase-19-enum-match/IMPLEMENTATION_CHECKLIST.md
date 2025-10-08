# @enum Implementation Checklist (Day 1-3)

**Based on**: `enum-module-architecture.md`
**Timeline**: 3 days core implementation + 1 day buffer
**Status**: Ready to Start

---

## Day 1: Parser + AST (Foundation) ⏰ 8 hours

### Phase 1.1: Setup & Understanding (1 hour)

- [ ] **Read reference files**:
  - [ ] `src/parser/mod.rs` - Main parser structure
  - [ ] `src/parser/declarations/box_definition.rs` - Box parsing patterns
  - [ ] `src/ast.rs` - AST node patterns
  - [ ] `docs/development/roadmap/phases/phase-19-enum-match/README.md` - Project overview

- [ ] **Create branch**:
  ```bash
  git checkout -b feature/enum-day1-parser
  ```

### Phase 1.2: AST Structures (1 hour)

- [ ] **Modify** `src/ast.rs`:
  ```rust
  // ADD these structs (around line 450, near BoxDeclaration):

  /// Enum declaration AST node (parsed from @enum syntax)
  #[derive(Debug, Clone, PartialEq)]
  pub struct EnumDeclaration {
      pub name: String,
      pub variants: Vec<EnumVariant>,
      pub span: Span,
  }

  /// Enum variant (e.g., "Some(value: T)" or "None()")
  #[derive(Debug, Clone, PartialEq)]
  pub struct EnumVariant {
      pub name: String,
      pub fields: Vec<EnumField>,
      pub span: Span,
  }

  /// Enum variant field (e.g., "value: IntegerBox")
  #[derive(Debug, Clone, PartialEq)]
  pub struct EnumField {
      pub name: String,
      pub type_name: String,  // "IntegerBox", "StringBox", "T", etc.
      pub span: Span,
  }
  ```

- [ ] **Add to ASTNode enum** (around line 300):
  ```rust
  pub enum ASTNode {
      // ... existing variants ...

      /// Enum declaration (macro syntax: @enum Name { ... })
      EnumDeclaration {
          name: String,
          variants: Vec<EnumVariant>,
          span: Span,
      },

      // ... rest ...
  }
  ```

- [ ] **Build to verify**:
  ```bash
  cargo build
  ```

### Phase 1.3: Tokenizer Integration (30 minutes)

- [ ] **Check if MacroKeyword exists**:
  ```bash
  grep -n "MacroKeyword" src/tokenizer/mod.rs
  ```

- [ ] **If NOT exists, add to** `src/tokenizer/mod.rs`:
  ```rust
  pub enum TokenType {
      // ... existing tokens ...
      MacroKeyword,  // for @ prefix
      // ...
  }
  ```

- [ ] **Update tokenizer to recognize @**:
  ```rust
  // In tokenize() function:
  '@' => {
      tokens.push(Token::new(TokenType::MacroKeyword, "@", self.line));
  }
  ```

- [ ] **Build to verify**:
  ```bash
  cargo build
  ```

### Phase 1.4: Parser Implementation (4 hours)

- [ ] **Create** `src/parser/declarations/enum_parser.rs`:
  ```rust
  /*!
   * Enum Declaration Parser Module
   *
   * Parses @enum declarations into AST nodes
   * Syntax: @enum Name { Variant(field: Type), ... }
   */

  use crate::ast::{ASTNode, EnumDeclaration, EnumVariant, EnumField, Span};
  use crate::parser::{NyashParser, ParseError};
  use crate::parser::common::ParserUtils;
  use crate::tokenizer::TokenType;

  impl NyashParser {
      /// Parse @enum declaration
      ///
      /// Syntax:
      ///   @enum EnumName {
      ///     VariantA(field1: TypeBox, field2: TypeBox)
      ///     VariantB()
      ///     VariantC(value: IntegerBox)
      ///   }
      pub(crate) fn parse_enum_declaration(&mut self) -> Result<ASTNode, ParseError> {
          // 1. Consume '@' (MacroKeyword)
          self.expect(&TokenType::MacroKeyword)?;

          // 2. Expect 'enum' identifier
          if !self.match_identifier("enum") {
              return Err(ParseError::UnexpectedToken {
                  found: self.current_token().token_type.clone(),
                  expected: "enum".to_string(),
                  line: self.current_token().line,
              });
          }
          self.advance();

          // 3. Parse enum name
          let enum_name = self.expect_identifier()?;

          // 4. Expect '{'
          self.expect(&TokenType::LBRACE)?;

          // 5. Parse variants
          let mut variants = Vec::new();
          while !self.match_token(&TokenType::RBRACE) && !self.is_at_end() {
              // Skip newlines
              if self.match_token(&TokenType::NEWLINE) {
                  self.advance();
                  continue;
              }

              let variant = self.parse_enum_variant()?;
              variants.push(variant);
          }

          // 6. Expect '}'
          self.expect(&TokenType::RBRACE)?;

          // 7. Check for duplicate variant names
          self.check_duplicate_variants(&variants)?;

          Ok(ASTNode::EnumDeclaration {
              name: enum_name,
              variants,
              span: Span::unknown(),
          })
      }

      fn parse_enum_variant(&mut self) -> Result<EnumVariant, ParseError> {
          // 1. Parse variant name
          let variant_name = self.expect_identifier()?;

          // 2. Expect '('
          self.expect(&TokenType::LPAREN)?;

          // 3. Parse fields (comma-separated)
          let mut fields = Vec::new();
          while !self.match_token(&TokenType::RPAREN) && !self.is_at_end() {
              if self.match_token(&TokenType::COMMA) {
                  self.advance();
                  continue;
              }

              let field = self.parse_enum_field()?;
              fields.push(field);
          }

          // 4. Expect ')'
          self.expect(&TokenType::RPAREN)?;

          Ok(EnumVariant {
              name: variant_name,
              fields,
              span: Span::unknown(),
          })
      }

      fn parse_enum_field(&mut self) -> Result<EnumField, ParseError> {
          // 1. Parse field name
          let field_name = self.expect_identifier()?;

          // 2. Expect ':'
          self.expect(&TokenType::COLON)?;

          // 3. Parse type name (identifier)
          let type_name = self.expect_identifier()?;

          Ok(EnumField {
              name: field_name,
              type_name,
              span: Span::unknown(),
          })
      }

      fn check_duplicate_variants(&self, variants: &[EnumVariant]) -> Result<(), ParseError> {
          use std::collections::HashSet;
          let mut seen = HashSet::new();

          for variant in variants {
              if !seen.insert(&variant.name) {
                  return Err(ParseError::UnexpectedToken {
                      found: TokenType::IDENTIFIER,
                      expected: format!("unique variant name (duplicate: {})", variant.name),
                      line: 0,
                  });
              }
          }

          Ok(())
      }
  }

  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn test_parse_two_variant_enum() {
          let input = r#"
              @enum Result {
                  Ok(value: IntegerBox)
                  Err(error: StringBox)
              }
          "#;

          let result = NyashParser::parse_from_string(input);
          assert!(result.is_ok());

          // TODO: Add assertions on AST structure
      }

      #[test]
      fn test_parse_zero_field_variant() {
          let input = r#"
              @enum Option {
                  Some(value: IntegerBox)
                  None()
              }
          "#;

          let result = NyashParser::parse_from_string(input);
          assert!(result.is_ok());
      }
  }
  ```

- [ ] **Build to verify**:
  ```bash
  cargo build
  ```

### Phase 1.5: Parser Integration (1 hour)

- [ ] **Modify** `src/parser/declarations/mod.rs`:
  ```rust
  pub mod enum_parser;  // ADD this line
  ```

- [ ] **Modify** `src/parser/mod.rs` - Find `parse_statement()` and add:
  ```rust
  // Around line 280, in parse_statement():

  // Handle @enum declarations
  if self.match_token(&TokenType::MacroKeyword) {
      // Peek ahead to check if it's "enum"
      let next_token = self.peek();
      if let Some(Token { token_type: TokenType::IDENTIFIER, value, .. }) = next_token {
          if value == "enum" {
              return self.parse_enum_declaration();
          }
      }
  }
  ```

- [ ] **Build to verify**:
  ```bash
  cargo build
  ```

### Phase 1.6: Parser Tests (30 minutes)

- [ ] **Run parser unit tests**:
  ```bash
  cargo test --lib enum_parser::tests
  ```

- [ ] **Fix any failures**

- [ ] **Add more test cases**:
  - [ ] Multi-variant enum (3+ variants)
  - [ ] Generic type parameters (T, E)
  - [ ] Duplicate variant error
  - [ ] Invalid syntax errors

### Phase 1.7: Day 1 Commit (30 minutes)

- [ ] **Verify all tests pass**:
  ```bash
  cargo test
  cargo build --release
  ```

- [ ] **Commit changes**:
  ```bash
  git add src/ast.rs src/parser/declarations/enum_parser.rs src/parser/declarations/mod.rs src/parser/mod.rs
  git commit -m "feat(parser): @enum declaration parsing (Day 1)

  - Add EnumDeclaration/EnumVariant/EnumField AST nodes
  - Implement parse_enum_declaration() in enum_parser.rs
  - Add MacroKeyword token support
  - Integrate enum parsing in main parser loop
  - Add parser unit tests

  Related: Phase 19 - Enum + Match Implementation"
  ```

---

## Day 2: Macro Expander (Code Generation) ⏰ 8 hours

### Phase 2.1: Setup (30 minutes)

- [ ] **Create branch**:
  ```bash
  git checkout -b feature/enum-day2-expander
  git merge feature/enum-day1-parser
  ```

- [ ] **Read reference files**:
  - [ ] `src/macro/engine.rs` - Macro expansion engine
  - [ ] `src/macro/macro_box.rs` - Built-in macro examples
  - [ ] `apps/lib/boxes/option.hako` - Manual Option implementation
  - [ ] `apps/lib/boxes/result.hako` - Manual Result implementation

### Phase 2.2: Expander Implementation (5 hours)

- [ ] **Create** `src/macro/enum_expander.rs`:
  ```rust
  /*!
   * Enum Macro Expander Module
   *
   * Transforms EnumDeclaration AST into BoxDeclaration + StaticBoxDeclaration
   */

  use crate::ast::{ASTNode, EnumDeclaration, EnumVariant, EnumField, Span};
  use crate::ast::{LiteralValue, BinaryOperator};
  use std::collections::HashMap;

  /// Expand @enum declaration into Box + StaticBox AST nodes
  ///
  /// Transformation:
  ///   @enum Result { Ok(value: T), Err(error: E) }
  ///
  /// Generates:
  ///   box ResultBox<T, E> {
  ///     variant: StringBox
  ///     ok_value: T
  ///     err_error: E
  ///
  ///     birth(variant, ...) { ... }
  ///     is_ok() { ... }
  ///     is_err() { ... }
  ///   }
  ///
  ///   static box Result {
  ///     Ok(value) { return new ResultBox("Ok", value, null) }
  ///     Err(error) { return new ResultBox("Err", null, error) }
  ///   }
  pub fn expand_enum(decl: &EnumDeclaration) -> Result<Vec<ASTNode>, EnumExpansionError> {
      let box_node = generate_box_declaration(decl)?;
      let static_node = generate_static_box(decl)?;

      Ok(vec![box_node, static_node])
  }

  fn generate_box_declaration(decl: &EnumDeclaration) -> Result<ASTNode, EnumExpansionError> {
      let box_name = format!("{}Box", decl.name);

      // 1. Collect all fields from all variants
      let mut fields = vec!["variant".to_string()]; // Always have variant field
      let mut all_variant_fields = Vec::new();

      for variant in &decl.variants {
          for field in &variant.fields {
              let field_name = format!("{}_{}", variant.name.to_lowercase(), field.name);
              fields.push(field_name.clone());
              all_variant_fields.push((variant.name.clone(), field_name, field.type_name.clone()));
          }
      }

      // 2. Generate birth() constructor
      let birth_method = generate_birth_method(decl, &all_variant_fields)?;

      // 3. Generate is_* query methods
      let mut methods = HashMap::new();
      methods.insert("birth".to_string(), birth_method);

      for variant in &decl.variants {
          let is_method_name = format!("is_{}", variant.name.to_lowercase());
          let is_method = generate_is_method(&variant.name)?;
          methods.insert(is_method_name, is_method);
      }

      // 4. Generate unwrap() method (for Option-like enums)
      // TODO: Add unwrap, unwrap_or, map, etc.

      Ok(ASTNode::BoxDeclaration {
          name: box_name,
          fields,
          methods,
          constructors: HashMap::new(),
          init_fields: vec![],
          weak_fields: vec![],
          is_interface: false,
          extends: vec![],
          implements: vec![],
          type_parameters: vec![], // TODO: Extract from enum declaration
          is_static: false,
          static_init: None,
          span: Span::unknown(),
      })
  }

  fn generate_static_box(decl: &EnumDeclaration) -> Result<ASTNode, EnumExpansionError> {
      let static_name = decl.name.clone();

      // Generate constructor method for each variant
      let mut methods = HashMap::new();

      for variant in &decl.variants {
          let constructor_method = generate_variant_constructor(decl, variant)?;
          methods.insert(variant.name.clone(), constructor_method);
      }

      Ok(ASTNode::BoxDeclaration {
          name: static_name,
          fields: vec![],
          methods,
          constructors: HashMap::new(),
          init_fields: vec![],
          weak_fields: vec![],
          is_interface: false,
          extends: vec![],
          implements: vec![],
          type_parameters: vec![],
          is_static: true, // This is a static box
          static_init: None,
          span: Span::unknown(),
      })
  }

  fn generate_birth_method(
      decl: &EnumDeclaration,
      all_fields: &[(String, String, String)],
  ) -> Result<ASTNode, EnumExpansionError> {
      // birth(variant, field1, field2, ...)
      let mut params = vec!["variant".to_string()];
      for (_, field_name, _) in all_fields {
          params.push(field_name.clone());
      }

      // Body: me.variant = variant; me.field1 = field1; ...
      let mut body = Vec::new();

      // me.variant = variant
      body.push(ASTNode::Assignment {
          target: Box::new(ASTNode::FieldAccess {
              object: Box::new(ASTNode::MeExpression { span: Span::unknown() }),
              field: "variant".to_string(),
              span: Span::unknown(),
          }),
          value: Box::new(ASTNode::Variable {
              name: "variant".to_string(),
              span: Span::unknown(),
          }),
          span: Span::unknown(),
      });

      // me.field1 = field1, etc.
      for (_, field_name, _) in all_fields {
          body.push(ASTNode::Assignment {
              target: Box::new(ASTNode::FieldAccess {
                  object: Box::new(ASTNode::MeExpression { span: Span::unknown() }),
                  field: field_name.clone(),
                  span: Span::unknown(),
              }),
              value: Box::new(ASTNode::Variable {
                  name: field_name.clone(),
                  span: Span::unknown(),
              }),
              span: Span::unknown(),
          });
      }

      Ok(ASTNode::FunctionDeclaration {
          name: "birth".to_string(),
          params,
          body,
          is_static: false,
          is_override: false,
          span: Span::unknown(),
      })
  }

  fn generate_is_method(variant_name: &str) -> Result<ASTNode, EnumExpansionError> {
      // is_ok() { return me.variant == "Ok" }
      let body = vec![ASTNode::Return {
          value: Some(Box::new(ASTNode::BinaryOp {
              operator: BinaryOperator::Equal,
              left: Box::new(ASTNode::FieldAccess {
                  object: Box::new(ASTNode::MeExpression { span: Span::unknown() }),
                  field: "variant".to_string(),
                  span: Span::unknown(),
              }),
              right: Box::new(ASTNode::Literal {
                  value: LiteralValue::String(variant_name.to_string()),
                  span: Span::unknown(),
              }),
              span: Span::unknown(),
          })),
          span: Span::unknown(),
      }];

      Ok(ASTNode::FunctionDeclaration {
          name: format!("is_{}", variant_name.to_lowercase()),
          params: vec![],
          body,
          is_static: false,
          is_override: false,
          span: Span::unknown(),
      })
  }

  fn generate_variant_constructor(
      decl: &EnumDeclaration,
      variant: &EnumVariant,
  ) -> Result<ASTNode, EnumExpansionError> {
      // Ok(value) { return new ResultBox("Ok", value, null) }
      let box_name = format!("{}Box", decl.name);

      let mut params = Vec::new();
      let mut args = vec![ASTNode::Literal {
          value: LiteralValue::String(variant.name.clone()),
          span: Span::unknown(),
      }];

      // Collect this variant's fields
      for field in &variant.fields {
          params.push(field.name.clone());
          args.push(ASTNode::Variable {
              name: field.name.clone(),
              span: Span::unknown(),
          });
      }

      // Fill in null for other variants' fields
      // TODO: Calculate total field count from all variants
      // For now, just add the params we have

      let body = vec![ASTNode::Return {
          value: Some(Box::new(ASTNode::New {
              class: box_name,
              arguments: args,
              type_arguments: vec![],
              span: Span::unknown(),
          })),
          span: Span::unknown(),
      }];

      Ok(ASTNode::FunctionDeclaration {
          name: variant.name.clone(),
          params,
          body,
          is_static: true,
          is_override: false,
          span: Span::unknown(),
      })
  }

  #[derive(Debug)]
  pub enum EnumExpansionError {
      DuplicateVariant(String),
      DuplicateField(String),
      InvalidTypeName(String),
  }

  impl std::fmt::Display for EnumExpansionError {
      fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          match self {
              Self::DuplicateVariant(name) => write!(f, "Duplicate variant: {}", name),
              Self::DuplicateField(name) => write!(f, "Duplicate field: {}", name),
              Self::InvalidTypeName(name) => write!(f, "Invalid type name: {}", name),
          }
      }
  }

  impl std::error::Error for EnumExpansionError {}

  #[cfg(test)]
  mod tests {
      use super::*;

      #[test]
      fn test_expand_result_enum() {
          // Create EnumDeclaration for Result
          let decl = EnumDeclaration {
              name: "Result".to_string(),
              variants: vec![
                  EnumVariant {
                      name: "Ok".to_string(),
                      fields: vec![EnumField {
                          name: "value".to_string(),
                          type_name: "IntegerBox".to_string(),
                          span: Span::unknown(),
                      }],
                      span: Span::unknown(),
                  },
                  EnumVariant {
                      name: "Err".to_string(),
                      fields: vec![EnumField {
                          name: "error".to_string(),
                          type_name: "StringBox".to_string(),
                          span: Span::unknown(),
                      }],
                      span: Span::unknown(),
                  },
              ],
              span: Span::unknown(),
          };

          let result = expand_enum(&decl);
          assert!(result.is_ok());

          let nodes = result.unwrap();
          assert_eq!(nodes.len(), 2); // Box + StaticBox

          // TODO: Add detailed assertions on generated AST structure
      }
  }
  ```

- [ ] **Build to verify**:
  ```bash
  cargo build
  ```

### Phase 2.3: Engine Integration (1.5 hours)

- [ ] **Modify** `src/macro/mod.rs`:
  ```rust
  pub mod enum_expander;  // ADD this line
  ```

- [ ] **Modify** `src/macro/engine.rs` - Find `expand()` method and add:
  ```rust
  use crate::macro::enum_expander;

  impl MacroEngine {
      pub fn expand(&mut self, ast: &ASTNode) -> (ASTNode, Vec<Patch>) {
          match ast {
              ASTNode::EnumDeclaration { .. } => {
                  // Expand enum macro
                  match enum_expander::expand_enum(ast) {
                      Ok(nodes) => {
                          // Replace EnumDeclaration with expanded nodes
                          // TODO: Implement node replacement logic
                          // For now, return first node
                          return (nodes[0].clone(), vec![]);
                      }
                      Err(e) => {
                          eprintln!("[macro][enum] Expansion error: {}", e);
                          return (ast.clone(), vec![]);
                      }
                  }
              }
              ASTNode::Program { statements, span } => {
                  // Expand each statement
                  let mut expanded_statements = Vec::new();
                  for stmt in statements {
                      match stmt {
                          ASTNode::EnumDeclaration { .. } => {
                              // Expand enum and add both generated nodes
                              if let Ok(nodes) = enum_expander::expand_enum(stmt) {
                                  expanded_statements.extend(nodes);
                              }
                          }
                          _ => {
                              // Recursively expand other nodes
                              let (expanded, _) = self.expand(stmt);
                              expanded_statements.push(expanded);
                          }
                      }
                  }
                  return (ASTNode::Program {
                      statements: expanded_statements,
                      span: span.clone(),
                  }, vec![]);
              }
              _ => (ast.clone(), vec![]),
          }
      }
  }
  ```

- [ ] **Build to verify**:
  ```bash
  cargo build
  ```

### Phase 2.4: Expander Tests (1 hour)

- [ ] **Run expander unit tests**:
  ```bash
  cargo test --lib enum_expander::tests
  ```

- [ ] **Add more test cases**:
  - [ ] Test generated Box structure (fields, methods)
  - [ ] Test generated StaticBox structure (constructor methods)
  - [ ] Test is_* method generation
  - [ ] Test error cases

### Phase 2.5: Day 2 Commit (30 minutes)

- [ ] **Verify all tests pass**:
  ```bash
  cargo test
  cargo build --release
  ```

- [ ] **Commit changes**:
  ```bash
  git add src/macro/enum_expander.rs src/macro/mod.rs src/macro/engine.rs
  git commit -m "feat(macro): @enum expansion to Box + StaticBox (Day 2)

  - Implement expand_enum() in enum_expander.rs
  - Generate BoxDeclaration with variant field + variant fields
  - Generate StaticBoxDeclaration with constructor methods
  - Generate is_* query methods
  - Integrate enum expansion in macro engine
  - Add expander unit tests

  Related: Phase 19 - Enum + Match Implementation"
  ```

---

## Day 3: Integration Tests + Documentation ⏰ 8 hours

### Phase 3.1: Setup (30 minutes)

- [ ] **Create branch**:
  ```bash
  git checkout -b feature/enum-day3-integration
  git merge feature/enum-day2-expander
  ```

- [ ] **Create test directory**:
  ```bash
  mkdir -p apps/tests/enum
  ```

### Phase 3.2: Integration Tests (3 hours)

- [ ] **Create** `apps/tests/enum/test_enum_basic.hako`:
  ```hakorune
  @enum Result {
      Ok(value: IntegerBox)
      Err(error: StringBox)
  }

  static box Main {
      main() {
          local r1
          r1 = Result.Ok(42)

          if r1.is_ok() {
              print("PASS: is_ok works")
          } else {
              print("FAIL: is_ok broken")
              return false
          }

          local r2
          r2 = Result.Err("oops")

          if r2.is_err() {
              print("PASS: is_err works")
          } else {
              print("FAIL: is_err broken")
              return false
          }

          return true
      }
  }
  ```

- [ ] **Run test**:
  ```bash
  ./target/release/hako apps/tests/enum/test_enum_basic.hako
  ```

- [ ] **Fix any issues**

- [ ] **Create** `apps/tests/enum/test_enum_option.hako`:
  ```hakorune
  @enum Option {
      Some(value: IntegerBox)
      None()
  }

  static box Main {
      main() {
          local opt1
          opt1 = Option.Some(100)

          if opt1.is_some() {
              print("PASS: is_some works")
          } else {
              print("FAIL: is_some broken")
              return false
          }

          local opt2
          opt2 = Option.None()

          if opt2.is_none() {
              print("PASS: is_none works")
          } else {
              print("FAIL: is_none broken")
              return false
          }

          return true
      }
  }
  ```

- [ ] **Run test**:
  ```bash
  ./target/release/hako apps/tests/enum/test_enum_option.hako
  ```

- [ ] **Create** `apps/tests/enum/test_enum_multi.hako`:
  ```hakorune
  @enum HttpStatus {
      Ok(code: IntegerBox)
      Redirect(url: StringBox)
      Error(message: StringBox)
  }

  static box Main {
      main() {
          local s1
          s1 = HttpStatus.Ok(200)

          local s2
          s2 = HttpStatus.Redirect("/home")

          local s3
          s3 = HttpStatus.Error("Not Found")

          if s1.is_ok() and s2.is_redirect() and s3.is_error() {
              print("PASS: all variants work")
              return true
          } else {
              print("FAIL: variant detection broken")
              return false
          }
      }
  }
  ```

- [ ] **Run all enum tests**:
  ```bash
  for test in apps/tests/enum/*.hako; do
      echo "Running $test..."
      ./target/release/hako "$test" || echo "FAILED: $test"
  done
  ```

### Phase 3.3: User Documentation (2.5 hours)

- [ ] **Create** `docs/reference/language/enum-syntax.md`:
  ```markdown
  # Enum Syntax Reference

  ## Overview

  Enums (enumerated types) allow you to define a type by enumerating its possible variants. Each variant can optionally carry associated data.

  ## Syntax

  ```hakorune
  @enum EnumName {
      VariantA(field1: Type1, field2: Type2)
      VariantB(field: Type)
      VariantC()
  }
  ```

  ## Examples

  ### Basic Two-Variant Enum (Result)

  ```hakorune
  @enum Result {
      Ok(value: IntegerBox)
      Err(error: StringBox)
  }

  static box Main {
      main() {
          local r
          r = Result.Ok(42)

          if r.is_ok() {
              print("Success!")
          }

          return true
      }
  }
  ```

  ### Option Type

  ```hakorune
  @enum Option {
      Some(value: IntegerBox)
      None()
  }

  static box Main {
      main() {
          local opt
          opt = Option.Some(100)

          if opt.is_some() {
              print("Has value")
          } else {
              print("Empty")
          }

          return true
      }
  }
  ```

  ### Multi-Variant Enum

  ```hakorune
  @enum HttpStatus {
      Ok(code: IntegerBox)
      Redirect(url: StringBox)
      Error(message: StringBox)
  }
  ```

  ## Generated Methods

  When you declare `@enum Name { ... }`, Hakorune generates:

  1. **Box class** (`NameBox`):
     - `variant: StringBox` field (stores variant name)
     - Fields for all variants (with nullable types)
     - `birth(variant, ...)` constructor
     - `is_*()` query methods for each variant

  2. **Static box** (`Name`):
     - Constructor methods for each variant (`Ok(value)`, `Err(error)`)

  ## Best Practices

  - Use `PascalCase` for enum names
  - Use `PascalCase` for variant names
  - Use `snake_case` for field names
  - Prefer descriptive variant names (`Ok`/`Err` over `Success`/`Failure`)

  ## Comparison: @enum vs Manual Implementation

  ### Using @enum (Clean)
  ```hakorune
  @enum Result { Ok(value: T), Err(error: E) }
  ```

  ### Manual Implementation (Verbose)
  ```hakorune
  box ResultBox {
      variant: StringBox
      ok_value: IntegerBox
      err_error: StringBox

      birth(variant, ok_value, err_error) {
          me.variant = variant
          me.ok_value = ok_value
          me.err_error = err_error
      }

      is_ok() { return me.variant == "Ok" }
      is_err() { return me.variant == "Err" }
  }

  static box Result {
      Ok(value) { return new ResultBox("Ok", value, null) }
      Err(error) { return new ResultBox("Err", null, error) }
  }
  ```

  ## Limitations (Day 1 MVP)

  - No pattern matching yet (use `is_*()` methods)
  - No `unwrap()` or utility methods yet
  - No exhaustiveness checking

  ## See Also

  - [Quick Reference](quick-reference.md)
  - [Box System](../boxes-system/)
  - Phase 19.2: Match Expression (coming soon)
  ```

- [ ] **Verify markdown renders correctly**

### Phase 3.4: Developer Documentation (1.5 hours)

- [ ] **Create** `docs/development/roadmap/phases/phase-19-enum-match/enum-implementation-notes.md`:
  ```markdown
  # @enum Implementation Notes (Developer Guide)

  ## Architecture Overview

  See [enum-module-architecture.md](enum-module-architecture.md) for complete design.

  ### Pipeline

  ```
  Source Code
      ↓ Tokenizer
  Token Stream
      ↓ Parser (enum_parser.rs)
  EnumDeclaration AST
      ↓ Macro Engine
  expand_enum() (enum_expander.rs)
      ↓
  BoxDeclaration + StaticBoxDeclaration AST
      ↓ MIR Builder (existing)
  MIR
      ↓ Backend (existing)
  Executable
  ```

  ## Key Files

  | File | Responsibility |
  |------|----------------|
  | `src/parser/declarations/enum_parser.rs` | Parse @enum syntax |
  | `src/ast.rs` | EnumDeclaration/Variant/Field structs |
  | `src/macro/enum_expander.rs` | Transform enum → box AST |
  | `src/macro/engine.rs` | Orchestrate expansion |
  | `apps/tests/enum/` | Integration tests |

  ## Generated Code Structure

  Input:
  ```hakorune
  @enum Result { Ok(value: T), Err(error: E) }
  ```

  Generates:
  ```hakorune
  box ResultBox<T, E> {
      variant: StringBox
      ok_value: T
      err_error: E

      birth(variant, ok_value, err_error) {
          me.variant = variant
          me.ok_value = ok_value
          me.err_error = err_error
      }

      is_ok() { return me.variant == "Ok" }
      is_err() { return me.variant == "Err" }
  }

  static box Result {
      Ok(value) { return new ResultBox("Ok", value, null) }
      Err(error) { return new ResultBox("Err", null, error) }
  }
  ```

  ## Known Issues

  - [ ] TODO: Implement unwrap() method
  - [ ] TODO: Generic type parameter extraction
  - [ ] TODO: Null handling for unused variant fields

  ## Future Work (Phase 19.2+)

  - Match expressions
  - Pattern matching destructuring
  - Exhaustiveness checking
  - Hakorune self-hosting (rewrite expander in Hakorune)

  ## Testing

  Run all tests:
  ```bash
  cargo test --lib enum_parser::tests
  cargo test --lib enum_expander::tests
  ./target/release/hako apps/tests/enum/*.hako
  ```
  ```

### Phase 3.5: Final Integration (30 minutes)

- [ ] **Run full test suite**:
  ```bash
  cargo test
  cargo build --release
  ./target/release/hako apps/tests/enum/test_enum_basic.hako
  ./target/release/hako apps/tests/enum/test_enum_option.hako
  ./target/release/hako apps/tests/enum/test_enum_multi.hako
  ```

- [ ] **Fix any remaining issues**

### Phase 3.6: Day 3 Commit (30 minutes)

- [ ] **Verify everything works**:
  ```bash
  cargo test
  cargo build --release
  for test in apps/tests/enum/*.hako; do ./target/release/hako "$test"; done
  ```

- [ ] **Commit changes**:
  ```bash
  git add apps/tests/enum/ docs/reference/language/enum-syntax.md docs/development/roadmap/phases/phase-19-enum-match/enum-implementation-notes.md
  git commit -m "feat(enum): integration tests + documentation (Day 3)

  - Add test_enum_basic.hako (Result-like)
  - Add test_enum_option.hako (Option-like)
  - Add test_enum_multi.hako (3+ variants)
  - Add enum-syntax.md user guide
  - Add enum-implementation-notes.md developer guide
  - All integration tests passing

  Related: Phase 19 - Enum + Match Implementation"
  ```

---

## Day 4: Polish + Edge Cases (Buffer) ⏰ 4-8 hours

### Phase 4.1: Error Handling (2 hours)

- [ ] **Improve error messages**:
  - [ ] Duplicate variant detection
  - [ ] Invalid variant name
  - [ ] Invalid field name
  - [ ] Missing required syntax elements

- [ ] **Add error tests**:
  ```hakorune
  // apps/tests/enum/test_enum_error_cases.hako

  // This should fail: duplicate variant
  @enum Bad1 {
      Ok(value: IntegerBox)
      Ok(value: IntegerBox)
  }

  // This should fail: reserved method name
  @enum Bad2 {
      Ok(is_ok: IntegerBox)
  }
  ```

### Phase 4.2: Edge Cases (2 hours)

- [ ] **Test zero-field variants**:
  ```hakorune
  @enum Flag {
      On()
      Off()
  }
  ```

- [ ] **Test single variant enum**:
  ```hakorune
  @enum Single {
      Only(value: IntegerBox)
  }
  ```

- [ ] **Test many variants (5+)**:
  ```hakorune
  @enum Color {
      Red(r: IntegerBox)
      Green(g: IntegerBox)
      Blue(b: IntegerBox)
      Yellow(y: IntegerBox)
      Magenta(m: IntegerBox)
  }
  ```

### Phase 4.3: Code Review & Refactor (2 hours)

- [ ] **Review all code**:
  - [ ] Remove debug prints
  - [ ] Add documentation comments
  - [ ] Simplify complex functions
  - [ ] Check for code duplication

- [ ] **Run lints**:
  ```bash
  cargo clippy
  cargo fmt --check
  ```

- [ ] **Fix any warnings**

### Phase 4.4: Final Commit (30 minutes)

- [ ] **Verify everything**:
  ```bash
  cargo test
  cargo build --release
  cargo clippy
  cargo fmt
  ```

- [ ] **Commit polish**:
  ```bash
  git add -A
  git commit -m "refactor(enum): polish error handling + edge cases (Day 4)

  - Improve error messages (duplicate variants, invalid names)
  - Add error case tests
  - Handle edge cases (zero-field variants, single variant, many variants)
  - Code review and refactoring
  - Clippy and fmt fixes

  Related: Phase 19 - Enum + Match Implementation"
  ```

---

## Final Steps: Merge & Celebrate 🎉

- [ ] **Create PR**:
  ```bash
  git checkout main
  git pull
  git merge feature/enum-day3-integration
  git push origin feature/enum-day3-integration
  ```

- [ ] **Write PR description** (use architecture doc summary)

- [ ] **Request code review**

- [ ] **Address review feedback**

- [ ] **Merge to main**

- [ ] **Update Phase 19 README with completion status**

- [ ] **Celebrate!** 🎉 Enum implementation complete!

---

## Success Criteria Checklist

- [ ] ✅ Parser can parse `@enum Name { Variant(field: Type), ... }`
- [ ] ✅ AST nodes correctly constructed
- [ ] ✅ Expander generates Box + StaticBox AST
- [ ] ✅ Generated code is valid Hakorune
- [ ] ✅ Integration tests pass (Result, Option, multi-variant)
- [ ] ✅ User documentation complete
- [ ] ✅ Developer documentation complete
- [ ] ✅ Error handling works
- [ ] ✅ Edge cases handled
- [ ] ✅ No regressions in existing tests

---

**Total Estimated Time**: 24-28 hours (3 days core + 1 day buffer)
**Status**: Ready to Start ✅
**Next Phase**: Phase 19.2 - Match Expressions
