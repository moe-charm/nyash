# value_gen vs ValueIdAllocatorBox: 二重管理の真相

**調査日**: 2025-10-17
**目的**: value_gen と ValueIdAllocatorBox の「二重管理」は本当に問題なのか？

---

## 📋 **Executive Summary**

### ✅ **結論: 二重管理は「意図的な疎結合設計」（問題なし）**

**設計意図**:
1. **Legacy経路（value_gen）**: 既存117箇所の `value_gen.next()` 呼び出しを保護
2. **新規安全経路（ValueIdAllocatorBox）**: 4-layer collision check 実装
3. **段階的移行**: ENV toggle で切り替え可能（Phase 2.P0→P3）

**競合は発生しない**:
- 両方とも `if offset > self.next_id` で保護
- safe_next_value() が排他的に選択（ENV依存）
- 同時に使われることはない

---

## 🔍 **アーキテクチャ詳細**

### **1. value_gen（Legacy経路）**

**実装**: `src/mir/value_id.rs:65-100`

```rust
pub struct ValueIdGenerator {
    next_id: u32,
}

impl ValueIdGenerator {
    pub fn next(&mut self) -> ValueId {
        let id = ValueId(self.next_id);
        self.next_id += 1;
        id
    }

    pub fn set_start_offset(&mut self, offset: u32) {
        if offset > self.next_id {  // ✅ 保護条件
            self.next_id = offset;
        }
    }
}
```

**特徴**:
- シンプルな increment のみ
- collision check なし
- 117箇所で直接呼び出し

---

### **2. ValueIdAllocatorBox（新規安全経路）**

**実装**: `src/mir/builder/value_allocator_box.rs:19-162`

```rust
pub struct ValueIdAllocatorBox {
    next_id: u32,
    in_use: HashSet<ValueId>,
    trace_enabled: bool,
}

impl ValueIdAllocatorBox {
    pub fn set_param_count(&mut self, param_count: u32) {
        if param_count > self.next_id {  // ✅ 保護条件
            self.next_id = param_count;
        }
    }

    pub fn allocate_safe(...) -> ValueId {
        self.sync_all(...);  // 4-layer collision check

        loop {
            let candidate = ValueId::new(self.next_id);
            self.next_id += 1;

            if self.is_available(candidate) {
                return candidate;
            }
        }
    }
}
```

**特徴**:
- 4-layer collision check（params, varmap, types, ssa）
- in_use set で衝突回避
- ENV toggle で有効化

---

### **3. safe_next_value()（統合ポイント）**

**実装**: `src/mir/builder.rs:258-269`

```rust
pub fn safe_next_value(&mut self) -> ValueId {
    if let Some(ref mut allocator) = self.value_allocator {
        allocator.allocate_safe(...)  // ← ValueIdAllocatorBox
    } else {
        self.value_gen.next()  // ← Legacy
    }
}
```

**排他的選択**:
- `HAKO_USE_VALUE_ALLOCATOR_BOX=1` → ValueIdAllocatorBox
- それ以外 → value_gen（Legacy）
- 両方が同時に使われることはない ✅

---

## 🎯 **初期化タイミングの詳細**

### **lowering.rs での同期（`lowering.rs:55-63`）**

```rust
let param_count = f.params.len() as u32;

// 1️⃣ Legacy経路の next_id 更新
self.value_gen.set_start_offset(param_count);

// 2️⃣ 新規安全経路の next_id 更新
if let Some(ref mut allocator) = self.value_allocator {
    allocator.set_param_count(param_count);
}
```

**両方とも更新される理由**:
- ENV toggle で切り替わる可能性があるため
- 両方を同期しておくことで安全性確保

**競合しない理由**:
- 両方とも `if offset > self.next_id` で保護
- 同じ param_count 値で更新
- next_id は単調増加のみ（後退なし）

---

## 🚨 **真の問題: 117箇所の value_gen.next() 直接呼び出し**

### **問題の構造**

```
┌─────────────────────────────────────┐
│ MirBuilder                          │
├─────────────────────────────────────┤
│ value_gen: ValueIdGenerator         │ ← Legacy経路
│   └─ next_id: 2 (param_count設定済)│
│                                     │
│ value_allocator: ValueIdAllocatorBox│ ← 新規安全経路
│   └─ next_id: 2 (param_count設定済)│
└─────────────────────────────────────┘
          ↓                 ↓
    117箇所が直接        safe_next_value()
    value_gen.next()     が経由
    を呼び出し           (ENV依存)
          ↓                 ↓
    ❌ collision      ✅ 4-layer
       check なし        collision check
```

### **問題シナリオ**

```rust
// lowering.rs:55-63 で正しく設定
let param_count = 2;  // v%0, v%1 がパラメータ
self.value_gen.set_start_offset(2);  // next_id = 2
allocator.set_param_count(2);  // next_id = 2

// ✅ safe_next_value() を使う場合
let dst = self.safe_next_value();  // → v%2（正しい）

// ❌ value_gen.next() を直接呼ぶ場合（117箇所）
let dst = self.value_gen.next();  // → v%2（一見正しい）
// しかし、variable_map に v%2 が既にある場合:
// → 衝突発生！（collision check なし）
```

