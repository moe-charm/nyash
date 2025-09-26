# Phase 15.5段階移行計画

**セルフホスティング前の基盤革命・詳細実装ロードマップ**

## 🎯 全体戦略

### 基本原則
1. **急がない**: 一気にJSON化せず段階的に
2. **壊さない**: 既存機能の完全互換性維持
3. **戻れる**: 各段階で前状態への撤退可能

### 3段階アプローチ
```
Phase A (今) → Phase B (次) → Phase C (最終)
JSON出力統一   JSON中心移行   完全JSON化
```

## 📋 Phase A: JSON出力統一（今すぐ実装）

### 🎯 目標
**出力を統一Call対応JSON v1に**

### 📊 優先度1: mir_json_emit統一Call対応

#### 実装箇所
```rust
// src/runner/mir_json_emit.rs
I::Call { dst, func, callee, args, effects } => {
    if let Some(callee_info) = callee {
        // 統一Call v1形式でJSON出力
        emit_unified_call_json(callee_info, dst, args, effects)
    } else {
        // 旧形式（互換性維持）
        emit_legacy_call_json(func, dst, args)
    }
}
```

#### JSON v1スキーマ
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
  "flags": [],
  "effects": ["IO"]
}
```

#### 環境変数制御
```bash
# v1形式出力（統一Call対応）
NYASH_MIR_UNIFIED_CALL=1 ./target/release/nyash program.nyash

# v0形式出力（既存互換）
NYASH_MIR_UNIFIED_CALL=0 ./target/release/nyash program.nyash
```

### 📊 優先度2: スキーマ情報追加

#### ヘッダー情報
```json
{
  "ir_schema": "nyash-mir-json",
  "version": 1,
  "capabilities": [
    "unified_callee",
    "effect_mask",
    "call_flags"
  ],
  "generator": "nyash-rust-v0.1.0",
  "timestamp": "2025-09-24T...",
  "module": { ... }
}
```

### 📊 優先度3: Python側対応

#### instruction_lower.py更新
```python
def handle_json_v1(inst_data):
    version = inst_data.get("version", 0)
    if version >= 1:
        # v1の統一Call処理
        return lower_unified_call_v1(inst_data)
    else:
        # v0の個別処理（既存）
        return lower_legacy_calls_v0(inst_data)
```

### ✅ Phase A完了条件
- [ ] 統一Call 6パターンのJSON化完了
- [ ] Python LLVM/PyVMでv1形式実行成功
- [ ] `NYASH_MIR_UNIFIED_CALL=0`で既存互換性維持
- [ ] round-trip整合性テスト通過

---

## 📋 Phase B: JSON中心化移行（次段階）

### 🎯 目標
**MIR ModuleをJSON v0のラッパー化**

### 📊 優先度1: JSON→MIRリーダー薄化

#### 現状問題
```rust
// 現在: 重厚なMIR Module構造体
pub struct MirModule {
    functions: HashMap<String, MirFunction>,
    globals: HashMap<String, MirGlobal>,
    // ... 大量のRust固有構造
}
```

#### 理想形
```rust
// 将来: JSON v0の薄いラッパー
pub struct MirModule {
    json_data: JsonValue,  // 実データはJSON
    // 型安全アクセサのみ
}

impl MirModule {
    fn get_function(&self, name: &str) -> Option<MirFunction> {
        // JSON→型安全ビューの変換
    }
}
```

### 📊 優先度2: HIR情報のJSON化

#### 名前解決情報
```json
{
  "hir_metadata": {
    "bindings": [
      {"id": "var_1", "name": "count", "scope": "local"},
      {"id": "func_main", "name": "main", "scope": "global"}
    ],
    "function_signatures": [
      {"id": "func_main", "params": [], "return_type": "void"}
    ]
  }
}
```

### 📊 優先度3: 型情報保持

#### 型安全性維持
```rust
// JSON v0 + 型情報のハイブリッド
pub struct TypedMirModule {
    json: JsonValue,           // データ
    type_info: TypeRegistry,   // 型安全性
}
```

### ✅ Phase B完了条件
- [ ] JSON→MIRリーダーの薄化完了
- [ ] HIR情報の完全JSON化
- [ ] 型安全性・パフォーマンス維持
- [ ] 既存最適化の動作確認

---

## 📋 Phase C: 完全JSON化（最終段階）

### 🎯 目標
**JSON v0を唯一の真実に**

### 📊 優先度1: MIR Module廃止準備

#### 依存箇所の洗い出し
```bash
# MIR Module直接利用の箇所を特定
grep -r "MirModule" src/ --include="*.rs"
```

#### 移行計画
1. 最適化エンジン → JSON v0 + 型ビュー
2. VM実行器 → JSON v0直接読み
3. デバッガー → JSON v0直接表示

### 📊 優先度2: 多言語実装技術実証

#### Python実装PoC
```python
# PyNyash: Nyash in Python
class NyashInterpreter:
    def __init__(self, json_v0_data):
        self.program = json_v0_data

    def execute(self):
        # JSON v0を直接実行
        pass
```

#### JavaScript実装PoC
```javascript
// nyash.js: Nyash in Browser
class NyashVM {
    constructor(jsonV0) {
        this.program = jsonV0;
    }

    execute() {
        // JSON v0をブラウザで実行
    }
}
```

### 📊 優先度3: プリンターのJSON化

#### 現状維持vs移行判断
```
利点: 統一性、他言語での再利用
欠点: 型情報喪失、パフォーマンス

→ Phase Cで慎重に判断
```

### ✅ Phase C完了条件
- [ ] 他言語実装の技術実証完了
- [ ] Rust依存の段階的削除
- [ ] セルフホスティング基盤完成
- [ ] パフォーマンス劣化なし

---

## ⏰ タイムライン

### Phase A: 2-3週間
- Week 1: mir_json_emit統一Call実装
- Week 2: Python側対応・テスト
- Week 3: 統合テスト・安定化

### Phase B: 4-6週間
- Week 1-2: JSON→MIRリーダー薄化
- Week 3-4: HIR情報JSON化
- Week 5-6: 型安全性確保・テスト

### Phase C: 8-12週間
- Week 1-4: 依存箇所移行
- Week 5-8: 多言語実装PoC
- Week 9-12: 最終統合・テスト

### セルフホスティング準備完了
**Phase 15.5完了後、Phase 15本格開始**

---

## 🚨 クリティカルパス

### 絶対に守るべき順序
1. **Phase A完了まではRust MIR触らない**
2. **各段階で既存機能の完全テスト**
3. **撤退戦略の常時確保**

### 並行作業可能項目
- ドキュメント整備
- テストケース拡充
- Python側の準備作業

これにより、セルフホスティング成功の確実な基盤が構築される。