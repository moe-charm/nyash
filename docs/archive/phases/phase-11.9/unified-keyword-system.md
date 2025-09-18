# Nyash統一予約語システム仕様

## 🎯 目的

Nyashの全実行層（Script/MIR/VM/JIT）で完全に同一の予約語・文法解釈を実現する。

## 📊 現状の予約語分散状況

### Tokenizer層 (src/tokenizer.rs)
```rust
// 現在: ハードコードされた予約語
match word.as_str() {
    "box" => TokenType::BOX,
    "me" => TokenType::ME,
    "from" => TokenType::FROM,
    "loop" => TokenType::LOOP,
    "if" => TokenType::IF,
    "else" => TokenType::ELSE,
    "local" => TokenType::LOCAL,
    "return" => TokenType::RETURN,
    "new" => TokenType::NEW,
    "static" => TokenType::STATIC,
    "init" => TokenType::INIT,
    "birth" => TokenType::BIRTH,
    "pack" => TokenType::PACK,
    "override" => TokenType::OVERRIDE,
    "and" => TokenType::AND,
    "or" => TokenType::OR,
    "not" => TokenType::NOT,
    _ => TokenType::IDENTIFIER(word)
}
```

### MIR Builder層での独自解釈
```rust
// 現在: MIR生成時の独自判断
fn build_from_call(&mut self) {
    // "from"の解釈がTokenizerと異なる可能性
}
```

### VM/JIT層での実行差異
```rust
// VM: 文字列連結の独自実装
VMValue::String(s1) + VMValue::String(s2) => concat

// JIT: 異なる最適化戦略
emit_call("nyash.string.concat_hh")
```

## 🏗️ 統一予約語システムの設計

### 1. 中央予約語レジストリ
```rust
// src/grammar/keyword_registry.rs
pub struct UnifiedKeywordRegistry {
    keywords: HashMap<&'static str, UnifiedKeyword>,
}

pub struct UnifiedKeyword {
    // トークン情報
    pub token_type: TokenType,
    pub literal: &'static str,
    
    // 文法情報
    pub category: KeywordCategory,
    pub precedence: Option<i32>,
    
    // セマンティクス情報
    pub semantic_action: SemanticAction,
    pub mir_instruction: Option<MirInstruction>,
    pub vm_opcode: Option<VmOpcode>,
    pub jit_pattern: Option<JitPattern>,
    
    // メタ情報
    pub deprecated_aliases: Vec<&'static str>,
    pub ai_hint: &'static str,
}

// 静的に初期化される単一インスタンス
pub static KEYWORDS: Lazy<UnifiedKeywordRegistry> = Lazy::new(|| {
    let mut registry = UnifiedKeywordRegistry::new();
    
    // "me" - 自己参照
    registry.register(UnifiedKeyword {
        token_type: TokenType::ME,
        literal: "me",
        category: KeywordCategory::SelfReference,
        semantic_action: SemanticAction::LoadSelf,
        mir_instruction: Some(MirInstruction::LoadLocal(0)),
        vm_opcode: Some(VmOpcode::LOAD_ME),
        jit_pattern: Some(JitPattern::LoadFirstParam),
        deprecated_aliases: vec!["this", "self", "@"],
        ai_hint: "Always use 'me' for self-reference",
        precedence: None,
    });
    
    // "from" - デリゲーション
    registry.register(UnifiedKeyword {
        token_type: TokenType::FROM,
        literal: "from",
        category: KeywordCategory::Delegation,
        semantic_action: SemanticAction::DelegateCall,
        mir_instruction: Some(MirInstruction::Call),
        vm_opcode: Some(VmOpcode::DELEGATE_CALL),
        jit_pattern: Some(JitPattern::VirtualCall),
        deprecated_aliases: vec!["super", "parent", "base"],
        ai_hint: "Use 'from' for parent method calls",
        precedence: None,
    });
    
    // "loop" - 制御フロー
    registry.register(UnifiedKeyword {
        token_type: TokenType::LOOP,
        literal: "loop",
        category: KeywordCategory::ControlFlow,
        semantic_action: SemanticAction::Loop,
        mir_instruction: Some(MirInstruction::Branch),
        vm_opcode: Some(VmOpcode::LOOP_START),
        jit_pattern: Some(JitPattern::LoopWithSafepoint),
        deprecated_aliases: vec!["while", "for", "repeat"],
        ai_hint: "Only 'loop' exists for iteration",
        precedence: None,
    });
    
    // 演算子も統一管理
    registry.register(UnifiedKeyword {
        token_type: TokenType::PLUS,
        literal: "+",
        category: KeywordCategory::BinaryOperator,
        precedence: Some(10),
        semantic_action: SemanticAction::Add,
        mir_instruction: Some(MirInstruction::BinOp(BinOpKind::Add)),
        vm_opcode: Some(VmOpcode::ADD),
        jit_pattern: Some(JitPattern::PolymorphicAdd),
        deprecated_aliases: vec![],
        ai_hint: "String + String = concat, Number + Number = add",
    });
    
    registry
});
```

### 2. 各層での統一API使用

#### Tokenizer統合
```rust
impl NyashTokenizer {
    fn tokenize_word(&mut self, word: String) -> TokenType {
        // 統一レジストリを参照
        KEYWORDS.lookup(&word)
            .map(|kw| kw.token_type.clone())
            .unwrap_or(TokenType::IDENTIFIER(word))
    }
}
```

