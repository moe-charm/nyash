# Hako CLI Options Quick Reference

Primary binary is `hako`. Aliases `nyash`/`hrn` may be available. Environment variables accept `HAKO_*`/`HAKU_*`/`HRN_*` as aliases of `NYASH_*` (auto-mapped both ways).

最終更新: 2025-10-02

## 基本
- `file`: 実行するNyashファイル（位置引数）
- `--backend {interpreter|vm|llvm}`: 実行バックエンド選択（既定: interpreter）
- `--debug-fuel {N|unlimited}`: パーサーのデバッグ燃料（無限ループ対策）

## MIR関連
- `--dump-mir`: MIRを出力（実行はしない）
- `--verify`: MIR検証を実施
- `--mir-verbose`: 詳細MIR出力（統計など）
- `--emit-mir-json FILE`: MIR(JSON) を FILE に書き出して終了（バックエンド非依存・早期ゲート）
  - 補足: どの `--backend` でも有効。パース→MIRコンパイル→JSON出力→即終了。
  - 互換: v1（mir_call）経路はハーネス時に `NYASH_LLVM_DOWNGRADE_V1=1` で v0 へ降格可能（compile‑only）。

## VM関連
- `--vm-stats`: VM命令統計を有効化（`NYASH_VM_STATS=1`）
- `--vm-stats-json`: VM統計をJSONで出力（`NYASH_VM_STATS_JSON=1`）

## GC
- `--gc {auto|rc+cycle|minorgen|stw|rc|off}`: GCモード（既定: `auto` → rc+cycle）
  - `rc+cycle`: 参照カウント + 循環回収（推奨・安定）
  - `minorgen`: 高速向けの軽量世代別（Gen‑0移動、上位非移動）
  - `stw`: 検証用の非移動Mark‑Sweep（開発者向け）
  - `rc`: 循環回収なしのRC（比較用）
  - `off`: 自己責任モード（循環はリーク）
- 関連ENV
  - `NYASH_GC_MODE`（CLIが優先）
  - `NYASH_GC_METRICS` / `NYASH_GC_METRICS_JSON`
  - `NYASH_GC_LEAK_DIAG` / `NYASH_GC_ALLOC_THRESHOLD`
  - 詳細: `docs/reference/runtime/gc.md`

## WASM/AOT
- `--compile-wasm`: WATを出力
- `--compile-native` / `--aot`: AOT実行ファイル出力（要wasm-backend）
- `--output, -o FILE`: 出力先を指定

## ベンチマーク
- `--benchmark`: バックエンド比較ベンチを実行
- `--iterations N`: ベンチ実行回数（既定: 10）

## 使用例
```bash
# インタープリターで実行
hako program.nyash

# VMで実行 + 統計をJSON出力
hako --backend vm --vm-stats --vm-stats-json program.nyash

# MIRを出力
hako --dump-mir --mir-verbose program.nyash

# MIR(JSON)を書き出し（出力後に終了）
hako --emit-mir-json /tmp/out.json program.nyash

# ベンチマーク
hako --benchmark --iterations 100
```

詳細は `docs/reference/architecture/execution-backends.md` も参照してください。

## 参考: `hako --help` スナップショット
- docs/tools/nyash-help.md（互換）
