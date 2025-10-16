# 箱化統合可能なレガシーコード調査報告

**調査日**: 2025-10-16
**調査目的**: Everything is Box 哲学に基づき、散らばったレガシーコードを箱化で整理できないか調査
**調査範囲**: `src/mir/builder/` 配下の全ファイル

---

## エグゼクティブサマリー

### 主要発見
1. ✅ **LegacyCallBridgeBox, OriginTrackerBox は既に箱化済み** - Everything is Box 哲学の成功事例
2. ⚠️ **`collect_free_vars` (149行) は完全なデッドコード** - 即時削除推奨
3. 💡 **`observe` module (181行) は箱化候補** - Dev-only機能の統一による保守性向上

### 推奨アクション
| アクション | 対象 | 行数 | 優先度 | 工数 |
|-----------|------|------|--------|------|
| 削除 | `src/mir/builder/vars.rs` | 149行 | High | 1時間 |
| 箱化 | `observe` module → `BuilderObserverBox` | 181行 | Medium | 2-3時間 |
| 現状維持 | `local_recv/arg/cond` 等 | ~60行 | Low | - |

---

## 詳細分析

### カテゴリA: VarTrackerBox（変数追跡系）

#### ファイル
- `src/mir/builder/vars.rs` (149行)

#### 内容
```rust
#[allow(dead_code)]
pub(super) fn collect_free_vars(
    node: &ASTNode,
    used: &mut HashSet<String>,
    locals: &mut HashSet<String>,
) {
    // 150行の再帰的AST走査コード
}
```

#### 問題点
- ✅ `#[allow(dead_code)]` アノテーションあり
- ✅ **呼び出し元0件確認済み**（全codebase検索済み）
- ✅ クロージャ変数キャプチャ用に設計されたが、実装されず放置
- ❌ `src/mir/builder.rs` で `mod vars;` として宣言されているが未使用

#### 推奨: ❌ **削除**（箱化不要）

**理由**:
1. 呼び出し元が完全に0件（デッドコード確定）
2. 将来的にもクロージャ実装で別アプローチが必要
3. 箱化するメリットなし

**実装手順**:
```bash
# Step 1: vars.rs 削除
rm src/mir/builder/vars.rs

# Step 2: builder.rs から mod 宣言削除
# src/mir/builder.rs の該当行を削除
# mod vars; // variables/scope helpers

# Step 3: ビルド確認
cargo build --release
```

**削減見込み**: **149行削除**

---

### カテゴリB: ObserverBox（観測・デバッグ系）

#### ファイル構成
```
src/mir/builder/observe/
├── mod.rs (11行)
├── resolve.rs (55行) - KPI記録機能
├── resolve_trace.rs - 解決トレース
├── ssa.rs - PHI/SSA デバッグ
├── common.rs - 共通ユーティリティ
└── varmap.rs - 変数マップ観測
合計: 181行
```

#### 現状分析

**使用状況**:
- 呼び出し元: 12件（builder内で使用中）
- 環境変数制御:
  - `NYASH_DEBUG_KPI_KNOWN=1` - KPI記録有効化
  - `NYASH_DEBUG_SAMPLE_EVERY=N` - N回ごとにKPI出力
- 目的: 開発時のmethod resolution KPI計測・トレース

**主要機能**:
```rust
// resolve.rs
static TOTAL_CHOOSE: AtomicUsize = AtomicUsize::new(0);
static KNOWN_CHOOSE: AtomicUsize = AtomicUsize::new(0);

fn record_kpi(meta: &serde_json::Value) {
    let total = TOTAL_CHOOSE.fetch_add(1, Ordering::Relaxed) + 1;
    let certainty = meta.get("certainty").and_then(|v| v.as_str()).unwrap_or("");
    if certainty == "Known" {
        KNOWN_CHOOSE.fetch_add(1, Ordering::Relaxed);
    }
    // 定期的にKnown率を出力
}

pub(crate) fn emit_try(builder: &MirBuilder, meta: serde_json::Value);
pub(crate) fn emit_choose(builder: &MirBuilder, meta: serde_json::Value);
```

