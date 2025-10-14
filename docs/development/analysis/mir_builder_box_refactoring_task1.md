# リファクタリング調査レポート Task 1: 箱化候補

## エグゼクティブサマリー

MirBuilder 内の状態管理を分析した結果、**5つの主要な Box 化候補**を特定しました。これらを Box 化することで、**推定 150-250 行の削減**、**テスト容易性の向上**、**デバッグ可能性の改善**が期待できます。

**優先順位**:
1. ✅ **LocalSSAStateBox** (既に部分実装済み) - 完成度80%
2. 🔥 **FieldOriginRegistryBox** - 即座に実装可能、重複ロジックが多い
3. 🔥 **MethodIndexBox** - 独立性が高く、テスト容易
4. 📦 **WeakFieldRegistryBox** - 小粒で安全
5. 🔄 **NormalizeStateBox** - 将来拡張の余地あり

---

## 1. MirBuilder 内の状態分析

### 1.1 現在の状態マップ（MirBuilder のフィールド一覧）

```rust
pub struct MirBuilder {
    // ===== SSA・ブロック管理 =====
    pub(super) local_ssa_map: HashMap<(BasicBlockId, ValueId, u8), ValueId>  // ✅ Box化候補1
    pub(super) schedule_mat_map: HashMap<(BasicBlockId, ValueId), ValueId>   // LocalSSAの一部

    // ===== Origin・メタデータ追跡 =====
    pub(super) value_origin_newbox: HashMap<ValueId, String>                 // ✅ 既にTrackerBox経由
    pub(super) value_types: HashMap<ValueId, MirType>                        // MetadataBox候補

    // ===== フィールド起源追跡 =====
    pub(super) field_origin_class: HashMap<(ValueId, String), String>        // ✅ Box化候補2
    pub(super) field_origin_by_box: HashMap<(String, String), String>        // ✅ Box化候補2

    // ===== Weak・Property管理 =====
    pub(super) weak_fields_by_box: HashMap<String, HashSet<String>>          // ✅ Box化候補4
    pub(super) property_getters_by_box: HashMap<String, HashMap<String, PropertyKind>>  // ✅ Box化候補4

    // ===== メソッドインデックス =====
    pub(super) static_method_index: HashMap<String, Vec<(String, usize)>>    // ✅ Box化候補3
    pub(super) instance_method_index: HashSet<(String, String, usize)>       // ✅ Box化候補3
    pub(super) method_tail_index: HashMap<String, Vec<String>>               // ✅ Box化候補3
    pub(super) method_tail_index_source_len: usize                           // ↑の一部

    // ===== 制御フロー・スタック =====
    pub(super) loop_header_stack: Vec<BasicBlockId>                          // 軽量スタック（Box化不要）
    pub(super) loop_exit_stack: Vec<BasicBlockId>
    pub(super) if_merge_stack: Vec<BasicBlockId>

    // ===== その他（Box化対象外） =====
    pub(super) variable_map: HashMap<String, ValueId>                        // 基本構造
    pub(super) user_defined_boxes: HashSet<String>                           // 設定類
    pub(super) static_box_names: HashSet<String>
    plugin_method_sigs: HashMap<(String, String), MirType>                   // 読み取り専用
    // ... その他の軽量フィールド ...
}
```

### 1.2 使用頻度分析

| 状態 | 使用箇所 | 更新頻度 | 複雑度 |
|------|---------|---------|--------|
| **local_ssa_map** | ssa/local.rs (9回), utils.rs (2回) | 高 | 中 |
| **field_origin_*** | fields.rs (7回), decls.rs (1回) | 中 | 高 |
| **method_*_index** | builder.rs (8回), lifecycle.rs, rewrite/gate.rs | 中 | 高 |
| **weak_fields_by_box** | decls.rs (1回), fields.rs (間接) | 低 | 低 |
| **property_getters_by_box** | decls.rs (1回), fields.rs (間接) | 低 | 中 |

---

## 2. Box 化候補リスト

### 候補 1: LocalSSAStateBox ✅

#### **現在の実装**
```rust
// src/mir/builder.rs (L179-181)
pub(super) local_ssa_map: HashMap<(BasicBlockId, ValueId, u8), ValueId>
pub(super) schedule_mat_map: HashMap<(BasicBlockId, ValueId), ValueId>

// src/mir/builder/ssa/local.rs (L30-67)
pub fn ensure(builder: &mut MirBuilder, v: ValueId, kind: LocalKind) -> ValueId {
    // ... 47行の複雑なロジック ...
}
```

