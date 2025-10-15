# Phase 20.8: GC + Rust Deprecation

**期間**: 6週間（2026-07-20 → 2026-08-30）
**ステータス**: 未開始
**前提条件**: Phase 20.7（Collections in Hakorune）完了

---

## 🎯 概要

Phase 20.8は「Pure Hakorune大作戦」の最終フェーズです。2つのサブフェーズで構成されます：

- **Phase E: GC v0** (Week 1-4) - Mark & Sweep ガベージコレクション実装
- **Phase F: Rust VM Deprecation** (Week 5-6) - Rust VM非推奨化と完全セルフホスト達成

このフェーズの完了により、以下を実現します：

```
Rust=floor, Hakorune=house ✅

Rust層: ≤ 100行（HostBridge API のみ）
Hakorune層: すべて（VM、コレクション、GC、コンパイラ）
```

---

## 🏗️ Phase E: GC v0 (Week 1-4)

### 実装内容

#### 1. Stop-the-world Mark & Sweep

最小限のGC実装：

- **Mark**: 到達可能オブジェクトをルートからトレース
- **Sweep**: 到達不可能オブジェクトを解放
- **Simple**: Generational GC、Incremental GCは実装しない

#### 2. GC Roots（ルート集合）

以下の3種類のルートから到達可能性を判定：

```hakorune
// 1. Stack frames（スタックフレームのローカル変数）
local obj = new MyBox()  // GC root

// 2. Global static boxes（グローバル静的Box）
static box GlobalState {
    config: ConfigBox  // GC root
}

// 3. HandleRegistry（C-ABI経由のハンドル）
// Rust側で保持されているハンドル
```

#### 3. GC Metrics（メトリクス収集）

GC動作の観測可能性を確保：

- **Allocation count**: 割り当てオブジェクト数
- **Survivor count**: 生存オブジェクト数
- **Sweep time**: GC実行時間
- **Handle count**: HandleRegistry内のハンドル数

#### 4. Observability（観測機能）

```bash
# 詳細GCログ出力
HAKO_GC_TRACE=1 ./hako program.hako

# 出力例:
# [GC] Mark phase: 1234 objects traced
# [GC] Sweep phase: 567 objects freed (45ms)
# [GC] Survivors: 667 objects (54% survival rate)

# プログラム終了時のGC統計
# [GC Stats] Total allocations: 10000
# [GC Stats] Total collections: 15
# [GC Stats] Total freed: 8500
# [GC Stats] Peak handles: 42
```

---

## 🚀 Phase F: Rust VM Compat Mode (Week 5-6)

### 実装内容

#### 1. Deprecate Rust VM（Rust VM非推奨化）

```bash
# デフォルト: Hakorune-VM
./hako program.hako  # Hakorune-VM使用

# 明示的指定
./hako --backend vm program.hako  # Hakorune-VM使用

# Rust-VMはopt-in（警告あり）
./hako --backend vm-rust program.hako
# Warning: Rust-VM is deprecated and will be removed in Phase 15.82.
#          Use Hakorune-VM (--backend vm) instead.
```

#### 2. Bit-Identical Verification（ビット完全一致検証）

セルフコンパイルチェーン：

```bash
# Hako₁: 既存のRust製コンパイラ
./hako_rust selfhost-compiler.hako -o hako_1

# Hako₂: Hako₁でビルドしたコンパイラ
./hako_1 selfhost-compiler.hako -o hako_2

# Hako₃: Hako₂でビルドしたコンパイラ
./hako_2 selfhost-compiler.hako -o hako_3

# 検証: Hako₁ == Hako₂ == Hako₃（バイト単位）
diff hako_1 hako_2  # Expected: 完全一致
diff hako_2 hako_3  # Expected: 完全一致
```

**CI統合**:

```yaml
# .github/workflows/self_compilation.yml
name: Self-Compilation Verification
on: [push, pull_request]
jobs:
  verify:
    runs-on: ubuntu-latest
    steps:
      - name: Build Hako₁ → Hako₂ → Hako₃
      - name: Verify bit-identical
        run: |
          diff hako_1 hako_2 || exit 1
          diff hako_2 hako_3 || exit 1
```

毎日CIで実行し、divergence（逸脱）を早期検出。

#### 3. Rust Layer Minimization（Rust層最小化）

最終的なRust層の役割：