#### 箱化案: `BuilderObserverBox`

**設計**:
```rust
// src/mir/builder/observe/observer_box.rs
pub struct BuilderObserverBox {
    kpi_enabled: bool,
    sample_every: usize,
    total_choose: AtomicUsize,
    known_choose: AtomicUsize,
}

impl BuilderObserverBox {
    pub fn new() -> Self {
        Self {
            kpi_enabled: std::env::var("NYASH_DEBUG_KPI_KNOWN").ok().as_deref() == Some("1"),
            sample_every: std::env::var("NYASH_DEBUG_SAMPLE_EVERY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            total_choose: AtomicUsize::new(0),
            known_choose: AtomicUsize::new(0),
        }
    }

    pub fn emit_try(&self, builder: &MirBuilder, meta: serde_json::Value) {
        let fn_name = builder.current_function.as_ref().map(|f| f.signature.name.as_str());
        let region = builder.debug_current_region_id();
        crate::debug::hub::emit("resolve", "try", fn_name, region.as_deref(), meta);
    }

    pub fn emit_choose(&self, builder: &MirBuilder, meta: serde_json::Value) {
        self.record_kpi(&meta);
        let fn_name = builder.current_function.as_ref().map(|f| f.signature.name.as_str());
        let region = builder.debug_current_region_id();
        crate::debug::hub::emit("resolve", "choose", fn_name, region.as_deref(), meta);
    }

    fn record_kpi(&self, meta: &serde_json::Value) {
        if !self.kpi_enabled { return; }
        let total = self.total_choose.fetch_add(1, Ordering::Relaxed) + 1;
        let certainty = meta.get("certainty").and_then(|v| v.as_str()).unwrap_or("");
        if certainty == "Known" {
            self.known_choose.fetch_add(1, Ordering::Relaxed);
        }
        if self.sample_every > 0 && total % self.sample_every == 0 {
            let known = self.known_choose.load(Ordering::Relaxed);
            let rate = if total > 0 { (known as f64) * 100.0 / (total as f64) } else { 0.0 };
            eprintln!("[NYASH-KPI] resolve.choose Known={} Total={} ({:.1}%)", known, total, rate);
        }
    }
}
```

#### 箱化メリット
1. **責務の明確化**: 観測・KPI記録機能を1つのBoxに統一
2. **環境変数制御の一元管理**: 初期化時に1回読み込み（パフォーマンス向上）
3. **テストのしやすさ**: Box単位でモックテスト可能
4. **拡張性**: 新しいKPI追加時にBox内で完結

#### 箱化コスト
- 実装工数: 2-3時間
- 既存コードへの影響: 小（呼び出し元12件の修正）
- テスト: 環境変数による動作確認のみ

#### 推奨: ✅ **箱化**（Medium優先度）

**理由**:
- Dev-only機能として明確な責務
- 散らばった観測機能を統一
- Everything is Box 哲学への適合

**実装手順**:
```bash
# Step 1: BuilderObserverBox 実装
# src/mir/builder/observe/observer_box.rs を作成

# Step 2: MirBuilder に observer フィールド追加
# src/mir/builder.rs
pub struct MirBuilder {
    observer: BuilderObserverBox,
    // ...
}

# Step 3: 呼び出し元12件を修正
# Before:
observe::resolve::emit_choose(builder, meta);
# After:
builder.observer.emit_choose(builder, meta);

# Step 4: 古い static 変数を削除
# resolve.rs の static TOTAL_CHOOSE/KNOWN_CHOOSE 削除
```

**削減見込み**: 直接的削減は少ないが、責務の明確化で長期的なメンテナンス性向上

---

### カテゴリC: LegacyCallBridgeBox（Call経路統合）

#### ファイル
- `src/mir/builder/calls/legacy_bridge/mod.rs` (310行)

#### 現状
✅ **既に箱化済み！**

