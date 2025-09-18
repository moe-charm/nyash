# Nyash ABI戦略議論まとめ (2025-09-01)

## 📋 概要

Phase 12のプラグインシステム実装において、C ABIとNyash ABIの選択について、AI先生方と深い技術議論を行いました。

## 🗂️ ドキュメント一覧

1. **[gemini-abi-analysis.md](gemini-abi-analysis.md)**
   - Gemini先生の長期エコシステム視点
   - ABI安定性の重要性
   - 段階的進化戦略（C ABI → SDK → WASM）

2. **[codex-abi-implementation.md](codex-abi-implementation.md)**
   - Codex先生の実装最適化視点
   - C呼出規約×Nyash値表現の提案
   - VM/JIT最適化の具体策

3. **[deep-analysis-synthesis.md](deep-analysis-synthesis.md)**
   - なぜ正解が難しいのかの深い分析
   - 時間軸のジレンマ
   - 「正解がない」ことが答え

## 🎯 結論：BoxCall拡張による統合案

### 最終的な実装方針

```rust
// MIRレベル：BoxCallをちょっと拡張
MirInstruction::BoxCall {
    receiver: Value,
    method: String,
    args: Vec<Value>,
    abi_hint: Option<AbiType>, // ← これだけ追加
}
```

### 利点

1. **MIR命令数は15個のまま**（美しさ維持）
2. **既存コードは変更不要**（後方互換）
3. **プラグインごとにABI選択可能**（段階的移行）
4. **Everything is Box哲学の体現**

### 実装計画

```yaml
Week 1: 基盤整備
  - PluginABI enum定義
  - nyash.tomlのabi field追加
  - NyashValue構造体作成

Week 2: VM統合
  - プラグインローダー拡張
  - VM実行時のABI分岐
  - pack/unpack実装

Week 3: 検証
  - 比較ベンチマーク
  - ドキュメント作成
  - 方向性判断
```

## 🔑 重要な洞察

1. **両ABI共存が現実的**
   - C ABI：既存資産・高速・安定
   - Nyash ABI：型安全・拡張性・将来性

2. **適応的戦略の採用**
   - 3ヶ月ごとに測定・評価
   - データに基づく進化

3. **箱理論による差し替え可能性**
   - 実装を箱に切り出す
   - いつでも戻せる安心感

## 📊 次のステップ

1. このREADMEを起点に実装開始
2. 最小限のプロトタイプ作成
3. 性能・開発体験の比較データ収集
4. Phase 12本実装への反映

---

*「間に挟むだけ」が最も難しい設計判断だった。しかし、BoxCall拡張という自然な解決策にたどり着いた。*