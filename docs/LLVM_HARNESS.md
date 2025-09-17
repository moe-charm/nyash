# llvmlite Harness（正式導入・Rust LLVM 対置運用）

Purpose
- Python + llvmlite による高速・柔軟な LLVM 生成経路を提供（検証・プロトタイプと将来の主役）。
- Rust/inkwell 経路と並走し、代表ケースで機能同値（戻り値・検証）を維持。

Switch
- `NYASH_LLVM_USE_HARNESS=1` でハーネス優先（LLVM バックエンド入口から起動）。

Tracing
- `NYASH_LLVM_TRACE_FINAL=1` を設定すると、代表コール（`Main.node_json/3`, `Main.esc_json/1`, `main` 等）を標準出力へ簡易トレースします。
  ON/OFF の最終 JSON 突合の補助に使用してください。

Protocol
- Input: MIR14 JSON（Rust 前段で Resolver/LoopForm 規約を満たした形）。
- Output: `.o` オブジェクト（既定: `NYASH_AOT_OBJECT_OUT` または `NYASH_LLVM_OBJ_OUT`）。
- 入口: `ny_main() -> i64`（戻り値は exit code 相当。必要時 handle 正規化を行う）。

CLI（crate）
- `crates/nyash-llvm-compiler` 提供の `ny-llvmc` は llvmlite ハーネスの薄ラッパーだよ。
  - ダミー: `./target/release/ny-llvmc --dummy --out /tmp/dummy.o`
  - JSON から: `./target/release/ny-llvmc --in mir.json --out out.o`
  - 既定のハーネスパスは `tools/llvmlite_harness.py`。変更は `--harness <path>` で上書き可。

Quick Start
- 依存: `python3 -m pip install llvmlite`
- ダミー生成（配線検証）:
  - `python3 tools/llvmlite_harness.py --out /tmp/dummy.o`
  - NyRT（libnyrt.a）とリンクして EXE 化（例: `cc /tmp/dummy.o -L target/release -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive -lpthread -ldl -lm -o app_dummy`）。

Wiring（Rust 側）
- `NYASH_LLVM_USE_HARNESS=1` のとき:
  1) `--emit-mir-json <path>` 等で MIR(JSON) を出力
  2) `python3 tools/llvmlite_harness.py --in <mir.json> --out <obj.o>` を起動
  3) 成功後は通常のリンク手順（NyRT とリンク）

Scope（Phase 15）
- 最小命令: Const/BinOp/Compare/Phi/Branch/Jump/Return
- 文字列: NyRT Shim（`nyash.string.len_h`, `charCodeAt_h`, `concat_hh`, `eq_hh`）を declare → call
- NewBox/ExternCall/BoxCall: まずは固定シンボル／by-id を優先（段階導入）
- 目標: `apps/selfhost/tools/dep_tree_min_string.nyash` の `.ll verify green → .o` 安定化

Acceptance
- Harness ON/OFF で機能同値（戻り値/検証）。代表ケースで `.ll verify green` と `.o` 生成成功。

Notes
- 初版は固定 `ny_main` から開始してもよい（配線確認）。以降、MIR 命令を順次対応。
- ハーネスは自律（外部状態に依存しない）。エラーは即 stderr に詳細を出す。

Schema Validation（任意）
- JSON v0 のスキーマは `docs/reference/mir/json_v0.schema.json` にあるよ。
- 検証: `python3 tools/validate_mir_json.py <mir.json>`（要: `python3 -m pip install jsonschema`）。

Appendix: 静的リンクについて
- 生成 EXE は NyRT（libnyrt.a）を静的リンク。完全静的（-static）は musl 推奨（dlopen 不可になるため動的プラグインは使用不可）。