#### **Box 化後の設計**
```rust
box LocalSSAStateBox {
    // State
    cache: HashMap<(BasicBlockId, ValueId, LocalKind), ValueId>
    schedule_cache: HashMap<(BasicBlockId, ValueId), ValueId>

    // Core operations
    ensure(bb: BasicBlockId, value: ValueId, kind: LocalKind) -> ValueId
    clear()

    // Debug/Observability
    dump() -> String
    trace_enabled() -> bool

    // Specialized accessors (thin wrappers)
    recv(bb: BasicBlockId, v: ValueId) -> ValueId
    arg(bb: BasicBlockId, v: ValueId) -> ValueId
    cond(bb: BasicBlockId, v: ValueId) -> ValueId
    field_base(bb: BasicBlockId, v: ValueId) -> ValueId
    cmp_operand(bb: BasicBlockId, v: ValueId) -> ValueId
}
```

#### **実装状況**
- ✅ **既存コードの80%がBox化済み**: `ssa/local.rs` モジュールとして分離済み
- ⚠️ **残課題**: `MirBuilder` から `local_ssa_map` を分離し、独立した Box にする

#### **メリット**
1. **テスト容易性**: モックビルダーを使わずに LocalSSA 単体テスト可能
2. **可視化**: `dump()` でキャッシュ状態を表示可能（PHIデバッグで有効）
3. **ロールバック**: ブロック切り替え時のクリア処理を明示化
4. **独立性**: ビルダー本体から SSA ロジックを完全分離

#### **実装難易度**: 低（既存モジュールの薄いラッパー化）
#### **削減可能行数**: 20-30 行（重複 clear 処理の統合）

---

### 候補 2: FieldOriginRegistryBox 🔥 **最優先**

#### **現在の実装**
```rust
// src/mir/builder.rs (L109-111)
pub(super) field_origin_class: HashMap<(ValueId, String), String>
pub(super) field_origin_by_box: HashMap<(String, String), String>

// src/mir/builder/fields.rs (L68-76, L99, L190-194)
// 散在する重複ロジック:
// - field_origin_class への登録（3箇所）
// - field_origin_by_box への登録（2箇所）
// - 両方のマップを参照する推論ロジック（4箇所）
```

#### **Box 化後の設計**
```rust
box FieldOriginRegistryBox {
    // State
    value_field_origins: HashMap<(ValueId, String), String>  // (base_id, field) -> class
    box_field_origins: HashMap<(String, String), String>      // (base_box, field) -> class

    // Registration
    register_value_field(base: ValueId, field: String, origin_class: String)
    register_box_field(base_box: String, field: String, origin_class: String)

    // Lookup
    infer_field_origin(base: ValueId, field: String, base_box_hint: Option<String>) -> Option<String>

    // Debug
    dump_value_origins() -> String
    dump_box_origins() -> String
}
```

#### **使用例**
```rust
// Before (重複コード4箇所)
self.field_origin_class.insert((base_val.0, field_name.clone()), origin_class.clone());
if let Some(base_origin) = self.origin_get(base_val) {
    self.field_origin_by_box.insert((base_origin.to_string(), field_name.clone()), origin_class.clone());
}

// After (統一インターフェース)
field_registry.register_value_field(base_val, field_name.clone(), origin_class.clone());
```

#### **メリット**
1. **重複削減**: 4箇所の登録ロジックを1箇所に統合
2. **推論の一元化**: 2つのマップを横断する推論ロジックを Box 内に閉じ込め
3. **テスト容易性**: フィールド起源推論を独立してテスト可能
4. **可視化**: フィールド起源マップをダンプしてデバッグ容易化

#### **実装難易度**: 低（既存ロジックの抽出・統合のみ）
#### **削減可能行数**: 60-80 行（重複登録・推論ロジックの統合）

---

### 候補 3: MethodIndexBox 🔥

