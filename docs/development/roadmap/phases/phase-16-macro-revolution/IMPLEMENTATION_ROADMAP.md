# Phase 16 実装ロードマップ - Macro Revolution統合戦略

**策定日**: 2025-09-18
**ステータス**: 実装準備完了
**総工数見積もり**: 4-6週間（ChatGPT・Codex・Gemini合意）

## 🎯 統合戦略の核心

### AI三賢者の合意事項
- **ChatGPT**: "@derive(Equals)から始めて段階的価値提供"
- **Gemini**: "Pattern Matching → Macro Systemの順序が最適"  
- **Codex**: "HIRパッチ式で既存MIR14命令無変更が可能"

### 技術的確定事項
- ✅ **Property（実行時）+ Macro（コンパイル時）の厳密分離**
- ✅ **HIRパッチ式マクロエンジン**でMIR14命令不変
- ✅ **MacroBox<InputAst, OutputAst>**型安全設計
- ✅ **最小4つのテストケース**で品質保証

## 🚀 Phase 16.1: Pattern Matching基盤（優先実装）

### 期間: 2週間
### 理由: Gemini「マクロ実装のツールになる」

#### 実装内容
```nyash
// 基本パターンマッチング
local result = match value {
    0 => "zero",
    1..10 => "small", 
    _ => "other"
}

// Box destructuring  
match user_box {
    UserBox(name, age) => process(name, age),
    _ => error("invalid box")
}
```

#### 完了条件
- [ ] AST Pattern/Unifier実装
- [ ] 基本match式の動作確認
- [ ] Box destructuringの実装
- [ ] MIR lowering完了

## 🛠️ Phase 16.2: AST操作基盤（1週間）

### Codex推奨の技術基盤

#### 実装内容
- **AST Pattern**: 変数束縛、ワイルドカード、可変長（…）
- **準引用/脱引用**: ASTをコード片として安全に構築
- **リライト機能**: 訪問/置換の汎用器（Span伝播対応）

#### API設計
```rust
// Rust側の基盤API
trait MacroPattern {
    fn match_ast(&self, node: &AstNode) -> Option<HashMap<String, AstNode>>;
}

trait AstBuilder {
    fn quote(&self, code: &str) -> AstNode;
    fn unquote(&self, template: &AstNode, bindings: &HashMap<String, AstNode>) -> AstNode;
}
```

#### 完了条件
- [ ] Pattern matching for AST
- [ ] Quote/unquote mechanism
- [ ] AST rewriter with Span preservation
- [ ] Unit tests for all components

## 🎯 Phase 16.3: 最小マクロMVP（2週間）

### ChatGPT最優先: @derive(Equals)

#### 実装目標
```nyash
// 入力
@derive(Equals, ToString)
box UserBox {
    name: StringBox
    age: IntegerBox
}

// 自動生成（HIRパッチとして注入）
method equals(other: UserBox) -> BoolBox {
    return me.name == other.name && me.age == other.age
}

method toString() -> StringBox {
    return "UserBox(name=" + me.name + ", age=" + me.age + ")"
}
```

#### 技術アーキテクチャ
```
Parse → HIR → (Macro Expansion) → TypeCheck → MIR14 → Backends
```

#### HIRパッチ式設計
- **マクロはHIR上の「差分パッチ」として実行**
- **生成物は普通のBox/メソッド定義**
- **MIRには一切新命令を足さない**

#### Hygiene（衛生）設計
```nyash
// gensymによる名前衝突回避
method __generated_equals_1234(other: UserBox) -> BoolBox {
    // 生成されたメソッド
}
```

#### 完了条件
- [x] @derive(Equals)の動作確認（AST展開→MIRで実行）
- [x] @derive(ToString)の動作確認  
- [x] HIRパッチエンジンの安定動作（MVP: no‑op/derive注入）
- [x] 初期スモークにてgreen（追加テスト拡充中）

## 🛡️ Phase 16.4: @validate統合（1週間）

### ChatGPT第2優先: 型安全・入力品質

#### 実装目標
```nyash
@validate
box UserBox {
    @required @email
    email: StringBox
    
    @range(0, 150)
    age: IntegerBox
}
```

#### 自動生成
```nyash
method set_email(value: StringBox) {
    if value.length() == 0 {
        throw new ValidationError("email is required")
    }
    if !value.contains("@") {
        throw new ValidationError("invalid email format")
    }
    me.email = value
}
```

#### 完了条件
- [ ] @required, @email, @range実装
- [ ] ValidationError統合
- [ ] setter methods自動生成
- [ ] Property System統合

## ⚙️ Phase 16.5: @config_schema統合（1週間）

### ChatGPT第3優先: 実アプリ即効

#### 実装目標
```nyash
@config_schema
box AppConfig {
    @env("DATABASE_URL") @required
    database_url: StringBox
    
    @env("DEBUG") @default(false) @parse_bool
    debug: BoolBox
}
```