```rust
pub struct LegacyCallBridgeBox<'a> {
    builder: &'a mut MirBuilder,
}

impl<'a> LegacyCallBridgeBox<'a> {
    pub fn new(builder: &'a mut MirBuilder) -> Self {
        Self { builder }
    }

    pub fn emit(mut self, dst: Option<ValueId>, target: CallTarget, args: Vec<ValueId>)
        -> Result<(), String>
    {
        // Legacy経路の統一実装（310行）
    }
}
```

#### 評価
- ✅ Everything is Box 哲学に完全準拠
- ✅ Legacy経路を1箇所に集約（Global/Method/Extern/Constructor）
- ✅ 責務が明確（deprecated paths consolidation）
- ✅ 使用中（`emit_legacy_call` 経由で多数呼び出し）
- ✅ Phase-in戦略明示: "prefer `emit_unified_call` for new code paths"

#### 設計パターン（参考価値高い）
```rust
// 使用例（emit.rs）
pub(in super::super) fn emit_legacy_call(
    &mut self,
    dst: Option<ValueId>,
    target: CallTarget,
    args: Vec<ValueId>,
) -> Result<(), String> {
    LegacyCallBridgeBox::new(self).emit(dst, target, args)
}
```

**利点**:
- Builder本体が肥大化しない
- Legacy経路の段階的廃止が容易
- テストが独立して実施可能

#### 推奨: ✅ **現状維持**（箱化完了）

---

### カテゴリD: OriginTrackerBox（Origin追跡系）

#### ファイル
- `src/mir/builder/origin/tracker.rs` (35行)

#### 現状
✅ **既に箱化済み！**

```rust
/// 薄い Origin 追跡箱。NYASH_ORIGIN_TRACE=1 でトレース。
pub struct OriginTrackerBox<'a> {
    map: &'a mut HashMap<ValueId, String>,
    trace: bool,
}

impl<'a> OriginTrackerBox<'a> {
    pub fn new(map: &'a mut HashMap<ValueId, String>, trace: bool) -> Self {
        Self { map, trace }
    }

    pub fn register_newbox<S: Into<String>>(&mut self, value_id: ValueId, class_name: S) {
        let cls = class_name.into();
        if self.trace {
            eprintln!("[OriginTracker] register v%{} = {}", value_id.0, cls);
        }
        self.map.insert(value_id, cls);
    }

    pub fn propagate(&mut self, from: ValueId, to: ValueId) {
        if let Some(origin) = self.map.get(&from).cloned() {
            if self.trace {
                eprintln!("[OriginTracker] propagate v%{} → v%{} ({})", from.0, to.0, origin);
            }
            self.map.insert(to, origin);
        }
    }

    pub fn get(&self, value_id: ValueId) -> Option<&str> {
        self.map.get(&value_id).map(|s| s.as_str())
    }
}
```

#### 評価
- ✅ Everything is Box 哲学に準拠
- ✅ **薄いBox設計**（軽量ラッパー、35行のみ）
- ✅ 環境変数制御: `NYASH_ORIGIN_TRACE=1`
- ✅ 使用中（NewBox時のOrigin記録、型推論に活用）
- ✅ 命名が明確（"薄い Origin 追跡箱"）

#### 推奨: ✅ **現状維持**（箱化完了）

---

### カテゴリE: BuilderUtilsBox（ユーティリティ系）

#### ファイル
- `src/mir/builder/utils.rs` (348行)

#### 対象関数
```rust
// LocalSSA convenience helpers（readability helpers）
#[allow(dead_code)]
pub(crate) fn local_recv(&mut self, v: ValueId) -> ValueId;
#[allow(dead_code)]
pub(crate) fn local_arg(&mut self, v: ValueId) -> ValueId;
#[allow(dead_code)]
pub(crate) fn local_field_base(&mut self, v: ValueId) -> ValueId;
#[allow(dead_code)]
pub(crate) fn local_cond(&mut self, v: ValueId) -> ValueId;

// WeakRef/Barrier helpers
#[allow(dead_code)]
pub(super) fn emit_weak_new(&mut self, box_val: ValueId) -> Result<ValueId, String>;
#[allow(dead_code)]
pub(super) fn emit_weak_load(&mut self, weak_ref: ValueId) -> Result<ValueId, String>;
#[allow(dead_code)]
pub(super) fn emit_barrier_read(&mut self, ptr: ValueId) -> Result<(), String>;
#[allow(dead_code)]
pub(super) fn emit_barrier_write(&mut self, ptr: ValueId) -> Result<(), String>;
```