#### **現在の実装**
```rust
// src/mir/builder.rs (L121-128)
pub(super) static_method_index: HashMap<String, Vec<(String, usize)>>
pub(super) instance_method_index: HashSet<(String, String, usize)>
pub(super) method_tail_index: HashMap<String, Vec<String>>
pub(super) method_tail_index_source_len: usize

// src/mir/builder.rs (L304-349) - 46行のindex管理ロジック
fn rebuild_method_tail_index(&mut self) { /* ... */ }
fn ensure_method_tail_index(&mut self) { /* ... */ }
pub(super) fn method_candidates(&mut self, method: &str, arity: usize) -> Vec<String> { /* ... */ }
pub(super) fn method_candidates_tail<S: AsRef<str>>(&mut self, tail: S) -> Vec<String> { /* ... */ }
```

#### **Box 化後の設計**
```rust
box MethodIndexBox {
    // State
    static_methods: HashMap<String, Vec<(String, usize)>>      // name -> [(BoxName, arity)]
    instance_methods: HashSet<(String, String, usize)>         // (BoxName, method, arity)
    tail_index: HashMap<String, Vec<String>>                   // ".method/arity" -> [full_names]
    tail_index_source_len: usize

    // Registration (called during lowering)
    register_static_method(name: String, box_name: String, arity: usize)
    register_instance_method(box_name: String, method: String, arity: usize)

    // Lookup
    exists_instance_method(box_name: String, method: String, arity: usize) -> bool
    find_candidates(method: String, arity: usize) -> Vec<String>
    find_candidates_by_tail(tail: String) -> Vec<String>

    // Maintenance
    rebuild_tail_index(functions: &HashMap<String, MirFunction>)
    ensure_tail_index(functions: &HashMap<String, MirFunction>)

    // Debug
    dump_instance_methods() -> String
    dump_static_methods() -> String
}
```

#### **メリット**
1. **独立性**: メソッドインデックスはビルダー本体から完全に独立
2. **テスト容易性**: メソッド解決ロジックを単体テスト可能
3. **保守性**: インデックス再構築ロジックを一箇所に集約
4. **可視化**: メソッドインデックスをダンプしてデバッグ容易化

#### **実装難易度**: 中（インデックス再構築ロジックの移植）
#### **削減可能行数**: 40-50 行（既存の46行ロジックを Box 内に移動）

---

### 候補 4: WeakFieldRegistryBox 📦

#### **現在の実装**
```rust
// src/mir/builder.rs (L102-106)
pub(super) weak_fields_by_box: HashMap<String, HashSet<String>>
pub(super) property_getters_by_box: HashMap<String, HashMap<String, PropertyKind>>

// src/mir/builder/decls.rs (L87, L125) - 2箇所で登録
```

#### **Box 化後の設計**
```rust
box WeakFieldRegistryBox {
    // State
    weak_fields: HashMap<String, HashSet<String>>              // BoxName -> {field_names}
    property_getters: HashMap<String, HashMap<String, PropertyKind>>  // BoxName -> {prop -> kind}

    // Registration
    register_weak_fields(box_name: String, fields: HashSet<String>)
    register_property_getter(box_name: String, prop_name: String, kind: PropertyKind)

    // Lookup
    is_weak_field(box_name: String, field: String) -> bool
    get_property_kind(box_name: String, prop: String) -> Option<PropertyKind>

    // Debug
    dump_weak_fields() -> String
    dump_property_getters() -> String
}
```

#### **メリット**
1. **小粒で安全**: 使用箇所が少なく、影響範囲が限定的
2. **明確な責務**: Weak field と property getter の管理を一箇所に
3. **拡張性**: 将来の property 管理機能を追加しやすい

#### **実装難易度**: 低（シンプルなラッパー）
#### **削減可能行数**: 10-15 行（重複登録ロジックの統合）

---

### 候補 5: NormalizeStateBox 🔄 **将来候補**

#### **現在の実装**
```rust
// src/mir/builder/normalize/string_length.rs
pub fn normalize_string_length_call(builder: &mut MirBuilder, callee: &mut Callee, args: &mut Vec<ValueId>) -> bool

// src/mir/builder/normalize/array_length.rs
pub fn normalize_array_length_call(builder: &mut MirBuilder, callee: &mut Callee, args: &mut Vec<ValueId>) -> bool
```

#### **現状分析**
- ✅ **既にモジュール化済み**: `normalize/` ディレクトリで分離済み
- ⚠️ **状態なし**: 現在は Pure Function として実装（状態保持なし）
- 🔄 **Box化の必要性は低い**: 将来、正規化の統計情報を記録する場合に検討

