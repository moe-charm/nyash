# Nyash CLI Options Quick Reference

最終更新: 2025-08-23

## 基本
- `file`: 実行するNyashファイル（位置引数）
- `--backend {interpreter|vm|llvm}`: 実行バックエンド選択（既定: interpreter）
- `--debug-fuel {N|unlimited}`: パーサーのデバッグ燃料（無限ループ対策）

## MIR関連
- `--dump-mir`: MIRを出力（実行はしない）
- `--verify`: MIR検証を実施
- `--mir-verbose`: 詳細MIR出力（統計など）

## VM関連
- `--vm-stats`: VM命令統計を有効化（`NYASH_VM_STATS=1`）
- `--vm-stats-json`: VM統計をJSONで出力（`NYASH_VM_STATS_JSON=1`）

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
nyash program.nyash

# VMで実行 + 統計をJSON出力
nyash --backend vm --vm-stats --vm-stats-json program.nyash

# MIRを出力
nyash --dump-mir --mir-verbose program.nyash

# ベンチマーク
nyash --benchmark --iterations 100
```

詳細は `docs/reference/architecture/execution-backends.md` も参照してください。

## 参考: `nyash --help` スナップショット
- docs/tools/nyash-help.md