#### 使用状況分析
**✅ 実際には使用中（20件以上の呼び出し）**:
- `local_recv`: 多数（`LegacyCallBridgeBox`, `emit.rs`, `build.rs`等）
- `local_arg`: 多数（`fields.rs`, `emit.rs`等）
- `local_field_base`: 2件（`fields.rs`）
- `local_cond`: 4件（`ops.rs`, `if_form.rs`）
- `emit_weak_new/load`: 2件（`fields.rs` - WeakRef操作）
- `emit_barrier_read/write`: 2件（`fields.rs` - メモリバリア）

**`#[allow(dead_code)]` の理由**:
- ❌ デッドコードではない
- ✅ コンパイラの誤検出防止用（inline関数のため）

#### 推奨: ❌ **箱化不要**（現状維持）

**理由**:
1. **実際には使用中** - 20件以上の呼び出し
2. **readability helpers** として設計 - 薄いインライン関数
3. **箱化コスト > メリット** - 単なる関数呼び出しのエイリアス
4. **既に独立モジュール化** - SSA Local系は `src/mir/builder/ssa/local.rs` で分離

**設計評価**: ✅ **既に適切に整理済み**

実装は `local.rs` で統一、`utils.rs` はインライン転送関数のみ：
```rust
// utils.rs - readability helpers
#[inline]
pub(crate) fn local_recv(&mut self, v: ValueId) -> ValueId {
    super::ssa::local::recv(self, v)
}

// ssa/local.rs - 実装
pub(crate) fn recv(builder: &mut super::MirBuilder, v: ValueId) -> ValueId {
    builder.local_ssa_ensure(v, 0)
}
```

---

## Everything is Box 哲学への適合度評価

### ✅ 成功事例（既に実現済み）

| Box名 | ファイル | 責務 | 評価 |
|-------|---------|------|------|
| **LegacyCallBridgeBox** | `calls/legacy_bridge/mod.rs` | Legacy経路統一 | ⭐⭐⭐⭐⭐ |
| **OriginTrackerBox** | `origin/tracker.rs` | Origin追跡 | ⭐⭐⭐⭐⭐ |
| **WeakFieldRegistryBox** | `weak_field_registry/mod.rs` | WeakField管理 | ⭐⭐⭐⭐⭐ |
| **FieldOriginRegistryBox** | `field_origin_registry/mod.rs` | Field Origin管理 | ⭐⭐⭐⭐⭐ |
| **EffectResolverBox** | `effects/resolver.rs` | 効果解決 | ⭐⭐⭐⭐⭐ |
| **CallRoutingBox** | `router/call_router.rs` | ルーティング | ⭐⭐⭐⭐⭐ |
| **MethodIndexBox** | `indexes/method_index.rs` | メソッド索引 | ⭐⭐⭐⭐⭐ |
| **BlockScheduleBox** | `schedule/block.rs` | ブロックスケジュール | ⭐⭐⭐⭐⭐ |
| **BirthPolicyBox** | `birth/policy.rs` | Birth方針 | ⭐⭐⭐⭐⭐ |
| **BirthCallEmitterBox** | `birth/emitter.rs` | Birth呼び出し | ⭐⭐⭐⭐⭐ |

**共通パターン**:
- ✅ 責務の明確化（1 Box = 1 Responsibility）
- ✅ 薄いBox設計（軽量ラッパー、状態最小限）
- ✅ 環境変数制御の集約（初期化時に読み込み）
- ✅ テストのしやすさ（Box単位で独立テスト）