#### **Box 化の動機（将来的）**
```rust
box NormalizeStateBox {
    // Statistics (dev-only)
    string_length_normalizations: usize
    array_length_normalizations: usize

    // Operations
    normalize_string_length(callee: &mut Callee, args: &mut Vec<ValueId>) -> bool
    normalize_array_length(callee: &mut Callee, args: &mut Vec<ValueId>) -> bool

    // Debug
    dump_stats() -> String
}
```

#### **実装難易度**: 低（既存関数の薄いラッパー）
#### **削減可能行数**: 0-10 行（現時点で重複なし）
#### **優先度**: 低（統計情報が必要になったら実装）

---

## 3. 優先順位付け

### 優先度マトリックス

| 候補 | 重複削減 | テスト容易性 | 独立性 | 実装難易度 | 総合スコア | 優先順位 |
|------|---------|-------------|--------|----------|----------|---------|
| **FieldOriginRegistryBox** | 60-80行 | ⭐⭐⭐ | ⭐⭐⭐ | 低 | ⭐⭐⭐⭐⭐ | **1位 🔥** |
| **MethodIndexBox** | 40-50行 | ⭐⭐⭐ | ⭐⭐⭐ | 中 | ⭐⭐⭐⭐ | **2位 🔥** |
| **LocalSSAStateBox** | 20-30行 | ⭐⭐⭐ | ⭐⭐ | 低 | ⭐⭐⭐ | **3位 ✅** |
| **WeakFieldRegistryBox** | 10-15行 | ⭐⭐ | ⭐⭐ | 低 | ⭐⭐ | **4位 📦** |
| **NormalizeStateBox** | 0-10行 | ⭐ | ⭐⭐⭐ | 低 | ⭐ | **5位 🔄** |

### 実装順序の推奨

#### **Phase 1: 即座に実装（Week 1-2）**
1. **FieldOriginRegistryBox** - 最大のROI（60-80行削減）
2. **WeakFieldRegistryBox** - 小粒で安全、Phase 1 のウォームアップに最適

#### **Phase 2: 中期実装（Week 3-4）**
3. **MethodIndexBox** - 独立性が高く、テストしやすい
4. **LocalSSAStateBox** - 既存モジュールの完成（既に80%完了）

#### **Phase 3: 将来検討（必要時）**
5. **NormalizeStateBox** - 統計情報が必要になったら実装

---

## 4. 実装ロードマップ

### Week 1: FieldOriginRegistryBox + WeakFieldRegistryBox

#### **目標**: 2つの小粒 Box を実装・テスト完了

#### **作業内容**
1. **Day 1-2: FieldOriginRegistryBox 実装**
   - `src/mir/builder/field_origin/mod.rs` 作成
   - 既存の `field_origin_class` / `field_origin_by_box` ロジックを移行
   - 統一 API 実装: `register_*`, `infer_field_origin`

2. **Day 3: WeakFieldRegistryBox 実装**
   - `src/mir/builder/weak_field/mod.rs` 作成
   - 既存の `weak_fields_by_box` / `property_getters_by_box` ロジックを移行

3. **Day 4-5: テスト・統合**
   - ユニットテスト作成（各 Box 単体）
   - 既存スモークテストで回帰確認
   - ドキュメント更新

#### **期待成果**
- ✅ 70-95 行削減
- ✅ 2つの Box が独立してテスト可能
- ✅ フィールド起源推論が明示的に

---

### Week 2: MethodIndexBox 実装

#### **目標**: メソッドインデックス管理を Box 化

#### **作業内容**
1. **Day 1-2: Box 基本実装**
   - `src/mir/builder/method_index/mod.rs` 作成
   - 既存の index 再構築ロジックを移行（46行）

2. **Day 3: API 統合**
   - `register_*` / `find_candidates` API 実装
   - `MirBuilder` から呼び出し箇所を更新

3. **Day 4-5: テスト・検証**
   - メソッド解決ロジックのユニットテスト
   - スモークテストで回帰確認

#### **期待成果**
- ✅ 40-50 行削減
- ✅ メソッドインデックスが独立してテスト可能
- ✅ インデックス再構築ロジックが明示的に

---

### Week 3: LocalSSAStateBox 完成

#### **目標**: 既存 `ssa/local.rs` を完全な Box に昇格

