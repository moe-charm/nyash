# Nyash Test Programs

このディレクトリには、CI/CDやデバッグ用のテストプログラムが含まれています。
実用的なアプリケーションは親ディレクトリ（`apps/`）にあります。

## テストプログラム一覧

### LLVMバックエンドテスト
- **ny-llvm-smoke/** - ArrayBox基本操作テスト
- **ny-array-llvm-ret/** - ArrayBox戻り値テスト
- **ny-echo-lite/** - 最小echo実装（I/Oテスト）
- **ny-map-llvm-smoke/** - MapBoxプラグインテスト
- **ny-vinvoke-smoke/** - 可変長引数（5引数）テスト
- **ny-vinvoke-llvm-ret/** - 可変長引数戻り値テスト
- **ny-vinvoke-llvm-ret-size/** - 固定長引数（size()）テスト

## 実行方法

これらのテストは主に `tools/llvm_smoke.sh` から実行されます：

```bash
# 環境変数でテストを有効化
NYASH_LLVM_MAP_SMOKE=1 ./tools/llvm_smoke.sh
NYASH_LLVM_VINVOKE_RET_SMOKE=1 ./tools/llvm_smoke.sh
```

## 注意事項

- これらは最小限の機能テストであり、実用的なアプリケーションではありません
- CIでの自動テストを前提に設計されています
- エラー時の切り分けが容易になるよう、各テストは単一の機能に焦点を当てています