### 📊 箱化可能な残存責務

| カテゴリ | 対象 | 行数 | 提案 | 優先度 |
|---------|------|------|------|--------|
| 変数追跡 | `collect_free_vars` | 149 | ❌ 削除 | High |
| 観測・KPI | `observe` module | 181 | ✅ 箱化 | Medium |

### 🎯 長期的なメリット

1. **責務の明確化**: 機能ごとにBoxで分離
   - 例: LegacyCallBridgeBox = Legacy経路のみ担当
   - 例: OriginTrackerBox = Origin追跡のみ担当

2. **テストのしやすさ**: Box単位でテスト可能
   - 独立したunit test作成が容易
   - モックテストの実装が簡単

3. **拡張性**: 新機能追加時にBox単位で実装
   - Builder本体が肥大化しない
   - 機能の追加・削除が容易

4. **保守性**: コードの散らばりを防ぐ
   - 1つの責務 = 1つのファイル
   - ドキュメント作成が容易

---

## 次のアクションプラン

### Phase 1: 即時削除（工数: 1時間）

**対象**: `src/mir/builder/vars.rs` (149行)

**手順**:
```bash
# Step 1: vars.rs 削除
rm src/mir/builder/vars.rs

# Step 2: builder.rs から mod 宣言削除
# src/mir/builder.rs:
# - mod vars; // variables/scope helpers

# Step 3: ビルド確認
cargo build --release

# Step 4: テスト実行
tools/smokes/v2/run.sh --profile quick

# Step 5: Commit
git add -A
git commit -m "refactor(builder): remove unused collect_free_vars (149 lines)

- Removed src/mir/builder/vars.rs (complete dead code)
- No callers found in entire codebase
- Cleanup for Phase 1-3 code reduction

Phase 1.x: Quick Wins - Dead Code Elimination"
```

**期待結果**: 149行削減、0エラー

---

### Phase 2: ObserverBox箱化（工数: 2-3時間）

**対象**: `src/mir/builder/observe/` (181行)

**手順**:

#### Step 1: BuilderObserverBox 実装（1時間）
```rust
// src/mir/builder/observe/observer_box.rs
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct BuilderObserverBox {
    kpi_enabled: bool,
    sample_every: usize,
    total_choose: AtomicUsize,
    known_choose: AtomicUsize,
}

impl BuilderObserverBox {
    pub fn new() -> Self {
        Self {
            kpi_enabled: std::env::var("NYASH_DEBUG_KPI_KNOWN").ok().as_deref() == Some("1"),
            sample_every: std::env::var("NYASH_DEBUG_SAMPLE_EVERY")
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(0),
            total_choose: AtomicUsize::new(0),
            known_choose: AtomicUsize::new(0),
        }
    }

    pub fn emit_try(&self, builder: &super::super::MirBuilder, meta: serde_json::Value) {
        let fn_name = builder.current_function.as_ref().map(|f| f.signature.name.as_str());
        let region = builder.debug_current_region_id();
        crate::debug::hub::emit("resolve", "try", fn_name, region.as_deref(), meta);
    }

    pub fn emit_choose(&self, builder: &super::super::MirBuilder, meta: serde_json::Value) {
        self.record_kpi(&meta);
        let fn_name = builder.current_function.as_ref().map(|f| f.signature.name.as_str());
        let region = builder.debug_current_region_id();
        crate::debug::hub::emit("resolve", "choose", fn_name, region.as_deref(), meta);
    }

    fn record_kpi(&self, meta: &serde_json::Value) {
        if !self.kpi_enabled { return; }
        let total = self.total_choose.fetch_add(1, Ordering::Relaxed) + 1;
        let certainty = meta.get("certainty").and_then(|v| v.as_str()).unwrap_or("");
        if certainty == "Known" {
            self.known_choose.fetch_add(1, Ordering::Relaxed);
        }
        if self.sample_every > 0 && total % self.sample_every == 0 {
            let known = self.known_choose.load(Ordering::Relaxed);
            let rate = if total > 0 { (known as f64) * 100.0 / (total as f64) } else { 0.0 };
            eprintln!("[NYASH-KPI] resolve.choose Known={} Total={} ({:.1}%)", known, total, rate);
        }
    }
}
```

