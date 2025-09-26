# AI深層考察: Nyash統一文法アーキテクチャ

## 🎯 概要

GeminiとCodexに時間無制限で深く考えてもらった、Nyash統一文法アーキテクチャに関する洞察をまとめました。

## 🔥 Gemini先生の洞察

### 核心的提言: 宣言的文法定義 + ビルド時コード生成

```
[ grammar.toml ] ← 宣言的SSoT（Single Source of Truth）
    ↓
[ build.rs ] ← メタプログラミング層
    ↓
├─ generated_tokens.rs
├─ generated_keywords.rs  
├─ generated_rules.rs
└─ generated_opcodes.rs
```

### 重要ポイント

1. **真の分離**: `UnifiedKeyword`構造体は依然として各層を密結合させる。宣言的ファイルからコード生成する方が疎結合を保てる。

2. **ゼロコスト抽象化**: 
   - ビルド時生成により実行時オーバーヘッドなし
   - `enum`と`match`文で高速ディスパッチ
   - `#[inline(always)]`で関数呼び出しコストなし

3. **コンパイラ駆動開発**:
   ```rust
   // 新機能追加時、全層でコンパイルエラー発生
   // → 実装漏れがなくなる
   match token {
       TokenType::Async => // 新しく追加されたので実装必須
       _ => // ...
   }
   ```

4. **他言語からの学び**:
   - **CPython**: `Grammar/Tokens`ファイルから生成
   - **V8**: Ignition(インタプリタ)とTurboFan(JIT)の分離
   - **rustc**: HIR→MIRという段階的表現

## 💡 Codex先生の洞察

### 核心的提言: MIRを中心とした統一セマンティクス基盤

```yaml
# grammar/nyash.yml
tokens:
  - name: ME
    literal: "me"
    soft: true
    contexts: ["expr", "pattern"]
    deprecated_aliases: ["self"]
    ai_hint: "Current object; not assignable."

operators:
  - symbol: "+"
    name: add
    precedence: 110
    associativity: left
    overloads:
      - types: ["i64","i64"] -> "i64"
        lower: MIR.AddI64
      - types: ["String","String"] -> "String"
        lower: MIR.Concat
```

### 実装戦略

1. **単一仕様ファイル**: `grammar/nyash.yml`に全て定義
   - キーワード、演算子、文法、型、強制変換
   - MIRローリング、VMオペコード、JITパターン
   - 非推奨、AIヒント

2. **コード生成クレート**: `crates/nygrammar-gen`
   - Perfect hash関数でO(1)キーワード認識
   - Pratt/PEGパーサーテーブル生成
   - 型ディスパッチマトリックス生成

3. **MIRが真実の基盤**:
   ```rust
   pub fn add(lhs: Value, rhs: Value) -> Result<MIRNode> {
       // 生成されたfast-pathを使用
       // 常にMIRノードを返す
   }
   ```

4. **性能最適化**:
   - ビルド時にすべて決定（実行時検索なし）
   - インラインキャッシュで呼び出しサイト最適化
   - ソフトキーワードはパーサー状態で判定

### 段階的移行計画

- **Phase 0**: ベースラインテスト（現状記録）
- **Phase 1**: 正準MIR定義
- **Phase 2**: KeywordRegistry生成
- **Phase 3**: UnifiedSemantics導入
- **Phase 4**: パーサー統一
- **Phase 5**: バックエンドマッピング
- **Phase 6**: 非推奨警告
- **Phase 7**: ツール/ドキュメント生成

## 🎯 統合された知見

両AIの提言を統合すると：

### 1. 宣言的定義 + コード生成が最強
- YAML/TOMLで文法を宣言的に定義
- build.rsでRustコードを生成
- 実行時オーバーヘッドゼロ

### 2. MIRを中心とした統一
- すべてのセマンティクスはMIRで表現
- 各バックエンドはMIRを実行/コンパイル
- 一貫性が自動的に保証される

### 3. AI友好的な設計
- 機械可読な仕様ファイル
- 豊富な例とエラーカタログ
- 自動生成されるドキュメント

### 4. 拡張性への配慮
- 新バックエンド追加が容易
- プラグインによる拡張可能
- 後方互換性の維持

## 📋 実装優先順位

1. **最優先**: `grammar/nyash.yml`の初版作成
2. **高優先**: `build.rs`によるトークン生成
3. **中優先**: MIR統一とUnifiedSemantics
4. **低優先**: JIT最適化ヒント

## 🚀 期待される効果

- **保守性**: 新機能追加が1箇所の修正で完了
- **一貫性**: 全層で同じセマンティクス保証
- **性能**: ビルド時最適化で実行時コストなし
- **AI対応**: LLMが正確にNyashコードを生成

これらの深い洞察により、Nyashの統一文法アーキテクチャは強固な基盤の上に構築されることになります。