**根本原因**:
- value_gen.next() は単純 increment のみ
- variable_map/value_types との衝突をチェックしない
- 結果: パラメータレジスタ v%0-v%N を間接的に上書き

---

## 🔧 **同期メカニズムの詳細**

### **set_start_offset() の実装**

```rust
pub fn set_start_offset(&mut self, offset: u32) {
    if offset > self.next_id {
        self.next_id = offset;
    }
}
```

### **set_param_count() の実装**

```rust
pub fn set_param_count(&mut self, param_count: u32) {
    if param_count > self.next_id {
        self.next_id = param_count;
    }
}
```

**完全に同じロジック**:
- 両方とも `if offset > self.next_id` で保護
- next_id は単調増加のみ
- 後退は発生しない

**同期が保証される理由**:
1. 同じ param_count 値で呼ばれる
2. 同じ条件チェック
3. 同じタイミング（lowering.rs で連続実行）

---

## 📊 **実行パスの可視化**

### **Case 1: HAKO_USE_VALUE_ALLOCATOR_BOX=1（新規安全経路）**

```
lowering.rs:55-63
  ├─ value_gen.set_start_offset(2)  → next_id=2
  └─ allocator.set_param_count(2)   → next_id=2

safe_next_value()
  └─ allocator.allocate_safe()  ✅ 使用
      ├─ sync_all() (4-layer check)
      ├─ candidate = v%2
      ├─ is_available(v%2) → true
      └─ return v%2  ✅ 安全
```

### **Case 2: HAKO_USE_VALUE_ALLOCATOR_BOX=0（Legacy経路）**

```
lowering.rs:55-63
  ├─ value_gen.set_start_offset(2)  → next_id=2
  └─ allocator.set_param_count(2)   → next_id=2 (unused)

safe_next_value()
  └─ value_gen.next()  ✅ 使用
      ├─ next_id = 2
      ├─ return v%2
      └─ next_id = 3  ⚠️ collision check なし
```

### **Case 3: value_gen.next() 直接呼び出し（問題）**

```
lowering.rs:55-63
  ├─ value_gen.set_start_offset(2)  → next_id=2
  └─ allocator.set_param_count(2)   → next_id=2

builder_calls/build.rs:29
  value_gen.next()  ❌ 直接呼び出し
      ├─ next_id = 2
      ├─ return v%2
      └─ next_id = 3  ❌ collision check なし

  （もし variable_map に v%2 が既にある場合）
  → SSA 違反発生！
```

---

## 🧪 **検証実験**

### **実験1: 同期確認**

```bash
# トレースログで両方の next_id を確認
HAKO_TRACE_VALUE_ALLOC=1 HAKO_USE_VALUE_ALLOCATOR_BOX=1 \
./target/release/hakorune apps/examples/json_query/main.nyash
```

**期待される出力**:
```
[value-alloc] 🔧 set_param_count: 0 -> 2
[value-alloc] 🔄 sync_all: in_use=2 (params=2, ...)
[value-alloc] ✅ allocated v%2 (attempts=1, in_use=2)
```

### **実験2: 競合テスト**

```rust
// test_double_management.rs
#[test]
fn test_value_gen_and_allocator_sync() {
    let mut builder = MirBuilder::new();

    // 初期化: param_count=2
    builder.value_gen.set_start_offset(2);
    if let Some(ref mut a) = builder.value_allocator {
        a.set_param_count(2);
    }

    // 両方の next_id が 2 になっていることを確認
    assert_eq!(builder.value_gen.peek_next().0, 2);
    // ValueIdAllocatorBox の next_id は private なので間接確認
    let v1 = builder.safe_next_value();
    assert_eq!(v1.0, 2);
}
```

---

## 🎯 **結論: 二重管理は「疎結合による段階移行」**

### ✅ **設計は正しい**
1. value_gen と ValueIdAllocatorBox は排他的使用
2. 両方とも同じロジックで同期される
3. ENV toggle で切り替え可能

### 🚨 **真の問題**
- 117箇所の `value_gen.next()` 直接呼び出し
- これらが safe_next_value() を経由しない
- collision check なし

### 🔧 **修正方針**
1. **Phase 2.P2**: 117箇所を safe_next_value() に置換
2. **Phase 2.P3**: value_gen 削除、ValueIdAllocatorBox のみ使用
3. **Phase 2.P4**: safe_next_value() → next_value() リネーム

---

## 📚 **関連資料**

- [task4_set_param_count_timing_report.md](./task4_set_param_count_timing_report.md)
- [task4_quick_summary.md](./task4_quick_summary.md)
- `src/mir/builder/value_allocator_box.rs`
- `src/mir/value_id.rs`
- `src/mir/builder.rs`

---

**調査完了日**: 2025-10-17
**次のアクション**: Phase 2.P2（117箇所置換計画）