#### **作業内容**
1. **Day 1-2: Box 化**
   - `local_ssa_map` を `MirBuilder` から分離
   - `LocalSSAStateBox` を独立した構造体に

2. **Day 3: API 最終化**
   - `clear()` / `dump()` メソッド実装
   - トレース機能の統合

3. **Day 4-5: テスト・ドキュメント**
   - LocalSSA 単体テスト
   - PHI デバッグガイド更新

#### **期待成果**
- ✅ 20-30 行削減
- ✅ LocalSSA が完全に独立
- ✅ PHI デバッグが容易に

---

### Week 4: 統合・最終調整

#### **作業内容**
1. **統合テスト**: 全 Box が協調動作することを確認
2. **パフォーマンス測定**: Box 化による overhead 確認
3. **ドキュメント整備**: 各 Box の使い方ガイド作成
4. **クリーンアップ**: デッドコード削除、コメント整理

---

## 5. 削減可能行数見積もり

### 詳細内訳

| Box | 削減行数 | 内訳 |
|-----|---------|------|
| **FieldOriginRegistryBox** | 60-80 行 | 重複登録ロジック (4箇所 × 15行) |
| **MethodIndexBox** | 40-50 行 | インデックス管理ロジック統合 |
| **LocalSSAStateBox** | 20-30 行 | clear 処理統合、重複削除 |
| **WeakFieldRegistryBox** | 10-15 行 | 重複登録ロジック統合 |
| **NormalizeStateBox** | 0-10 行 | 将来的な統計情報記録 |
| **合計** | **130-185 行** | |

### 追加メリット（数値化困難）
- ✅ **テスト容易性**: 各 Box を単体テスト可能（テストコード 100-200 行追加予定）
- ✅ **可視化**: `dump()` メソッドでデバッグ容易化
- ✅ **保守性**: 責務が明確化され、変更影響範囲が限定的に

---

## 6. リスク評価

### 低リスク
- ✅ **WeakFieldRegistryBox**: 使用箇所2箇所のみ、影響範囲が限定的
- ✅ **FieldOriginRegistryBox**: 既存ロジックの抽出のみ、新機能なし

### 中リスク
- ⚠️ **MethodIndexBox**: インデックス再構築ロジックの移植に注意
- ⚠️ **LocalSSAStateBox**: 既存の `ssa/local.rs` との互換性維持

### 高リスク
- （なし）

### リスク軽減策
1. **段階的実装**: 1 Box ずつ実装・テスト・統合
2. **回帰テスト**: 既存スモークテストで毎回検証
3. **ロールバック可能**: 各 Box を独立したブランチで開発

---

## 7. 成功指標

### 定量指標
- ✅ **コード削減**: 130-185 行削減（目標: 150 行以上）
- ✅ **テストカバレッジ**: 各 Box で 80% 以上のカバレッジ
- ✅ **スモークテスト**: 全テスト PASS（0 regression）

### 定性指標
- ✅ **可読性**: 新規開発者が各 Box の責務を理解しやすい
- ✅ **デバッグ容易性**: `dump()` メソッドで状態確認が容易
- ✅ **保守性**: 変更時の影響範囲が Box 内に閉じる

---

## 8. まとめ

### 推奨アクション

#### **即座に開始（Week 1）**
1. ✅ **FieldOriginRegistryBox 実装** - 最大のROI（60-80行削減）
2. ✅ **WeakFieldRegistryBox 実装** - 小粒で安全なウォームアップ

#### **次のステップ（Week 2-3）**
3. ✅ **MethodIndexBox 実装** - 独立性が高く、テスト容易
4. ✅ **LocalSSAStateBox 完成** - 既存の80%を最終化

#### **将来検討（必要時）**
5. 🔄 **NormalizeStateBox** - 統計情報が必要になったら実装

### 期待効果
- **コード削減**: 130-185 行（保守性向上）
- **テスト容易性**: 各 Box を独立してテスト可能
- **デバッグ容易性**: `dump()` メソッドで状態可視化
- **保守性**: 責務が明確化され、変更影響範囲が限定的

### 箱理論4原則の実践
1. ✅ **箱にする**: 状態をすべて Box に閉じ込め
2. ✅ **境界を作る**: 各 Box が明確な責務を持つ
3. ✅ **戻せる**: 各 Box を独立してロールバック可能
4. ✅ **見える化**: `dump()` メソッドで状態を可視化