#### Step 2: MirBuilder統合（30分）
```rust
// src/mir/builder.rs
pub struct MirBuilder {
    observer: BuilderObserverBox,
    // ... 既存フィールド
}

impl MirBuilder {
    pub fn new(...) -> Self {
        Self {
            observer: BuilderObserverBox::new(),
            // ... 既存初期化
        }
    }
}
```

#### Step 3: 呼び出し元修正（30分）
```bash
# 12件の呼び出し元を修正
# Before:
observe::resolve::emit_choose(builder, meta);
observe::resolve::emit_try(builder, meta);

# After:
builder.observer.emit_choose(builder, meta);
builder.observer.emit_try(builder, meta);
```

#### Step 4: 古いコード削除（30分）
```bash
# resolve.rs の static 変数削除
# - static TOTAL_CHOOSE: AtomicUsize
# - static KNOWN_CHOOSE: AtomicUsize
# - static KPI_ENABLED: OnceLock<bool>
# - static SAMPLE_EVERY: OnceLock<usize>

# テスト実行
NYASH_DEBUG_KPI_KNOWN=1 NYASH_DEBUG_SAMPLE_EVERY=10 \
  cargo build --release && tools/smokes/v2/run.sh --profile quick
```

#### Step 5: Commit
```bash
git add -A
git commit -m "refactor(builder): consolidate Observer into BuilderObserverBox

- Created BuilderObserverBox for KPI/trace observation
- Unified environment variable control (init-once pattern)
- Replaced 12 static function calls with Box methods
- Everything is Box philosophy: Dev-only features → Box

Phase 1.x: Box-based Code Organization"
```

**期待結果**: 責務の明確化、長期的なメンテナンス性向上

---

### Phase 3: ドキュメント整備（工数: 1時間）

**対象**: 箱化事例の記録

**作成ドキュメント**:

#### 1. `docs/development/architecture/box-examples.md`
```markdown
# Everything is Box 成功事例

## MirBuilder Box Pattern

### LegacyCallBridgeBox
- 責務: Legacy call経路の統一
- 設計: 薄いBox、310行
- メリット: 段階的廃止が容易

### OriginTrackerBox
- 責務: ValueId → Box名の追跡
- 設計: 薄いBox、35行
- メリット: 環境変数トレース統合

### BuilderObserverBox (NEW)
- 責務: Dev-only KPI記録・トレース
- 設計: 薄いBox、181行
- メリット: 環境変数制御の一元化

## 共通パターン
1. 薄いBox設計（状態最小限）
2. 環境変数制御の集約
3. 1 Box = 1 Responsibility
4. テスト容易性
```

#### 2. Phase 1-3 完了報告更新
- CURRENT_TASK.md に成果記録
- 149行削減（vars.rs）
- 責務明確化（BuilderObserverBox）

---

## 結論

### 成果
1. ✅ **149行の即時削除候補を特定** - `collect_free_vars` デッドコード
2. ✅ **181行の箱化候補を特定** - `observe` module → `BuilderObserverBox`
3. ✅ **既存の10個の成功事例を確認** - Everything is Box 哲学の実現

### Everything is Box 哲学の実現状況
- ✅ **LegacyCallBridgeBox, OriginTrackerBox 等は模範事例**
- ✅ **Builder内のほぼすべての責務がBox化済み**
- 💡 **残存する箱化候補は観測・デバッグ系のみ**

### 推奨優先順位
1. **High**: `collect_free_vars` 削除（1時間、149行削減）
2. **Medium**: `BuilderObserverBox` 箱化（2-3時間、責務明確化）
3. **Low**: ドキュメント整備（1時間）

**総工数見積もり**: 4-5時間
**総削減見込み**: 149行 + 責務の明確化による長期的メンテナンス性向上