#### Parser統合
```rust
impl Parser {
    fn parse_keyword(&mut self, token: &Token) -> Result<ASTNode> {
        if let Some(keyword) = KEYWORDS.get_by_token(&token.token_type) {
            // 統一されたセマンティクスアクションを実行
            match keyword.semantic_action {
                SemanticAction::Loop => self.parse_loop_unified(keyword),
                SemanticAction::DelegateCall => self.parse_from_unified(keyword),
                // ...
            }
        }
    }
}
```

#### MIR Builder統合
```rust
impl MirBuilder {
    fn build_keyword(&mut self, keyword: &UnifiedKeyword, args: Vec<MirValue>) -> MirValue {
        // 統一されたMIR命令を生成
        if let Some(mir_inst) = &keyword.mir_instruction {
            self.emit_unified(mir_inst.clone(), args)
        }
    }
}
```

#### VM統合
```rust
impl VM {
    fn execute_keyword(&mut self, keyword: &UnifiedKeyword) -> Result<()> {
        // 統一されたVMオペコードを実行
        if let Some(opcode) = &keyword.vm_opcode {
            self.execute_unified_opcode(opcode)
        }
    }
}
```

#### JIT統合
```rust
impl JitBuilder {
    fn compile_keyword(&mut self, keyword: &UnifiedKeyword, args: &[Value]) {
        // 統一されたJITパターンを適用
        if let Some(pattern) = &keyword.jit_pattern {
            self.emit_unified_pattern(pattern, args)
        }
    }
}
```

## 🔄 セマンティクス統一の例

### 現在の問題: "+" 演算子の挙動差異

```rust
// Interpreter: 独自の型変換ロジック
fn execute_add(&mut self, left: Value, right: Value) -> Value {
    // 複雑な型変換ロジック
}

// VM: 異なる型変換ロジック
fn vm_add(&mut self) {
    // 別の型変換ロジック
}

// JIT: さらに異なる最適化
fn jit_add(&mut self) {
    // JIT独自の最適化
}
```

### 統一後: 単一のセマンティクス定義

```rust
// src/grammar/unified_semantics.rs
pub struct UnifiedSemantics;

impl UnifiedSemantics {
    /// すべての層が使用する統一Add実装
    pub fn add(left: &Value, right: &Value) -> Result<Value> {
        use Value::*;
        match (left, right) {
            // String優先（Nyashの仕様）
            (String(s1), String(s2)) => Ok(String(s1.clone() + s2)),
            (String(s), other) | (other, String(s)) => {
                Ok(String(s.clone() + &Self::coerce_to_string(other)?))
            }
            // 数値演算
            (Integer(i1), Integer(i2)) => Ok(Integer(i1 + i2)),
            (Float(f1), Float(f2)) => Ok(Float(f1 + f2)),
            (Integer(i), Float(f)) | (Float(f), Integer(i)) => {
                Ok(Float(*i as f64 + f))
            }
            // その他はエラー
            _ => Err(Error::TypeMismatch)
        }
    }
    
    /// 統一された文字列変換
    pub fn coerce_to_string(value: &Value) -> Result<String> {
        match value {
            Value::String(s) => Ok(s.clone()),
            Value::Integer(i) => Ok(i.to_string()),
            Value::Float(f) => Ok(f.to_string()),
            Value::Bool(b) => Ok(b.to_string()),
            Value::Null => Ok("null".to_string()),
            _ => Err(Error::CannotCoerceToString)
        }
    }
}

// 各層での使用
// Interpreter
left_value = UnifiedSemantics::add(&left, &right)?;

// VM
let result = UnifiedSemantics::add(&self.pop()?, &self.pop()?)?;
self.push(result);

// JIT
self.emit_call("UnifiedSemantics::add", args);
```

## 📋 実装チェックリスト

- [ ] `src/grammar/keyword_registry.rs` - 統一予約語レジストリ
- [ ] `src/grammar/unified_semantics.rs` - 統一セマンティクス
- [ ] `src/grammar/mod.rs` - モジュール統合
- [ ] Tokenizer修正 - 統一レジストリ参照
- [ ] Parser修正 - 統一セマンティクス使用
- [ ] MIR Builder修正 - 統一MIR生成
- [ ] VM修正 - 統一実行
- [ ] JIT修正 - 統一コード生成
- [ ] テストスイート - 全層の一致確認
- [ ] ドキュメント - 統一仕様書

## 🎯 成功基準

1. **完全一致**: すべての層で同じ入力が同じ出力を生成
2. **単一修正**: 新予約語追加が1ファイルの修正で完了
3. **AI正確性**: AIが生成するコードのエラー率90%削減
4. **性能維持**: 統一化による性能劣化5%以内

## 🚀 移行計画

### Phase 1: 基盤構築（1週間）
- 統一レジストリ実装
- 既存コードとの並行動作

### Phase 2: Tokenizer/Parser統合（1週間）
- 段階的切り替え
- 差分検出とログ

### Phase 3: 実行層統合（2週間）
- MIR/VM/JIT の統一
- 包括的テスト

### Phase 4: 完全移行（1週間）
- 旧コード削除
- ドキュメント完成

これにより、Nyashのすべての層で完全に統一された予約語・文法解釈が実現される。