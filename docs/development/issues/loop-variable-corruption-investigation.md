# ループ変数破損バグ調査レポート

**調査日**: 2025-10-17
**調査方法**: 4 Task Agent並列調査
**調査範囲**: Rust VM側のループ処理不具合（ループ綺麗綺麗修正後の残存問題）

---

## 📋 Executive Summary

ループ処理の綺麗綺麗修正（ループヘッダ条件の変数マップ汚染、LocalSSA衝突）完了後、`json_query` テストで「String と Integer の比較ミスマッチ」エラーが残存。

4 Task Agent による並列調査の結果、**3つの独立した問題が絡み合っている**ことが判明：

1. **パラメータレジスタ上書きバグ** (P0 - 最重要) 🔥
2. **メソッド降下の不安定性** (P1)
3. **variable_map の ValueId 衝突** (P2)

---

## 🔥 問題1: パラメータレジスタ上書きバグ (Task 3発見)

### 症状

**テスト**: `apps/examples/json_query/main.nyash:348` の `skip_ws` 関数
**エラー**:
```
use of undefined value ValueId(38)
Type error: compare Lt on String("0") and Integer(1)
```

### 根本原因

**MIR Builder が関数パラメータレジスタ v%0-v%N をローカル変数で再利用している**

#### MIR 証拠 (bb122)

```mir
%19 = phi [%4, ...]     # %4 は Integer ループ変数 'j'
%23 = copy %17          # %17 は String パラメータ 's'
%4 = copy %23           # ❌ BUG: v%4 (元々 Integer j) を String で上書き！
%22 = call %4.substring(...)  # 型エラー: Integer レジスタで String メソッド呼び出し
```

#### 型破壊のメカニズム

1. **関数定義**: `skip_ws(s, i, end)` → パラメータが v%0(s), v%1(i), v%2(end) に割り当て
2. **ループ変数**: `local j = i` → **j が v%4 に割り当て** (Integerとして)
3. **メソッド呼び出し**: `s.substring(j, j+1)` の receiver materialization:
   - `ensure(s)` → v%17 (String)
   - **v%4 = copy v%17** ← 🔥 **j のレジスタを s で上書き！**
4. **次のループ**: `j < end` で v%4 を読むと String が返る → 型エラー

### 影響範囲

**すべての「パラメータを持つ関数でループ内にメソッド呼び出しがあるケース」で発生する可能性**

- `json_query` の `skip_ws`
- `json_query` の `parse_string`
- その他多数の関数（潜在的バグ）

### 生成ドキュメント

- `/docs/development/issues/task3_json_query_mir_analysis.md` - 詳細MIR解析
- `/tmp/json_query_mir.txt` - 完全MIRダンプ
- `/tmp/param_register_bug_minimal.hako` - 最小再現ケース

---

## ⚠️ 問題2: メソッド降下の不安定性 (Task 1発見)

### 症状

`s.substring(j, j+1)` が状況によって **BoxCall** と **Extern** の間で揺れる

### 根本原因

**Receiver の起源情報 (origin) の有無で降下経路が変わる**

#### 決定木

```
s.substring(j, j+1)
│
├─ origin_get(s) == "StringBox" ✅
│  └─ try_lower_via_table → Extern("nyrt.string.substring") [安定]
│
├─ value_types[s] == MirType::String ✅
│  └─ try_lower_via_table → Extern("nyrt.string.substring") [安定]
│
└─ origin 不明 ❌
   └─ infer_receiver → "UnknownBox"
      └─ choose_route → BoxCall → 実行時エラー [不安定]
```

### 重要な発見 (Task 4)

**String メソッドの Extern 正規化は既に完全実装済み！**

| メソッド | MIR Builder | Extern Adapter | Runtime Router | 状態 |
|---------|-------------|----------------|----------------|------|
| length/size/len | ✅ | ✅ | ✅ Slot 300 | 完全実装 |
| substring | ✅ | ✅ | ✅ Slot 301 | 完全実装 |
| indexOf/find | ✅ | ✅ | ✅ Slot 303 | 完全実装 |
| lastIndexOf | ✅ | ✅ | ✅ Slot 313 | 完全実装 |
| charAt | ✅ | ✅ | ✅ Slot 314 | 完全実装 |
| replace | ✅ | ✅ | ✅ Slot 304 | 完全実装 |

**問題は「origin 推論の失敗」であり、正規化処理自体の欠陥ではない**

