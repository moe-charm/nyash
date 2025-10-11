# HakoRune Benchmark Suite

このディレクトリには、HakoRune（Nyash）の性能測定用ベンチマークファイルが含まれています。

## 📊 ベンチマーク一覧

| ファイル | 説明 | 期待値 | 特徴 |
|---------|------|--------|------|
| `01_counter.nyash` | シンプルカウンター | 10 | loop PHI検証、基本的な演算 |
| `02_fibonacci.nyash` | フィボナッチ数列 | 89 | ループ、複数変数、代入 |
| `03_prime_check.nyash` | 素数判定 | 1 | 条件分岐、剰余演算 |

すべてのベンチマークは `print("Result: <値>")` 形式で結果を出力します。

---

## 🔧 ビルド方法

### 前提条件

- **Rust**: stable + cargo
- **Python**: 3.10–3.12 + llvmlite
  ```bash
  pip install -U llvmlite
  ```
- **C toolchain**: gcc/clang/ld

### ビルド手順

```bash
# 1. LLVM コンパイラのビルド
cargo build --release -p nyash-llvm-compiler

# 2. HakoRune（LLVM機能付き）のビルド
cargo build --release --features llvm

# 3. Kernel ライブラリのビルド
cargo build --release -p hako_kernel

# 4. ビルド確認
ls target/release/hako
ls target/release/ny-llvmc
ls target/release/libhako_kernel.a
```

**ビルド時間**: 標準ビルド ~1分、LLVMビルド ~3-5分

---

## 🚀 ベンチマーク実行方法

### 1. VM ベンチマーク（高速）

Rust VM専用の高速ベンチマークです。

```bash
bash tools/bench_vm.sh
```

**実行内容**:
- 各ベンチマーク5回実行
- 平均・最小・最大時間を計測
- 結果をJSON形式で保存（`tmp/bench_results/bench_vm_*.json`）

**出力例**:
```
📊 ベンチマーク: カウンター (01_counter.nyash)
  ✓ 結果: 10 (期待値: 10) OK
  ⏱  平均時間: 4ms
  📊 最小/最大: 4ms / 5ms
  📈 5回の実行: 4ms, 4ms, 5ms, 4ms, 4ms
```

### 2. 統合ベンチマーク（VM + LLVM + WASM）

VM、LLVM、WASM（未実装）を統合的にベンチマークします。

```bash
bash tools/bench_unified.sh
```

**実行内容**:
- 各ベンチマーク3回実行（VM、LLVM）
- 速度比較（VM vs LLVM）
- 結果をJSON形式で保存（`tmp/bench_results/bench_*.json`）

**出力例**:
```
📊 サマリー
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
ベンチマーク          VM (ms)    LLVM (ms)  速度比
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
カウンター            4ms        320ms      0.01x
フィボナッチ          6ms        350ms      0.02x
素数判定              8ms        380ms      0.02x
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

**注意**: LLVMは現在コンパイル時間が含まれるため、実行時間が長くなります。最適化は今後実施予定です。

---

## 🧪 単体実行方法

### VM実行

```bash
# 基本実行
NYASH_QUIET=1 ./target/release/hako apps/benchmarks/01_counter.nyash

# 時間計測
time NYASH_QUIET=1 ./target/release/hako apps/benchmarks/01_counter.nyash
```

### LLVM実行（llvmlite harness）

```bash
# One-shot実行（推奨）
env NYASH_QUIET=1 \
    NYASH_NYRT_SILENT_RESULT=1 \
    NYASH_LLVM_USE_HARNESS=1 \
    NYASH_NY_LLVM_COMPILER=target/release/ny-llvmc \
    NYASH_EMIT_EXE_NYRT=target/release \
    ./target/release/hako --backend llvm apps/benchmarks/01_counter.nyash

# 時間計測
time env NYASH_QUIET=1 ... (上記と同じ)
```

### WASM実行（未実装）

WASM実装は将来のフェーズで追加予定です。

---

## 📁 結果ファイル

ベンチマーク結果は `tmp/bench_results/` に保存されます。

```bash
tmp/bench_results/
├── bench_vm_20251002_183000.json   # VM専用ベンチマーク
└── bench_20251002_183100.json      # 統合ベンチマーク
```

**JSON形式例**:
```json
{
  "timestamp": "2025-10-02T18:30:00+09:00",
  "backend": "Rust VM",
  "benchmarks": {
    "01_counter.nyash": {
      "name": "カウンター",
      "expected": 10,
      "result": 10,
      "time_ms_avg": 4,
      "time_ms_min": 4,
      "time_ms_max": 5,
      "times": [4, 4, 5, 4, 4],
      "status": "PASS"
    }
  }
}
```

---

## 🔧 環境変数リファレンス

### 必須環境変数（LLVM実行時）

| 環境変数 | 用途 | 値 |
|---------|-----|---|
| `NYASH_LLVM_USE_HARNESS` | llvmliteハーネス有効化 | `1` |
| `NYASH_NY_LLVM_COMPILER` | ny-llvmcパス指定 | `target/release/ny-llvmc` |
| `NYASH_EMIT_EXE_NYRT` | ランタイムライブラリディレクトリ | `target/release` |

### オプション環境変数

| 環境変数 | 用途 | 値 |
|---------|-----|---|
| `NYASH_QUIET` | 非推奨メッセージ・デバッグ出力抑制 | `1` |
| `NYASH_NYRT_SILENT_RESULT` | ランタイム末尾出力を抑制 | `1` |
| `NYASH_CLI_VERBOSE` | 詳細診断ログ | `1` |

---

## 🐛 トラブルシューティング

### "LLVM backend not available"

```bash
# 解決策: LLVM機能付きビルド
cargo build --release --features llvm
```

### "ny-llvmc not found"

```bash
# 解決策: ny-llvmcをビルド
cargo build --release -p nyash-llvm-compiler

# パス指定
NYASH_NY_LLVM_COMPILER=$PWD/target/release/ny-llvmc
```

### "libhako_kernel.a not found"

```bash
# 解決策: カーネルライブラリをビルド
cargo build --release -p hako_kernel

# 確認
ls target/release/libhako_kernel.a
```

### LLVMが遅い

現在、LLVMは毎回コンパイルを行うため実行時間が長くなります。
将来の最適化で改善予定です。

---

## 📚 関連ドキュメント

- **LLVM Build Quickstart**: [CLAUDE.md](../../CLAUDE.md#-selfhost-phi修正統合完了-2025-10-02)
- **環境変数完全ガイド**: [CLAUDE.md](../../CLAUDE.md#-環境変数完全ガイド)
- **スモークテスト**: [tools/smokes/v2/README.md](../../tools/smokes/v2/README.md)

---

## 🎯 次のステップ

1. **LLVM実行時間最適化**: コンパイルキャッシュ・AOTコンパイル
2. **WASM実装**: WASMターゲット追加
3. **ベンチマーク拡張**: より複雑なアルゴリズム追加
4. **可視化**: グラフ・チャート生成

---

🌿 **wasm-development branch**: ベンチマークシステム v1.0
