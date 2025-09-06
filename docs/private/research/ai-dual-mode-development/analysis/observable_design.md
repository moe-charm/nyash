# 観測可能性の設計 - AI協調開発の要

## 概要

観測可能性（Observability）は、AI二重化モデルにおいて問題の迅速な特定と解決を可能にした核心的要素である。

## 観測可能性の原則

### 1. 単純な指標
```json
{
  "argc": 0  // たった1つの数値が問題を示す
}
```
- 複雑な状態を単純な値に還元
- AIも人間も即座に理解可能

### 2. 階層的な観測
```
レベル1: 成功/失敗（boolean）
レベル2: 理由（sig_mismatch, policy_denied等）
レベル3: 詳細（argc, arg_types等）
```

### 3. リアルタイム性
- JSONLによる即座の出力
- バッファリングなしの観測

## 実装例：JIT Event System

### イベント設計
```rust
#[derive(Serialize)]
pub enum Event {
    Compile { func: String, ms: f64 },
    Execute { func: String, ok: bool },
    Hostcall { 
        func: String, 
        method: String, 
        decision: String,
        argc: usize,        // 観測点1
        arg_types: Vec<String>  // 観測点2
    },
    Fallback { func: String, reason: String }
}
```

### 観測の効果
1. **argc==0** → MIR引数配線の欠落を即座に発見
2. **decision: "sig_mismatch"** → 型の不一致を特定
3. **arg_types: ["I64"]** → 期待値F64との差異を明示

## AI協調開発への影響

### 1. 問題の迅速な特定
```
観測: argc==0
↓ AIの推論
原因: MIR層での引数配線欠落
↓ 
解決: BoxCallのargs生成修正
```

### 2. 仮説の即座の検証
- 修正を入れる → argcが1になる → 成功を確認
- フィードバックループの高速化

### 3. AIへの明確な情報提供
- AIは観測可能な値から正確に問題を推論
- 曖昧さのない判断材料

## 観測点の設計パターン

### 1. 入口と出口
```rust
// 入口：何が入ってきたか
log_entry(args.len(), args.types());

// 処理
let result = process(args);

// 出口：何が出ていくか
log_exit(result.type(), result.success());
```

### 2. 分岐点での記録
```rust
match check_signature() {
    Allow => log_event("allow"),
    Deny(reason) => log_event("deny", reason),
}
```

### 3. 統計の自動集計
```rust
static COUNTER: AtomicU64 = AtomicU64::new(0);
// 各イベントで自動インクリメント
```

## 効果の定量化

### 従来の開発
- 問題発生 → 原因調査（数時間）→ 仮説 → 実装 → 検証
- トータル: 1-2日

### 観測可能性を持つAI協調開発
- 問題発生 → 観測値確認（数秒）→ AI推論（数分）→ 実装 → 即座に検証
- トータル: 1-2時間

## 設計のベストプラクティス

1. **観測点は最小限に**
   - 多すぎると情報過多
   - 少なすぎると問題特定困難

2. **階層的な詳細度**
   - 通常は概要のみ
   - 問題時は詳細を出力

3. **型安全な観測**
   ```rust
   // 文字列ではなく型で表現
   enum Decision { Allow, Deny(DenyReason) }
   ```

## 結論

観測可能性は、AI協調開発における**問題解決の高速道路**である：

1. **AIに明確な判断材料を提供**
2. **人間の認知負荷を軽減**
3. **フィードバックループを劇的に短縮**

「argc==0」という単純な観測が、数時間の開発時間短縮につながった事実は、観測可能性の設計がいかに重要かを示している。