---

## 付録A: 実装テンプレート

### FieldOriginRegistryBox 実装例

```rust
// src/mir/builder/field_origin/mod.rs

use crate::mir::ValueId;
use std::collections::HashMap;

/// FieldOriginRegistryBox - フィールド起源の追跡・推論を一元化
pub struct FieldOriginRegistryBox {
    /// (base_id, field) -> origin_class
    value_field_origins: HashMap<(ValueId, String), String>,
    /// (base_box, field) -> origin_class
    box_field_origins: HashMap<(String, String), String>,
    trace_enabled: bool,
}

impl FieldOriginRegistryBox {
    pub fn new() -> Self {
        let trace_enabled = std::env::var("NYASH_FIELD_ORIGIN_TRACE")
            .ok()
            .as_deref() == Some("1");
        Self {
            value_field_origins: HashMap::new(),
            box_field_origins: HashMap::new(),
            trace_enabled,
        }
    }

    /// Register field origin for a specific value
    pub fn register_value_field(&mut self, base: ValueId, field: String, origin_class: String) {
        if self.trace_enabled {
            eprintln!("[field-origin] register value v%{}.{} = {}", base.0, field, origin_class);
        }
        self.value_field_origins.insert((base, field), origin_class);
    }

    /// Register field origin for a box type
    pub fn register_box_field(&mut self, base_box: String, field: String, origin_class: String) {
        if self.trace_enabled {
            eprintln!("[field-origin] register box {}.{} = {}", base_box, field, origin_class);
        }
        self.box_field_origins.insert((base_box, field), origin_class);
    }

    /// Infer field origin (value-specific > box-level)
    pub fn infer_field_origin(
        &self,
        base: ValueId,
        field: &str,
        base_box_hint: Option<&str>,
    ) -> Option<String> {
        // 1. Try value-specific origin
        if let Some(cls) = self.value_field_origins.get(&(base, field.to_string())) {
            return Some(cls.clone());
        }

        // 2. Try box-level origin
        if let Some(base_box) = base_box_hint {
            if let Some(cls) = self.box_field_origins.get(&(base_box.to_string(), field.to_string())) {
                return Some(cls.clone());
            }
        }

        None
    }

    /// Dump all origins (debug)
    pub fn dump_value_origins(&self) -> String {
        let mut lines = vec!["=== Value Field Origins ===".to_string()];
        for ((base, field), origin) in &self.value_field_origins {
            lines.push(format!("v%{}.{} = {}", base.0, field, origin));
        }
        lines.join("\n")
    }

    pub fn dump_box_origins(&self) -> String {
        let mut lines = vec!["=== Box Field Origins ===".to_string()];
        for ((base_box, field), origin) in &self.box_field_origins {
            lines.push(format!("{}.{} = {}", base_box, field, origin));
        }
        lines.join("\n")
    }
}
```

---

## 付録B: テストテンプレート

### FieldOriginRegistryBox ユニットテスト

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_field_origin() {
        let mut registry = FieldOriginRegistryBox::new();
        let base = ValueId(100);

        registry.register_value_field(base, "name".to_string(), "StringBox".to_string());

        let origin = registry.infer_field_origin(base, "name", None);
        assert_eq!(origin, Some("StringBox".to_string()));
    }

    #[test]
    fn test_box_field_origin_fallback() {
        let mut registry = FieldOriginRegistryBox::new();
        let base = ValueId(200);

        registry.register_box_field("PersonBox".to_string(), "name".to_string(), "StringBox".to_string());

        let origin = registry.infer_field_origin(base, "name", Some("PersonBox"));
        assert_eq!(origin, Some("StringBox".to_string()));
    }

    #[test]
    fn test_value_origin_priority() {
        let mut registry = FieldOriginRegistryBox::new();
        let base = ValueId(300);

        // Register both value-specific and box-level
        registry.register_value_field(base, "age".to_string(), "IntegerBox".to_string());
        registry.register_box_field("PersonBox".to_string(), "age".to_string(), "NumberBox".to_string());

        // Value-specific should have priority
        let origin = registry.infer_field_origin(base, "age", Some("PersonBox"));
        assert_eq!(origin, Some("IntegerBox".to_string()));
    }
}
```

---

## 変更履歴
- 2025-10-15: 初版作成（Task 1 調査完了）
