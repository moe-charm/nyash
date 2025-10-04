# Phase 23: Type System & Rune (型システムとルーン)

**ステータス**: 提案段階（Phase 15完了後に検討）
**優先度**: Phase 20-25期間中に実装
**前提条件**: Phase 15セルフホスティング完了

---

## 🎯 概要

Hakoruneの型システムを拡張し、**Rune-First設計**を導入する。

**Rune（ルーン）**: 箱に刻まれる能力の印。不変の契約として複数組み合わせ可能。

### 目的

1. **共通API保証**: すべてのBoxで共通の操作を提供
2. **安全ダウンキャスト**: 型判別と安全な型変換
3. **Capability表現**: 決定性・シリアライズ可能性の明示
4. **型パスへの橋渡し**: Phase 20-25の型推論・型検査との連携

---

## 📋 主要ドキュメント

### [rune-design.md](./rune-design.md)
Rune-First設計の完全仕様。他言語の事例研究、実装案、Phase 15との関係を詳述。

---

## 🚀 実装計画（3段階）

### Phase A: プレリュード関数（Phase 15完了直後）
- **実装コスト**: 50-100行、1日
- **内容**: `box_type_id()`, `try_as_string()`, `is_deterministic()`等の関数
- **ファイル**: `core/prelude/box_protocol.hako`

### Phase B: Rune構文（Phase 20-25）
- **実装コスト**: 200-300行、1週間
- **内容**: `@rune ValueLike { ... }`, `implements ValueLike for StringBox`
- **脱糖**: プレリュード関数を呼ぶ

### Phase C: @deriveマクロ（Phase 25+）
- **実装コスト**: 500-700行、2週間
- **内容**: `@derive(ValueLike, Debug, Hash)`
- **前提**: マクロシステム完成

---

## 📊 他言語の参考事例

| 言語 | 機能 | Hakoruneへの応用 |
|------|------|----------------|
| Rust | `Any` trait | 型判別＋ダウンキャスト |
| Swift | Protocol | 複数Rune組み合わせ |
| Elixir | Protocol | 型と実装の分離 |
| Haskell | Type Class | Runeを型制約として表現 |

---

## ⚠️ Phase 15との関係

**重要**: Phase 15中はRust VMを複雑にしない！

- ❌ Rust VMにRune実装
- ❌ Rust VMにtrait追加
- ✅ .hakoコードで実装
- ✅ Phase 15完了後に開始

---

## 🎊 Hakorune哲学との整合性

- ✅ **Everything is Box**: Boxの能力（Rune）として表現
- ✅ **Plugin-First**: プラグインもRune実装可能
- ✅ **Deterministic**: `is_deterministic()`で明示
- ✅ **De-sugaring**: MIR14に脱糖
- ✅ **Fail-Fast**: Option/Resultで安全失敗

---

## 🔗 関連Phase

- **Phase 15**: セルフホスティング（前提条件）
- **Phase 20**: Python統合（Capability連携）
- **Phase 21**: 最適化（型情報活用）
- **Phase 24-25**: 型推論・型検査（Rune制約）

---

**作成日**: 2025-10-03
**レビュー**: ChatGPT o1 + Claude Sonnet 4.5
