# BoxCall拡張によるABI戦略 - 最終実装決定 (2025-09-01)

## 🎯 両先生の回答から得た重要な洞察

### Gemini先生の誤解が示す重要な点
- 先生は「BoxCall = Box<dyn Trait>を渡す」と誤解
- しかし、この誤解が**abi_stable**の重要性を教えてくれた
- 将来的にRustトレイトオブジェクトを扱う際の指針に

### Codex先生の実践的分析
1. **abi_hintは実は不要かもしれない**
   - VM実行時にtype_idから判定可能
   - プラグインローダーが型情報を保持
   - MIR層を汚さない

2. **既存実装への影響最小化**
   - TypeMetaにABI情報を含める
   - nyash.tomlから読み込み済み
   - VM側で判定するだけ

## 🚀 最終実装方針

### Option A: 最小限実装（推奨）

```rust
// MIR層：変更なし！
MirInstruction::BoxCall {
    receiver: Value,
    method: String,
    args: Vec<Value>,
    // abi_hint不要！
}

// VM層：型判定でABI選択
fn execute_boxcall(...) {
    let type_id = receiver.get_type_id();
    
    // プラグインローダーから型情報取得
    if let Some(plugin_info) = get_plugin_info(type_id) {
        match plugin_info.abi {
            "c" => call_c_abi(...),
            "nyash" => call_nyash_abi(...),
            _ => fallback_to_c_abi(...),
        }
    }
}
```

### Option B: 明示的ヒント（将来の拡張性）

```rust
// MIR層：最小限の拡張
MirInstruction::BoxCall {
    receiver: Value,
    method: String,
    args: Vec<Value],
    metadata: Option<u32>, // 汎用メタデータ（ABIヒント含む）
}
```

## 📊 比較表

| 観点 | Option A（最小限） | Option B（ヒント付き） |
|------|------------------|---------------------|
| MIR変更 | なし ✅ | 最小限 |
| 実装工数 | 1週間 | 2週間 |
| JIT最適化 | VM時判定 | ビルド時判定可能 |
| 将来拡張 | 型情報経由 | メタデータ活用 |

## 🎯 結論：Option Aで始める

### 理由
1. **80/20ルール**: まず動くものを作る
2. **MIR無変更**: 15命令の美しさ維持 → さらに14命令へ（PluginInvoke統合）
3. **段階的進化**: 必要になったらOption Bへ

### Phase 12での追加統合
- **PluginInvoke → BoxCall 完全統合**
  - ビルトインBox廃止により区別が不要
  - VM層でのABI判定で十分
  - Core-15 → Core-14 への削減

### 実装ステップ（1週間）

```yaml
Day 1-2: nyash.toml拡張
  - abi: "c" | "nyash" field追加
  - プラグインローダー対応

Day 3-4: VM実行時分岐
  - execute_boxcall内でABI判定
  - NyashValue pack/unpack実装

Day 5-6: テスト・ベンチマーク
  - 同一機能の2種類プラグイン
  - 性能比較測定

Day 7: 判断
  - データに基づく方向性決定
  - Option B移行の必要性評価
```

## 💡 深い洞察

**「abi_hintは不要」という発見が示すもの**：
- 型システムが既に十分な情報を持っている
- MIRは純粋な意図の表現で良い
- 実行時の賢い判断に任せる

これこそ「Everything is Box」の真髄 - Boxが自分の実装方法を知っている！

## 🚀 次のアクション

1. nyash.tomlパーサーに`abi` field追加
2. プラグインローダーでABI情報保持
3. VM execute_boxcallで分岐実装
4. 最小限のテストプラグイン作成

---

*最小限から始めて、データで判断する。これがNyashの道。*