#### 自動生成
```nyash
static method load() -> Result<AppConfig, ConfigError> {
    // 環境変数ベース設定ローダー
}
```

#### 完了条件
- [ ] @env, @default, @required統合
- [ ] 環境変数読み込み
- [ ] 型変換（@parse_bool等）
- [ ] Result型での安全な設定読み込み

## 🎉 Phase 16.6: 統合テスト・デモ（1週間）

### 品質保証とデモンストレーション

#### ChatGPT推奨の必須テスト（進捗: 部分達成）
1. **derive等価性**: `UserBox("a",1) == UserBox("a",1)` → 真（達成）
2. **validation**: `age=200` → `ValidationError`（未）
3. **config**: `DATABASE_URL`未設定 → `Err`（未）
4. **hygiene**: 手書き`equals`と生成コードが衝突しない（MVPでは上書き回避で担保）

#### デバッグツール
- `nyash --expand`: マクロ展開結果の可視化
- `NYASH_MACRO_TRACE=1`: ステップバイステップ追跡

#### 完了条件
- [ ] 全テストケース通過
- [ ] 実用アプリでの動作確認
- [ ] パフォーマンス測定
- [ ] ドキュメント完成

## 🌟 Phase 16.7: 即効追加マクロ（1週間）

### ChatGPT推奨の低コスト・高価値マクロ

#### @test/@bench（最優先実装）
```nyash
@test
method test_user_creation() {
    local user = new UserBox("Alice", 25)
    assert user.name == "Alice"
}

@bench  
method bench_sorting() {
    // ベンチマーク処理
}
```
- **実装コスト**: 超低（関数収集+ランナーのみ）
- **価値**: 言語信頼性の即座向上
- **実行**: `nyash test`, `nyash bench`

#### @serde(Json)
```nyash
@serde(Json)
box ApiResponseBox {
    status: IntegerBox
    data: UserBox
}
```
- **実装コスト**: 超低（@derive(Json)拡張）
- **価値**: Web開発必須機能

## 🚀 Phase 16.8: DX革命マクロ（1-2週間）

#### @derive(Builder)（Nyash独自の魅力）
```nyash
@derive(Builder)
box HttpRequestBox {
    url: StringBox
    method: StringBox  
    headers: MapBox
}

// 美しいFluent API生成
local request = HttpRequestBox.builder()
    .url("https://api.example.com")
    .method("POST")
    .build()
```

## 🔧 Phase 16.9以降: 高度マクロ

#### Phase 16.9: @using(resource)（2-3週間）
- **RAII/cleanup統合**: リソース安全管理
- **実装コスト**: 高（スコープ管理複雑）

#### Phase 16.10: その他高度機能
- **@log(entry|exit)**: AOP基盤
- **@api_client**: OpenAPI統合
- **@sql_schema**: 型安全SQL
- **@state_machine**: 究極の差別化

## 📊 成功指標とマイルストーン

### Phase 16.3完了時（MVP達成）
- ✅ @derive(Equals, ToString)動作
- ✅ HIRパッチエンジン安定
- ✅ 既存MIR14バックエンドで実行可能
- ✅ `nyash --expand`でデバッグ可能

### Phase 16.6完了時（実用達成）
- ✅ @derive/@validate/@config_schema完全動作
- ✅ 実用アプリでの活用例動作
- ✅ 4つの必須テスト完全通過
- ✅ Property System完全統合

### Phase 16.7完了時（世界最強達成）
- ✅ JSON serde完全動作
- ✅ テストランナー統合
- ✅ API生成・SQL型安全の実証
- ✅ 他言語を超越する表現力実現

## ⚠️ リスク対応策

### 技術的リスク
- **無限展開**: 再帰上限と循環検出
- **デバッグ困難**: 展開トレースと可視化ツール
- **パフォーマンス**: HIRパッチの最適化

### プロジェクトリスク  
- **複雑化**: 段階的導入で制御
- **学習コスト**: 充実したドキュメントとサンプル
- **既存影響**: MIR14不変でリスク最小化

## 🎯 次のアクション（進捗反映）

### 即座着手（今週）
1. **Pattern/Quote最小実装**（$name / $...name / OrPattern）完了
2. **AST操作基盤API設計**（MVP）完了
3. **HIRパッチエンジン**（MVP）完了
4. **@derive(Equals/ToString)** 実装済み（MVP）

### 2週間後
1. **Test Runner拡張**（Box内/entry policy/args JSON）
2. **Pattern強化**（配列/マップ/中間可変）
3. **Macro debug CLI**（展開ステップの可視化の拡張）
4. **実用アプリ適用**（derive/test導入）

---

**Phase 16 Macro Revolution**により、Nyashは世界最強のマクロ言語への道を確実に歩む。

*全AI賢者の叡智を統合した、実現可能かつ革新的な実装戦略。*
