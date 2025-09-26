# JSON v0中心化戦略

**将来のRust離脱・多言語実装を見据えた基盤変革**

## 🎯 戦略概要

### 現状の問題
```
現在: AST → Rust MIR → 各実行器(バラバラ)
問題: Rustに過度依存、実行器間でコード重複
```

### 理想の未来
```
理想: すべて → JSON v0 → 実行器(統一)
利点: 言語非依存、真実は1つ、デバッグ簡単
```

### 現実的なアプローチ
**段階的移行で既存機能を保護しながらJSON中心化**

## 📊 バージョン戦略

### 入出力の分離
```
json_in:v0  - Python parser → Rust（既存互換）
json_out:v1 - Rust → 実行器（統一Call対応）
```

### スキーマ進化
```json
{
  "ir_schema": "nyash-mir-json",
  "version": 1,
  "capabilities": [
    "unified_callee",
    "effect_mask",
    "call_flags"
  ]
}
```

## 🔄 段階移行計画

### Phase A: JSON出力統一
**目標**: 出力をJSON v1に統一
```
Rust MIR (内部正規形) → JSON v1 (交換形式) → 実行器
```

**実装箇所**:
- `mir_json_emit.rs`: 統一Call対応
- スキーマバージョニング実装
- 後方互換性維持

### Phase B: JSON中心化移行
**目標**: MIRをJSON v0のラッパー化
```
AST → JSON v0 → MIR Module(薄ラッパー) → JSON v1
```

**実装内容**:
- JSON→MIRリーダーの薄化
- HIR/名前解決情報のJSON化
- 型安全ビューの実装

### Phase C: 完全JSON化
**目標**: Rust MIR廃止準備
```
AST → JSON v0のみ（MIR Module廃止）
```

**最終形**:
- JSON v0が唯一の内部表現
- Rustは高性能実行用の型安全ビューのみ
- 多言語実装基盤完成

## 🏗️ アーキテクチャ設計

### 役割分担（Phase A-B）
```
Rust MIR: 内部正規形（最適化・検証・型安全）
JSON:     境界/交換フォーマット（実行器・保存・デバッグ）
```

### 最終形（Phase C）
```
JSON v0:      唯一の真実（保存・交換・デバッグ）
型安全ビュー: 高性能処理用（Rust/他言語で実装）
```

## 📋 統一CallのJSON化

### v1スキーマ例
```json
{
  "op": "call",
  "dst": 12,
  "callee": {
    "kind": "Method",
    "box_id": "StringBox",
    "method": "upper",
    "receiver": 7
  },
  "args": [42],
  "flags": ["tail_hint"],
  "effects": ["IO"]
}
```

### Calleeパターン
```json
// Global関数
{"kind": "Global", "id": "nyash.builtin.print"}

// Method呼び出し
{"kind": "Method", "box_id": "StringBox", "method": "len", "receiver": 3}

// Constructor
{"kind": "Constructor", "box_type": "ArrayBox"}

// Extern関数
{"kind": "Extern", "name": "nyash.console.log"}

// Value（動的）
{"kind": "Value", "value_id": 5}

// Closure
{"kind": "Closure", "params": ["x"], "captures": [{"name": "y", "id": 8}]}
```

## ⚠️ リスク管理

### 回避すべき落とし穴

**1. 型喪失**
- **問題**: 最適化・検証が弱くなる
- **対策**: MIR型を先に保持し、JSONは運搬役に

**2. 性能劣化**
- **問題**: 巨大関数のJSONパースがボトルネック
- **対策**: バイナリ表現（FlatBuffers等）への移行余地確保

**3. 互換地獄**
- **問題**: 一種類に固定すると後で詰む
- **対策**: `json_in`/`json_out`分離で進化速度分離

### 撤退戦略
各Phaseで前の状態に戻せる設計を維持

## 🧪 検証計画

### Phase A検証
- [ ] 統一Call全パターンのJSON化
- [ ] round-trip整合性（emit→read→emit）
- [ ] 後方互換性（`NYASH_MIR_UNIFIED_CALL=0`でv0出力）
- [ ] PyVM実行（`print(2)`等の最小ケース）

### Phase B検証
- [ ] JSON→MIRラッパーの型安全性
- [ ] HIR情報の完全保持
- [ ] パフォーマンス劣化なし

### Phase C検証
- [ ] 他言語実装の技術実証
- [ ] Rust MIR廃止の影響範囲確認
- [ ] セルフホスティング準備完了

## 🔗 関連技術

### フォーマット候補
- **JSON**: 人間可読、デバッグ簡単（Phase A-B）
- **MessagePack**: バイナリ効率化（Phase C候補）
- **FlatBuffers**: ゼロコピー高速化（将来候補）

### 互換性管理
```json
{
  "format": "json-v1",
  "compression": "none",
  "extensions": ["unified_call", "effect_tracking"]
}
```

---

**ChatGPT戦略の核心**:
> "急がず・壊さず・段階的に" JSON v0を中心とした基盤に移行

これにより、セルフホスティング成功と将来の多言語実装基盤が両立する。