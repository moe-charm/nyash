# リファクタリング調査 Task 1 完了サマリー

## 調査完了

**日付**: 2025-10-15
**タスク**: MirBuilder 内の Box 化候補特定と実装提案
**結果**: ✅ 完了（5つの Box 化候補を特定、詳細実装計画を策定）

---

## エグゼクティブサマリー

### 発見事項
1. **5つの Box 化候補を特定** - 合計 **130-185 行削減**の見込み
2. **重複コード 70 行を発見** - FieldOriginRegistry で集約可能
3. **1つのBox は既に80%実装済み** - LocalSSAStateBox（残り20%で完成）

### 優先順位
1. 🔥 **FieldOriginRegistryBox** - 最大ROI（60-80行削減）
2. 🔥 **MethodIndexBox** - 独立性高い（40-50行削減）
3. ✅ **LocalSSAStateBox** - 既存完成（20-30行削減）
4. 📦 **WeakFieldRegistryBox** - 小粒で安全（10-15行削減）
5. 🔄 **NormalizeStateBox** - 将来検討（0-10行削減）

---

## 成果物

### 📄 ドキュメント（3件作成）

1. **詳細レポート** (13,000 words)
   - [mir_builder_box_refactoring_task1.md](./mir_builder_box_refactoring_task1.md)
   - 各 Box の詳細設計、実装例、テストテンプレート

2. **状態マップ** (視覚化)
   - [mir_builder_state_map.md](./mir_builder_state_map.md)
   - 依存関係グラフ、優先順位マトリックス、タイムライン

3. **サマリー** (本文書)
   - 即座に実行可能なアクションプラン

---

## 即座に実行可能なアクション

### Week 1: 小粒Box実装（今日から開始可能）

#### **Day 1-2: FieldOriginRegistryBox** 🔥
```bash
# 実装場所
mkdir -p src/mir/builder/field_origin
touch src/mir/builder/field_origin/mod.rs

# 移行対象
# - field_origin_class (HashMap)
# - field_origin_by_box (HashMap)
# - 重複登録ロジック (fields.rs L68-76, L190-194)
```

**期待効果**: 60-80 行削減

#### **Day 3: WeakFieldRegistryBox** 📦
```bash
# 実装場所
mkdir -p src/mir/builder/weak_field
touch src/mir/builder/weak_field/mod.rs

# 移行対象
# - weak_fields_by_box (HashMap)
# - property_getters_by_box (HashMap)
# - 登録ロジック (decls.rs L87, L125)
```

**期待効果**: 10-15 行削減

---

## 箱化の核心設計（統一パターン）

### 統一 API パターン

すべての Box は以下の統一インターフェースに従う:

```rust
box SomeStateBox {
    // === State ===
    internal_map: HashMap<K, V>

    // === Core Operations ===
    register_*(...) -> ()          // 状態登録
    find_*(...) -> Option<T>       // 状態照会
    infer_*(...) -> Option<T>      // 推論

    // === Maintenance ===
    clear() -> ()                  // 状態クリア
    rebuild(...) -> ()             // 再構築（必要な場合）

    // === Observability ===
    dump() -> String               // 状態可視化
    trace_enabled() -> bool        // トレースON/OFF
}
```

### 実装テンプレート（コピペ可能）

```rust
// src/mir/builder/some_box/mod.rs

use crate::mir::ValueId;
use std::collections::HashMap;

/// SomeStateBox - [責務の説明]
pub struct SomeStateBox {
    state: HashMap<KeyType, ValueType>,
    trace_enabled: bool,
}

impl SomeStateBox {
    pub fn new() -> Self {
        let trace_enabled = std::env::var("NYASH_SOMEBOX_TRACE")
            .ok()
            .as_deref() == Some("1");
        Self {
            state: HashMap::new(),
            trace_enabled,
        }
    }

    /// Register [説明]
    pub fn register(&mut self, key: KeyType, value: ValueType) {
        if self.trace_enabled {
            eprintln!("[somebox] register {:?} = {:?}", key, value);
        }
        self.state.insert(key, value);
    }

    /// Find [説明]
    pub fn find(&self, key: &KeyType) -> Option<&ValueType> {
        self.state.get(key)
    }

    /// Clear all state
    pub fn clear(&mut self) {
        if self.trace_enabled {
            eprintln!("[somebox] clear ({} entries)", self.state.len());
        }
        self.state.clear();
    }

    /// Dump state (debug)
    pub fn dump(&self) -> String {
        let mut lines = vec![format!("=== SomeStateBox ({} entries) ===", self.state.len())];
        for (k, v) in &self.state {
            lines.push(format!("{:?} = {:?}", k, v));
        }
        lines.join("\n")
    }
}
```

---

