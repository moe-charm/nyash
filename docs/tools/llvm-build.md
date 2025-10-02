# LLVM Build Quick Guide (llvmlite harness)

Purpose
- Build and run the LLVM (llvmlite) harness line locally for AOT object emit and parity checks.

Requirements
- System LLVM 18 (`llvm-config-18`) on PATH
- Python 3 + `llvmlite` installed (`pip install llvmlite`)

Build steps
- Build harness compiler and core with LLVM feature:
```
cargo build --release -p nyash-llvm-compiler
cargo build --release --features llvm
```

Run examples
- Harness-first (emit object + run via harness):
```
NYASH_LLVM_USE_HARNESS=1 \
NYASH_NY_LLVM_COMPILER=target/release/ny-llvmc \
NYASH_EMIT_EXE_NYRT=target/release \
./target/release/hakorune --backend llvm apps/tests/phi_loop_simple.nyash
```
- Emit an object only:
```
NYASH_LLVM_USE_HARNESS=1 \
NYASH_LLVM_OBJ_OUT=$PWD/target/aot_objects/demo.o \
./target/release/hakorune --backend llvm apps/tests/phi_loop_simple.nyash
```

Notes
- If harness is unavailable, smoke scripts SKIP gracefully.
- Use `NYASH_LLVM_DUMP_IR=tmp/ir.ll` to dump the IR text for inspection.
- PHI safety: keep `NYASH_LLVM_SANITIZE_EMPTY_PHI=1` for development.

Harness‑First policy and env (Phase‑B)
- Harness‑first is既定（smokes は LLVM を常にハーネス経由で実行）
- Externs Registry JSON はランナーが自動出力し、`NYASH_EXTERN_SPEC_JSON` でハーネスに渡される
- 代表環境変数（開発用）
  - `NYASH_LLVM_USE_HARNESS=1`: ハーネス固定で実行
  - `NYASH_LLVM_DUMP_IR=tmp/file.ll`: 生成 IR のダンプ先
  - `NYASH_LLVM_SANITIZE_EMPTY_PHI=1`: 空 PHI のサニタイズ（開発向け）
  - `NYASH_LLVM_EXTERN_SYMBOL_STYLE={dotted|underscores}`: extern シンボル命名の選択（既定=dotted）
  - `NYASH_LLVM_UNKNOWN_EXTERN_FALLBACK=1`: 未知 extern を void() 宣言で強行（開発時のみ）
  - `NYASH_MIR_JSON_SKIP_VALIDATOR=1`: MIR→JSON バリデータを一時無効化（デバッグ用）

Externs naming（仕様）
- 既定は dotted 形式（例: `nyrt.time.now_ms`）。Kernel 側のエクスポート名と一致。
- underscores を選ぶ場合は `NYASH_LLVM_EXTERN_SYMBOL_STYLE=underscores` を明示。

MIR JSON バリデータ（Fail‑Fast）
- ランナーが JSON 書き出し前に必須フィールドを検証（unop/binop/compare/externcall/typeop/newbox/boxcall）。
- 欠落時は即失敗。緊急時のみ `NYASH_MIR_JSON_SKIP_VALIDATOR=1` で回避可能。
