# LLVM/AOT Test Programs

このファイルはLLVMバックエンドのテスト用プログラムについて説明します。
これらは主にCIスモークテストで使用されます。

**注意**: これらのテストプログラムは `apps/tests/` ディレクトリに移動されました。

## テストプログラム一覧

### 基本動作テスト
- **ny-llvm-smoke/** - ArrayBox基本操作（push/get）+ print
  - 現在の問題: ArrayBoxプラグインの引数エンコーディング問題で "Invalid arguments"
  - 状態: NYASH_LLVM_ARRAY_SMOKE=0でデフォルトスキップ

- **ny-array-llvm-ret/** - ArrayBox戻り値テスト（printなし）
  - 目的: print依存を排除して安定性向上
  - 期待値: Result: 3

- **ny-echo-lite/** - 最小echo実装
  - 目的: 標準入力/出力の基本動作確認
  - 状態: NYASH_LLVM_ECHO_SMOKE=0でデフォルトスキップ

### プラグイン呼び出しテスト
- **ny-map-llvm-smoke/** - MapBoxプラグイン基本テスト
  - 目的: by-idプラグイン呼び出しの動作確認
  - 期待値: "Map: v=42" および "size=1"

### 可変長引数テスト（VInvoke）
- **ny-vinvoke-smoke/** - 5引数呼び出し（文字列出力）
  - 目的: 可変長引数（≥3）のtagged vector経路テスト
  - 期待値: "VInvokeRc: 42"

- **ny-vinvoke-llvm-ret/** - 5引数呼び出し（戻り値）✅
  - 目的: 可変長引数の戻り値検証
  - 期待値: Result: 42

- **ny-vinvoke-llvm-ret-size/** - 0引数呼び出し（size()）✅
  - 目的: 固定長引数（≤2）の経路テスト
  - 期待値: Result: 1

## CIスモークテストでの使用

`tools/llvm_smoke.sh`で以下の環境変数により制御：

```bash
# 基本テスト（問題があるためデフォルトOFF）
NYASH_LLVM_ARRAY_SMOKE=1    # ny-llvm-smoke
NYASH_LLVM_ARRAY_RET_SMOKE=1 # ny-array-llvm-ret
NYASH_LLVM_ECHO_SMOKE=1      # ny-echo-lite

# プラグインテスト（安定）
NYASH_LLVM_MAP_SMOKE=1       # ny-map-llvm-smoke
NYASH_LLVM_VINVOKE_SMOKE=1   # ny-vinvoke-smoke
NYASH_LLVM_VINVOKE_RET_SMOKE=1 # ny-vinvoke-llvm-ret + ny-vinvoke-llvm-ret-size
```

## 既知の問題

1. **ArrayBoxプラグイン**: set/getで "Invalid arguments" エラー
   - 原因: プラグイン側の引数デコード問題
   - 対策: 戻り値型テストで回避

2. **print文字列連結**: `print("Result: " + v)` での型エラー
   - 原因: binop型不一致
   - 対策: toString()を使用するか、戻り値型テストで回避

## 実アプリケーションとの違い

これらのテストプログラムは：
- 最小限の機能に絞った単体テスト
- CI自動実行を前提とした設計
- エラー時の切り分けが容易

実際のアプリケーション（chip8_nyash、kilo_nyash等）とは目的が異なります。