### コード位置

| ファイル | 行番号 | 関数 | 役割 |
|---------|--------|------|------|
| `method_call_handlers.rs` | 106-229 | `handle_standard_method_call` | メソッド呼び出しの振り分け |
| `lowering/mod.rs` | 26 | (table) | `substring/2 → nyrt.string.substring` |
| `method_call_handlers.rs` | 169 | (inline) | origin マッチ → Early return |
| `infer/receiver.rs` | 9-61 | `infer_receiver` | Receiver クラス推論 |
| `router/policy.rs` | 16-80 | `choose_route` | Route::BoxCall vs Unified |

### 生成ドキュメント

- `/docs/development/analysis/method-routing-mechanism.md` - 完全調査レポート
- `/docs/development/analysis/method-routing-flowchart.md` - フローチャート

---

## 🎯 問題3: variable_map の ValueId 衝突 (Task 2発見)

### 症状

メソッド呼び出しの結果が既存のループ変数と同じ ValueId を割り当てられる

### 根本原因

**variable_map は単方向マッピング（変数名→ValueId）で、逆引きインデックスがない**

#### 衝突シーケンス

```
1. ループ変数 j が v%5 を使用
   variable_map["j"] = v%5

2. メソッド呼び出し s.substring(j, j+1) の結果も v%5 を割り当て
   dst = value_gen.next() → v%5

3. variable_map が上書きされる
   variable_map["j"] = v%5 (substring の結果) ← 🔥 上書き！

4. 次のループで j を読むと substring の結果が返る
```

### 修正済み箇所

**`src/mir/builder/ssa/local.rs:41-51`** - ensure() での衝突回避:

```rust
// 🎯 COLLISION AVOIDANCE: Never reuse source or parameter registers
if let Some(ref fun) = builder.current_function {
    while loc == v || fun.params.contains(&loc) {
        loc = builder.value_gen.next(); // Skip to next available
    }
} else {
    while loc == v {
        loc = builder.value_gen.next();
    }
}
```

### カバー範囲

**ensure() が呼ばれる箇所**:
- ✅ Receiver materialization (`recv()` calls)
- ✅ Argument materialization (`arg()` calls)
- ✅ Condition materialization (`cond()` calls)
- ✅ Compare operands (`cmp_operand()` calls)

**ensure() が呼ばれない箇所** (未対応):
- ❌ メソッド呼び出し結果の dst 割り当て
- ❌ Assignment RHS temporary values
- ❌ BinOp result allocation
- ❌ NewBox result allocation

### 未カバーの衝突パターン

#### Pattern A: Method Result → Loop Variable
```hakorune
local j = 0
loop(j < 10) {
  local temp = arr.get(j)  // If get() returns v%j, collision!
  j = j + 1
}
```

#### Pattern B: BinOp Result → Loop Variable
```hakorune
local i = 0
loop(i < n) {
  local sum = i + offset  // If sum allocated as v%i, collision!
  i = i + 1
}
```

---

## 🛠️ 修正方針: 3段階アプローチ

### Phase 1: 緊急パッチ（今週中）⚡

**目標**: `json_query` を復活させる
**優先度**: P0（最重要）

**実装箇所**: `src/mir/builder/var_tracker.rs`

```rust
impl VarTracker {
    pub fn new(param_count: usize) -> Self {
        Self {
            next_id: param_count, // v%(N+1) からスタート
            reserved_params: (0..param_count).map(|i| ValueId::new(i)).collect(),
        }
    }

    pub fn next_local(&mut self) -> ValueId {
        loop {
            let candidate = ValueId::new(self.next_id);
            self.next_id += 1;
            if !self.reserved_params.contains(&candidate) {
                return candidate;
            }
        }
    }
}
```

**成果物**:
- ✅ `json_query_vm` テスト復活
- ⚠️ 技術的負債（Phase 2で解消）

**所要時間**: 1-2時間

---

### Phase 2: Box化・正規化（来週）🔧

**目標**: 処理の共通化・正規化（箱理論に沿った実装）
**優先度**: P1

**実装内容**:

#### 1. ParameterGuardBox (100行)
```rust
pub struct ParameterGuardBox {
    param_count: usize,
    reserved_registers: HashSet<ValueId>,
}

impl ParameterGuardBox {
    pub fn new(param_count: usize) -> Self;
    pub fn is_parameter(&self, vid: ValueId) -> bool;
    pub fn validate_no_overwrite(&self, vid: ValueId) -> Result<(), String>;
    pub fn start_offset(&self) -> usize;
}
```