```rust
// src/host_bridge.rs (~100 lines)

// HostBridge API (C-ABI)
#[no_mangle]
pub extern "C" fn Hako_RunScriptUtf8(
    script_ptr: *const u8,
    script_len: usize
) -> i32 { /* ... */ }

#[no_mangle]
pub extern "C" fn Hako_Retain(handle: usize) { /* ... */ }

#[no_mangle]
pub extern "C" fn Hako_Release(handle: usize) { /* ... */ }

#[no_mangle]
pub extern "C" fn Hako_ToUtf8(
    handle: usize,
    out_ptr: *mut *const u8,
    out_len: *mut usize
) { /* ... */ }

#[no_mangle]
pub extern "C" fn Hako_LastError(
    out_ptr: *mut *const u8,
    out_len: *mut usize
) { /* ... */ }
```

**それ以外のすべてはHakorune実装**:

```
Hakorune実装:
- VM (MiniVmBox)
- Parser (MirJsonBuilderBox)
- Collections (MapBox, ArrayBox)
- GC (GcBox)
- Standard library (StringBox, IntegerBox, etc.)
```

---

## ✅ 成功基準

### Phase E (GC v0)

- [ ] GC v0動作（メモリリークなし）
- [ ] Mark & Sweep正常動作
- [ ] GC Roots正しく検出（stack, global, handles）
- [ ] メトリクス収集機能動作
- [ ] `HAKO_GC_TRACE=1` で詳細ログ出力

### Phase F (Rust VM Deprecation)

- [ ] Hakorune-VMがデフォルトバックエンド
- [ ] Rust-VM非推奨（警告表示）
- [ ] Bit-identical self-compilation（Hako₁ == Hako₂ == Hako₃）
- [ ] CI daily verification動作
- [ ] Rust層 ≤ 100行（HostBridge APIのみ）

### 総合

- [ ] **True Self-Hosting**: Hakorune IS Hakorune
- [ ] すべてのスモークテストPASS（Hakorune-VM使用）
- [ ] ドキュメント完備（GC設計、HostBridge API spec）

---

## 🎯 マイルストーン

### Week 1-2: GC Mark Phase

- [ ] GC Roots実装（stack, global, handles）
- [ ] Mark algorithm実装
- [ ] 基本トレース機能

### Week 3-4: GC Sweep Phase & Metrics

- [ ] Sweep algorithm実装
- [ ] メトリクス収集機構
- [ ] `HAKO_GC_TRACE=1` 実装
- [ ] GC統合テスト

### Week 5: Rust VM Deprecation

- [ ] `--backend vm` デフォルト化
- [ ] `--backend vm-rust` 警告追加
- [ ] ドキュメント更新

### Week 6: Bit-Identical Verification

- [ ] Self-compilation chain実装
- [ ] CI verification追加
- [ ] Rust layer audit（≤ 100行確認）
- [ ] 最終ドキュメント更新

---

## 📊 実装戦略

### GC実装原則

1. **Simplicity First**: 最小限の実装（Generational/Incremental不要）
2. **Correctness**: メモリリーク完全排除
3. **Observability**: `HAKO_GC_TRACE=1` で動作可視化
4. **Fail-Fast**: 不正なGC状態は即座にpanic

### Rust VM Deprecation原則

1. **Backward Compatibility**: `--backend vm-rust` で継続使用可能
2. **Clear Warning**: 非推奨警告を明示
3. **Golden Tests**: Rust-VM vs Hakorune-VM parity維持
4. **Documentation**: 移行ガイド提供

---

## 🔗 関連ドキュメント

- **Phase 20.7 (Collections)**: [phase-20.7/README.md](../phase-20.7/README.md)
- **Phase 15.80 (VM Core)**: [phase-15.80/README.md](../phase-15.80/README.md)
- **Pure Hakorune Roadmap**: [phase-20.5/PURE_HAKORUNE_ROADMAP.md](../phase-20.5/PURE_HAKORUNE_ROADMAP.md)
- **HostBridge API**: [phase-20.5/HOSTBRIDGE_API_DESIGN.md](../phase-20.5/HOSTBRIDGE_API_DESIGN.md)

---

## 🎉 最終成果物

Phase 20.8完了後の状態：

```
Rust層（~100行）:
├── host_bridge.rs           # C-ABI functions
└── (end)

Hakorune層（すべて）:
├── apps/selfhost-compiler/  # Compiler
├── src/runtime/             # VM (MiniVmBox)
├── plugins/                 # Collections, GC, stdlib
└── (everything else)
```

**これにより達成**:

- **True Self-Hosting**: Hakorune IS Hakorune
- **Rust=floor, Hakorune=house**: Rust層最小化完了
- **完全自律**: メモリ管理もHakorune実装

---

**ステータス**: 未開始
**開始予定**: 2026-07-20
**完了予定**: 2026-08-30
**依存関係**: Phase 20.7 完了必須
