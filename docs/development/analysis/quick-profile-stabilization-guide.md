# Quick Profile 安定化ガイド

**目的**: quick profile の成功率を向上させるための推奨環境変数設定

---

## 📊 現状分析

**ベースライン** (2025-10-16):
- 総テスト数: 58
- 成功: 43 PASS (74.1%)
- 失敗: 15 FAIL
- 非決定的失敗: **0件** (すべて決定的)

**主な失敗原因**:
1. **Feature Flag 無効化** (2件): async_await, gc_mode_off
2. **構文エラー** (1件): phi_switch_case_int
3. **未実装機能** (残りの失敗)

---

## 🎯 推奨環境変数セット

### 基本セット（すべてのテストで推奨）

```bash
# クリーン出力
export NYASH_QUIET=1

# 詳細診断（デバッグ時のみ）
# export NYASH_CLI_VERBOSE=1

# プラグイン制御（問題がある場合のみ）
# export NYASH_DISABLE_PLUGINS=1
```

### デバッグセット（テスト失敗時）

```bash
# MIR 出力
export NYASH_DUMP_MIR=1

# VM トレース（特定命令のみ）
export HAKO_VM_TRACE="op=compare,binop,boxcall;regs=1"

# ExternCall トレース
export NYASH_EXTERN_TRACE=1

# スモークテスト詳細ログ
export SMOKES_DEV_LOG=1
```

### GC 安定化セット

```bash
# GC モード（デフォルトは counting）
export NYASH_GC_MODE=counting

# GC メトリクス（診断時のみ）
# export NYASH_GC_METRICS=1
# export NYASH_GC_TRACE=1

# GC 閾値調整（メモリ不足時）
# export NYASH_GC_ALLOC_THRESHOLD=1000000
```

### Async/Await セット（Future テスト用）

```bash
# Future 構文リライト
export NYASH_REWRITE_FUTURE=1

# Await タイムアウト（デフォルト 5000ms）
export HAKO_AWAIT_MAX_MS=10000

# 注意: legacy-boxes feature が必要
# cargo build --release --features legacy-boxes
```

---

## 🔧 ビルド設定推奨

### デフォルトビルド（推奨）
```bash
cargo build --release
```

**有効な機能**:
- ✅ VM インタープリタ
- ✅ プラグインシステム
- ✅ Host Anchors
- ❌ Legacy Boxes（Future サポートなし）

### Full 機能ビルド（async/await テスト用）
```bash
cargo build --release --features legacy-boxes
```

**有効な機能**:
- ✅ VM インタープリタ
- ✅ プラグインシステム
- ✅ Host Anchors
- ✅ Legacy Boxes（Future サポート）

### LLVM ビルド
```bash
cargo build --release --features llvm
```

---

## 📋 テストプロファイル別推奨設定

### Quick Profile（開発・デバッグ）
```bash
# 基本
export NYASH_QUIET=1

# 実行
tools/smokes/v2/run.sh --profile quick
```

**期待結果**: 43-45 PASS (74-78%)

### Integration Profile（本番・最適化）
```bash
# 基本
export NYASH_QUIET=1
export NYASH_LLVM_USE_HARNESS=1

# 実行
tools/smokes/v2/run.sh --profile integration
```

**期待結果**: 高成功率（LLVM バックエンド）

---

## 🐛 既知の問題と回避策

### 1. async_await / gc_mode_off 失敗

**原因**: `legacy-boxes` feature が無効

**回避策**:
```bash
# Option 1: Feature 有効化
cargo build --release --features legacy-boxes

# Option 2: テストをスキップ（推奨）
# → Task 3 完了後、テストスクリプトに SKIP ロジック追加予定
```

### 2. phi_switch_case_int 失敗

**原因**: 構文エラー（case 節のフォールスルー）

**回避策**:
```bash
# 修正待ち（Task 1 で対応予定）
# 一時的にテストをスキップ
```

### 3. プラグインロードエラー

**症状**: `libnyash_*.so not found`

**回避策**:
```bash
# プラグイン無効化
export NYASH_DISABLE_PLUGINS=1

# または
export LD_LIBRARY_PATH=./target/release:$LD_LIBRARY_PATH
```

### 4. Parser 無限ループ

**症状**: テストがハングする

**回避策**:
```bash
# デバッグ燃料制限
./target/release/hako --debug-fuel 1000 program.nyash

# または timeout 使用
timeout 30s ./target/release/hako program.nyash
```

---

## 📊 環境変数クイックリファレンス

### 優先度: 高（常に使う）
| 変数名 | 推奨値 | 用途 |
|--------|-------|------|
| `NYASH_QUIET=1` | 1 | 出力抑制 |

### 優先度: 中（デバッグ時）
| 変数名 | 推奨値 | 用途 |
|--------|-------|------|
| `NYASH_CLI_VERBOSE=1` | 1 | 詳細診断 |
| `NYASH_DUMP_MIR=1` | 1 | MIR 出力 |
| `SMOKES_DEV_LOG=1` | 1 | テスト詳細ログ |

### 優先度: 低（特定問題対応時）
| 変数名 | 推奨値 | 用途 |
|--------|-------|------|
| `NYASH_DISABLE_PLUGINS=1` | 1 | プラグイン無効化 |
| `HAKO_VM_TRACE` | "op=compare;regs=1" | VM トレース |
| `NYASH_GC_MODE` | "counting"/"off" | GC モード |
| `HAKO_AWAIT_MAX_MS` | 10000 | Await タイムアウト |

**完全リスト**: `docs/reference/environment-variables.md` 参照

---

## 🚀 実践例

### 例1: 通常のテスト実行
```bash
export NYASH_QUIET=1
tools/smokes/v2/run.sh --profile quick
```

### 例2: 特定テストのデバッグ
```bash
export NYASH_CLI_VERBOSE=1
export NYASH_DUMP_MIR=1
export SMOKES_DEV_LOG=1

tools/smokes/v2/profiles/quick/core/binop_mul.sh
```

### 例3: async_await テスト（legacy-boxes 有効時）
```bash
# ビルド
cargo build --release --features legacy-boxes

# 実行
export NYASH_REWRITE_FUTURE=1
export HAKO_AWAIT_MAX_MS=10000
tools/smokes/v2/profiles/quick/core/async_await.sh
```

### 例4: VM トレース付き実行
```bash
export HAKO_VM_TRACE="op=compare,binop,boxcall;regs=1;block=*"
./target/release/hakorune test.hkr
```

---

## 📚 関連ドキュメント

- **Task 3 調査レポート**: `docs/development/analysis/async-gc-determinism-report.md`
- **環境変数完全ガイド**: `docs/reference/environment-variables.md`
- **スモークテストガイド**: `tools/smokes/README.md`
- **デバッグガイド**: `docs/guides/smoke-test-debugging.md`

---

## ✅ チェックリスト

**テスト実行前**:
- [ ] `NYASH_QUIET=1` を設定
- [ ] プラグインビルド完了確認
- [ ] `cargo build --release` 完了確認

**デバッグ時**:
- [ ] `NYASH_CLI_VERBOSE=1` を追加
- [ ] `NYASH_DUMP_MIR=1` を追加
- [ ] エラーメッセージをファイルに保存

**async/await テスト時**:
- [ ] `cargo build --release --features legacy-boxes`
- [ ] `NYASH_REWRITE_FUTURE=1` を設定
- [ ] タイムアウトを 10秒以上に設定

---

**最終更新**: 2025-10-16
**関連 Task**: Task 3 (非決定要素調査)