#### 2. ValueIdAllocatorBox (150行)
```rust
pub struct ValueIdAllocatorBox {
    next_id: usize,
    guard: ParameterGuardBox,
    in_use: HashSet<ValueId>,
}

impl ValueIdAllocatorBox {
    pub fn new(param_count: usize) -> Self;
    pub fn allocate_safe(&mut self) -> ValueId;
    pub fn sync_in_use(&mut self, variable_map: &HashMap<String, ValueId>);
}
```

#### 3. MirBuilder 統合 (50行)
```rust
pub struct MirBuilder {
    value_allocator: Option<ValueIdAllocatorBox>, // Box化
}

impl MirBuilder {
    fn safe_next_value(&mut self) -> ValueId {
        if let Some(ref mut allocator) = self.value_allocator {
            allocator.sync_in_use(&self.variable_map);
            allocator.allocate_safe()
        } else {
            self.value_gen.next() // Fallback
        }
    }
}
```

**環境変数フラグ**:
- `HAKO_USE_VALUE_ALLOCATOR_BOX=1` - Box化機能を有効化
- `HAKO_TRACE_VALUE_ALLOC=1` - ValueId割り当てトレース

**成果物**:
- ✅ 戻せる（フラグで旧動作に切り替え可能）
- ✅ テスト可能（Box単体テスト10個）
- ✅ 見える化（トレース機能）
- ✅ 共通化（1箇所で全ValueId割り当てを管理）

**所要時間**: 4-6時間

---

### Phase 3: Hakoruneスクリプト化（Phase 4 Todo）🚀

**目標**: Rust層凍結準備
**優先度**: P2（長期計画）

**実装内容**:

#### 1. parameter_guard_box.hako
```hakorune
box ParameterGuardBox {
    param_count: IntegerBox
    reserved: ArrayBox

    birth(param_count) { ... }
    is_parameter(vid) { ... }
    validate_no_overwrite(vid) { ... }
}
```

#### 2. value_id_allocator_box.hako
```hakorune
box ValueIdAllocatorBox {
    next_id: IntegerBox
    guard: ParameterGuardBox
    in_use: MapBox

    birth(param_count) { ... }
    allocate_safe() { ... }
    sync_in_use(variable_map) { ... }
}
```

**成果物**:
- ✅ Phase 4 Todo完了
- ✅ Hakoruneスクリプトがメイン開発に
- ✅ Rust層99.8%削減に貢献

**所要時間**: 2-3日

---

## 📊 修正の影響範囲予測

| 修正 | 影響テスト数 | リスク | 優先度 | 所要時間 |
|-----|------------|--------|--------|---------|
| **Phase 1: 緊急パッチ** | 50-100+ | 中 | P0 | 1-2時間 |
| **Phase 2: Box化** | 20-50 | 低 | P1 | 4-6時間 |
| **Phase 3: Hakorune化** | 全体 | 中 | P2 | 2-3日 |

---

## ✅ 次のステップ

1. **即座実施**: Phase 1（緊急パッチ） - `json_query` 復活
2. **来週実施**: Phase 2（Box化） - 箱理論に沿った正規化
3. **Phase 4 Todo**: Phase 3（Hakorune化） - Rust層凍結準備

---

## 📚 関連ドキュメント

### Task調査レポート
- **Task 1**: [method-routing-mechanism.md](../analysis/method-routing-mechanism.md) - メソッド降下経路
- **Task 2**: (本レポートに統合) - variable_map衝突メカニズム
- **Task 3**: [task3_json_query_mir_analysis.md](task3_json_query_mir_analysis.md) - json_query MIR解析
- **Task 4**: (本レポートに統合) - String Extern正規化の現状

### MIRダンプ・再現ケース
- `/tmp/json_query_mir.txt` - 完全MIRダンプ
- `/tmp/param_register_bug_minimal.hako` - 最小再現ケース

### 箱理論関連
- [Box理論ドキュメント](../../guides/box-theory.md) - Everything is Box
- [Phase 4 Todo](../roadmap/phases/phase-4/) - ParameterGuardBox Hakorune化

---

**調査完了日**: 2025-10-17
**調査者**: Claude Code + 4 Task Agents (並列調査)
**次のアクション**: Phase 1実装開始
