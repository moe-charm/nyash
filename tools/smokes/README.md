# HakoRune Smoke Tests v2 (aka Nyash) — Guide

Overview
- Entry: `tools/smokes/v2/run.sh` — unified runner for quick/integration/full.
- Profiles:
  - `quick` — fast developer checks.
  - `integration` — VM↔LLVM parity, basic stability.
  - `full` — comprehensive matrix.

## 🎯 Two Baselines (Runbook)

これから開発の基準となる2つのベースライン：

### 📦 VM ライン（Rust VM - 既定）

**用途**: 開発・デバッグ・検証用（高速・型安全）

```bash
# ビルド
cargo build --release

# 一括スモークテスト
tools/smokes/v2/run.sh --profile quick

# 個別スモークテスト
tools/smokes/v2/run.sh --profile quick --filter "<glob>"
# 例: --filter "core/json_query_min_vm.sh"

# 単発実行（参考）
./target/release/nyash --backend vm apps/APP/main.nyash
```

### ⚡ llvmlite ライン（LLVMハーネス）

**用途**: 本番・最適化・配布用（実証済み安定性）

**前提**: Python3 + llvmlite
```bash
pip install llvmlite  # 未導入の場合
```

**実行手順**:
```bash
# ビルド（LLVM_SYS_180_PREFIX不要！）
cargo build --release --features llvm

# 一括スモークテスト
tools/smokes/v2/run.sh --profile integration

# 個別スモークテスト
tools/smokes/v2/run.sh --profile integration --filter "<glob>"

# 単発実行
NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm apps/tests/peek_expr_block.nyash

# 有効化確認
./target/release/nyash --version | rg -i 'features.*llvm'
```

**💡 重要**: 両方のラインのテストが通ることで、MIR14統一アーキテクチャの品質を保証！

### 🔁 QuickでAST/LLVM系も実行したいとき

通常、`quick` は LLVM未ビルド時に AST/LLVM系テストを自動で SKIP します。
Quickでも実行したい場合は、先に LLVM 有効でビルドしてください：

```bash
LLVM_SYS_180_PREFIX=$(llvm-config-18 --prefix) cargo build --release --features llvm
tools/smokes/v2/run.sh --profile quick
```

テストランナーは LLVM 非対応時にヒントを出力します（buildコマンドの案内）。

Notes
- Using resolution: prefer hako.toml aliases (SSOT; compat: nyash.toml). Some tests may enable `NYASH_ALLOW_USING_FILE=1` internally for convenience.
- Plugin warnings are informational; smokes are designed to pass without dynamic plugins.
- Harness single-run may take longer due to link+exec; integration profile includes generous timeouts.

Dev Mode (defaults)
- In v2 smokes, the `quick` profile exports `NYASH_DEV=1` by default.
  - This enables CLI `--dev`-equivalent defaults inside Nyash:
    - AST using ON (SSOT + AST prelude merge)
    - Operator Boxes in observe mode (no adoption)
    - Minimal diagnostics; output parity is preserved
- You can also run manually with `nyash --dev script.nyash`.

Common commands
- Quick suite (auto `NYASH_DEV=1`):
  - `tools/smokes/v2/run.sh --profile quick`
- Focus JSON smokes:
  - `tools/opbox-json.sh` (Roundtrip/Nested, plugins disabled, generous timeout)
- One-off program (VM):
  - `target/release/nyash --backend vm --dev apps/APP/main.nyash`

Key env knobs
- `NYASH_DEV=1` — enable dev defaults (same effect as `--dev`).
- `SMOKES_DEFAULT_TIMEOUT` — per test timeout seconds (default 15 for quick).
- `SMOKES_PLUGIN_MODE=dynamic|static` — plugin mode for preflight (auto by default).
- `SMOKES_FORCE_CONFIG=rust_vm_dynamic|llvm_static` — force backend config.
- `SMOKES_NOTIFY_TAIL` — lines to show on failure tail (default 80).

Notes
- Dev defaults are designed to be non-intrusive: tests remain behavior‑compatible.
- To repro outside smokes, either pass `--dev` or export `NYASH_DEV=1`.

Heavy JSON probes
- Heavy JSON tests (nested/roundtrip/query_min) run a tiny parser probe first.
- The probe's stdout last non-empty line is trimmed and compared to `ok`.
- If not `ok`, the test is SKIP (parser unavailable), not FAIL. This avoids
  false negatives due to environment noise or optional dependencies.
