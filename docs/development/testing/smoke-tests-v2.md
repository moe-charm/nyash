# Smoke Tests v2 - Phase 15.5後のテストシステム

## 📋 概要

Phase 15.5でCore Box完全削除後のNyashテストシステム。すべてのBoxがプラグイン化されたため、新しいテスト体系を構築。

## 🚀 クイックスタート

```bash
# 基本実行（quick profile）
./tools/smokes/v2/run.sh --profile quick

# 統合テスト
./tools/smokes/v2/run.sh --profile integration

# 完全テスト
./tools/smokes/v2/run.sh --profile full
```

## 📊 現在の状況（2025-09-24）

### ✅ 動作確認済み
- **基本算術演算**: `10 + 25`, `100 - 42`, `7 * 6`, `84 / 2`
- **文字列リテラル**: `"Hello World"`
- **変数代入**: `local x = 42`
- **制御構文**: `if`, `loop`, `break`, `continue`

### ⚠️ 既知の問題

#### StringBox/IntegerBox プラグイン回帰（2025-09-24）
- **症状**: Phase 15.5でCore Box削除後、プラグイン版が正しく動作しない
  - `new StringBox("test")` → オブジェクト生成は成功（ハンドル返却）
  - `.toString()` → 空文字列を返す（データ保存失敗）
  - `.length()` → 0を返す（内部状態が空）
  - `.get()` → 空文字列を返す
- **IntegerBox**: 同様の問題（値の保存・取得が失敗）

#### 根本原因（Codex調査による）
- **TypeBox v2 resolveブランチの欠落**: birthおよびtoStringメソッドの解決パスが未実装
- **method_id衝突**: 0-3は予約済み（toString/type/equals/clone）だが、修正後も動作せず
- **プラグインインボケーション**: nyash_plugin_invokeは呼ばれているが、TLV形式の応答処理に問題

#### 緩和策
1. **基本機能テストに集中**: 算術演算、制御構文、文字列リテラルは正常動作
2. **他のプラグインBox使用**: FileBox、PathBox等は動作する可能性あり
3. **デバッグ用環境変数**: `NYASH_CLI_VERBOSE=1`で詳細ログ確認

## 🔧 テスト環境設定

### 重要な環境変数（開発時の補助）
```bash
# エントリ解決（既定ON: top-level main も許可されます。無効化したい場合のみ0を設定）
# export NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=0

# プラグイン設定（Phase 15.5以降は削除不可）
# NYASH_DISABLE_PLUGINS=1  # ❌ 使用不可（すべてプラグイン化済み）

# デバッグ用（任意）
NYASH_CLI_VERBOSE=1        # 詳細ログ出力
```

### プラグイン初期化メッセージ抑制
テスト実行時に以下のメッセージが表示されますが、grep -vで除外済み：
```
[FileBox] Plugin initialized
Net plugin: LOG_ON=false, LOG_PATH=net_plugin.log
```

## 📁 ディレクトリ構造

```
tools/smokes/v2/
├── run.sh                    # メインエントリポイント
├── README.md                 # ユーザーガイド
├── profiles/                 # テストプロファイル
│   ├── quick/                # 1-2分の高速テスト
│   │   ├── core/             # 言語基本機能
│   │   └── boxes/            # Box操作（現在一部動作せず）
│   ├── integration/          # 5-10分の統合テスト
│   └── full/                 # 15-30分の完全テスト
├── lib/                      # 共通ライブラリ
│   ├── test_runner.sh        # テスト実行器
│   ├── plugin_manager.sh     # プラグイン管理
│   ├── result_checker.sh     # 結果検証
│   └── preflight.sh          # 事前チェック
└── configs/                  # 環境設定
    ├── rust_vm_dynamic.conf  # Rust VM設定
    └── llvm_static.conf      # LLVM設定
```

## 🧪 テストの作成方法

### 基本テンプレート
```bash
#!/bin/bash
# test_name.sh - テストの説明

# 共通ライブラリ読み込み（必須）
source "$(dirname "$0")/../../../lib/test_runner.sh"
source "$(dirname "$0")/../../../lib/result_checker.sh"

# 環境チェック（必須）
require_env || exit 2

# プラグイン整合性チェック（必須）
preflight_plugins || exit 2

# テスト実装
test_example() {
    local output
    output=$(run_nyash_vm -c 'print(10 + 20)' 2>&1)
    check_exact "30" "$output" "example_test"
}

# テスト実行
run_test "example" test_example
```

## 🐛 トラブルシューティング

### プラグインが見つからない
```bash
# nyash.tomlのパス設定を確認
grep "path = " nyash.toml

# 正しいパス: plugins/*/lib*.so
# 間違ったパス: target/release/lib*.so
```

### StringBoxが動作しない
Phase 15.5でCore Box削除後、プラグイン実装が不完全。現在調査中。
回避策：基本的な算術演算や制御構文のテストに集中。

### プラグイン初期化メッセージが邪魔
`lib/test_runner.sh`で自動的にgrep -vで除外済み。

## 📝 今後の改善点

1. **StringBox/IntegerBoxプラグインの修正**
   - メソッド実装の確認と修正
   - v2プラグインAPIへの完全対応

2. **エラーメッセージの改善**
   - プラグインロード失敗時の明確なエラー表示
   - メソッド呼び出し失敗時の詳細情報

3. **パリティテスト強化**
   - Rust VM ↔ LLVM の出力一致確認
   - プラグイン動作の一貫性検証

## 🔗 関連ドキュメント

- [Phase 15.5 Core Box Unification](../roadmap/phases/phase-15/phase-15.5-core-box-unification.md)
- [Plugin System Reference](../../reference/plugin-system/)
- [PyVM Usage Guidelines](../../reference/pyvm-usage-guidelines.md)