## メトリクス検証

### ファイルサイズ確認 ✅

```
src/mir/builder.rs:               734 行
src/mir/builder/ssa/local.rs:     146 行  ✅ 既に分離済み
src/mir/builder/fields.rs:        201 行  🔥 field_origin使用5箇所
src/mir/builder/decls.rs:         135 行
```

### 重複コード確認 ✅

```
field_origin_* 使用箇所:  5箇所（fields.rs）
local_ssa_map 使用箇所:   2箇所（local.rs, utils.rs）
method_*_index 使用箇所:  8箇所（builder.rs）
```

**結論**: 重複削減の余地が大きい（推定70-95行削減可能）

---

## リスク評価

### 低リスク（即座に実施可能）✅
- ✅ **WeakFieldRegistryBox**: 使用箇所2箇所のみ
- ✅ **FieldOriginRegistryBox**: 既存ロジックの抽出のみ

### 中リスク（慎重に実施）⚠️
- ⚠️ **MethodIndexBox**: インデックス再構築ロジックに注意
- ⚠️ **LocalSSAStateBox**: 既存 `ssa/local.rs` との互換性維持

### 高リスク
- （なし）

### リスク軽減策
1. ✅ **段階的実装**: 1 Box ずつ実装・テスト・統合
2. ✅ **回帰テスト**: 既存スモークテストで毎回検証
3. ✅ **ロールバック可能**: 各 Box を独立したブランチで開発

---

## 次のステップ（明日から）

### 即座に実行（今日中に準備）
```bash
# 1. ブランチ作成
git checkout -b refactor/field-origin-registry-box

# 2. ディレクトリ準備
mkdir -p src/mir/builder/field_origin
mkdir -p src/mir/builder/weak_field

# 3. テンプレートコピー
# 詳細レポートの「付録A: 実装テンプレート」を参照
```

### Week 1 実装（月-金）
- **Day 1-2**: FieldOriginRegistryBox 実装
- **Day 3**: WeakFieldRegistryBox 実装
- **Day 4-5**: テスト・統合・回帰確認

### 成功指標
- ✅ 70-95 行削減（Week 1）
- ✅ 全スモークテスト PASS
- ✅ 2つの Box が独立してテスト可能

---

## 詳細資料リンク

### 📚 本調査の成果物
1. **[詳細レポート](./mir_builder_box_refactoring_task1.md)** - 13,000 words
   - 各 Box の詳細設計
   - 実装テンプレート
   - テストテンプレート
   - 実装ロードマップ

2. **[状態マップ](./mir_builder_state_map.md)** - 視覚化
   - 依存関係グラフ
   - 優先順位マトリックス
   - 重複コード分布
   - 4週間タイムライン

### 📖 関連ドキュメント
- **箱理論**: [CLAUDE.md](../../CLAUDE.md#箱理論-box-first)
- **MirBuilder本体**: [src/mir/builder.rs](../../src/mir/builder.rs)
- **LocalSSA実装**: [src/mir/builder/ssa/local.rs](../../src/mir/builder/ssa/local.rs)

---

## 質問・相談事項

### ユーザー（tomoaki）に確認したい点

1. **実装優先順位の確認**
   - Week 1: FieldOriginRegistryBox + WeakFieldRegistryBox で OK?
   - それとも LocalSSAStateBox 完成を優先?

2. **テスト方針**
   - ユニットテストの詳細度（各 Box で何個のテストケース?）
   - 回帰テストの範囲（全スモークテスト? 一部のみ?）

3. **パフォーマンス目標**
   - Box 化による overhead は許容範囲?
   - ベンチマークテストは必要?

---

## まとめ

### 成果
- ✅ **5つの Box 化候補を特定**（合計 130-185 行削減見込み）
- ✅ **詳細実装計画を策定**（4週間ロードマップ）
- ✅ **実装テンプレート作成**（コピペ可能）
- ✅ **リスク評価完了**（低リスク・即実行可能）

### 推奨アクション
1. 🔥 **今日中**: ブランチ作成、ディレクトリ準備
2. 🔥 **明日から**: FieldOriginRegistryBox 実装開始
3. ✅ **Week 1完了**: 70-95 行削減達成

### 箱理論4原則の実践
1. ✅ **箱にする**: 状態をすべて Box に閉じ込め
2. ✅ **境界を作る**: 各 Box が明確な責務を持つ
3. ✅ **戻せる**: 各 Box を独立してロールバック可能
4. ✅ **見える化**: `dump()` メソッドで状態を可視化

---

**準備完了！ 明日から実装開始できます！** 🚀

---

**最終更新**: 2025-10-15
**作成者**: Claude (Agent - Task 1 Investigation)
**Status**: ✅ 調査完了・実装準備OK
