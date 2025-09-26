# Smokes v2 - 段階的スモークテストシステム

**Rust VM + LLVM 2本柱対応**の効率的スモークテストシステム

## 🚀 クイックスタート

```bash
# 開発時（1-2分）- 毎コミット推奨
./run.sh --profile quick

# 統合時（5-10分）- 毎日・重要PR
./run.sh --profile integration

# リリース前（15-30分）- マイルストーン
./run.sh --profile full
```

## 📋 プロファイル

| プロファイル | 実行時間 | 用途 | 対象 |
|------------|---------|------|------|
| **quick** | 1-2分 | 開発時高速チェック | 言語/コア機能（プラグイン非依存） |
| **integration** | 5-10分 | 基本パリティ確認 | VM↔LLVM整合性 |
| **full** | 15-30分 | 完全マトリックス | 全組み合わせテスト |
| **plugins** | 数十秒〜 | 任意の補助スイート | using.dylib 自動読み込みなど |

## 🎯 使用方法

### 基本実行
```bash
./run.sh --profile quick
./run.sh --profile plugins
./run.sh --profile integration --filter "plugins:*"
./run.sh --profile full --format json --jobs 4 --timeout 300
```

### オプション
```bash
--profile {quick|integration|full}  # 実行プロファイル
--filter "pattern"                  # テストフィルタ（例: "boxes:string"）
--format {text|json|junit}          # 出力形式
--jobs N                           # 並列実行数
--timeout SEC                      # タイムアウト（秒）
--verbose                          # 詳細出力
--dry-run                          # テスト一覧表示のみ
```

## 📁 ディレクトリ構造

```
tools/smokes/v2/
├── run.sh                    # 単一エントリポイント
├── README.md                 # このファイル
├── profiles/                 # テストプロファイル
│   ├── quick/                # 開発時高速テスト（1-2分）
│   │   ├── core/             # 言語・制御構文・演算
│   │   └── boxes/            # 各Boxの最小API
│   ├── integration/          # 統合テスト（5-10分）
│   │   ├── parity/           # VM↔LLVM・動的↔静的観点合わせ
│   │   └── plugins/          # プラグイン整合性
│   └── full/                 # 完全テスト（15-30分）
│       ├── matrix/           # 全組み合わせ実行
│       └── stress/           # 負荷・ストレステスト
│   └── plugins/              # プラグイン専用スイート（任意）
│       └── dylib_autoload.sh # using kind="dylib" 自動読み込みの動作確認（Fixture/Counter 等）
├── lib/                      # 共通ライブラリ（強制使用）
│   ├── test_runner.sh        # 中核実行器
│   ├── plugin_manager.sh     # プラグイン設定管理
│   ├── result_checker.sh     # 結果検証
│   └── preflight.sh          # 前処理・環境チェック
├── configs/                  # 環境プリセット
│   ├── rust_vm_dynamic.conf  # Rust VM + 動的プラグイン
│   ├── llvm_static.conf      # LLVM + 静的プラグイン
│   ├── auto_detect.conf      # 自動判別設定
│   └── matrix.conf           # マトリックステスト定義
└── artifacts/                # 実行結果・ログ
    └── smokes/<timestamp>/   # タイムスタンプ別結果
```

## 🔧 テスト作成規約

### 必須前処理
すべてのテストスクリプトは冒頭で以下を実行：

```bash
#!/bin/bash
# 共通ライブラリ読み込み（必須）
source "$(dirname "$0")/../../lib/test_runner.sh"

# 環境チェック（必須）
require_env || exit 2

# プラグイン整合性チェック（必須）
preflight_plugins || exit 2

# テスト実装
run_test "test_name" {
    # テスト内容
}
```

### ディレクトリ別ガイドライン

#### **quick/core** - 言語・制御構文・演算
- 基本的な言語機能のみ
- プラグインに依存しない機能
- 1テスト30秒以内

#### **quick/boxes** - 各Boxの最小API
- toString/length/concat等の基本API
- 1Box1ファイル原則
- エラーハンドリング確認

#### **integration/parity** - VM↔LLVM観点合わせ
- 同一スクリプトでRust VM vs LLVM実行
- 出力・性能・エラーメッセージの一致確認
- 動的vs静的プラグインの挙動差分確認

#### **full/matrix** - 全組み合わせテスト
- configs/matrix.confの定義に基づく
- Cartesian積実行
- 回帰テスト・互換性確認

## 📊 出力形式

### text（デフォルト）
```
✅ quick/core/basic_print.sh: PASS (0.2s)
❌ quick/boxes/stringbox.sh: FAIL (1.1s)
   Expected: "Hello"
   Actual: "Hell"
```

### json
```json
{
  "profile": "quick",
  "total": 15,
  "passed": 14,
  "failed": 1,
  "duration": 78.5,
  "tests": [...]
}
```

### junit（CI用）
```xml
<testsuite name="smokes_quick" tests="15" failures="1" time="78.5">
  <testcase name="basic_print" classname="quick.core" time="0.2"/>
  ...
</testsuite>
```

## 🚨 運用ルール

### PR/CI統合
```bash
# PR必須チェック
./run.sh --profile quick --format junit

# mainブランチ自動実行
./run.sh --profile integration --format junit

# リリース前・毎晩実行
./run.sh --profile full --format junit
```

### 失敗時の対応
1. **quick失敗**: PR承認停止
2. **integration失敗**: 即座修正・再実行
3. **full失敗**: リリース延期・根本原因調査

### flaky対策
- 自動リトライ: 2回
- ログ保存: `artifacts/smokes/<timestamp>/`
- タイムアウト: プロファイル別設定

### 追加ポリシー（テストの“積み”方針）
- Quick/Core: 目安 12〜16 本。意味論の軽量ガードのみ（< 0.5s/本）
  - 増やす基準: バグ/回帰が出たとき“最小再現”を1本追加
  - 既存と同型のバリエーションは増やさない（効果逓減を避ける）
- Integration/Parity: 目安 8〜10 本。代表構文の VM ↔ LLVM ハーネス一致
  - 増やす基準: LLVM 側の修正で差分が出る領域のみ 1 本追加
- Plugins: 1〜3 本/プラグイン。環境依存は必ず SKIP ガード
  - 例: FileBox 未ロード時は SKIP（エラーメッセージをマッチして回避）

### ノイズ抑止と共通フィルタ
実行出力のノイズは `lib/test_runner.sh` の `filter_noise` に集約して管理する。
新しいノイズが出たらフィルタへ追加し、各テスト個別の `grep -v` は増やさない。

### LLVM パリティ（Python ハーネス）
- Integration の `check_parity` は LLVM 実行時に `NYASH_LLVM_USE_HARNESS=1` を自動付与して llvmlite ハーネスで検証する。
- 使い方（例）:
  - `check_parity -c 'print("Hello")' "hello_parity"`
  - 同一コードを VM と LLVM で実行し、終了コードと整形後の標準出力を比較する。

## 💡 トラブルシューティング

### よくあるエラー

#### プラグイン読み込み失敗
```bash
# プラグイン整合性チェック
./lib/preflight.sh --validate-plugins

# プラグイン再ビルド
tools/plugin-tester/target/release/plugin-tester build-all
```

#### パリティテスト失敗
```bash
# 詳細diff確認
./run.sh --profile integration --filter "parity:*" --verbose

# 個別実行で原因特定
NYASH_CLI_VERBOSE=1 ./target/release/nyash test.nyash
NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend llvm test.nyash
```

#### タイムアウト
```bash
# タイムアウト延長
./run.sh --profile full --timeout 600

# 並列度調整
./run.sh --profile integration --jobs 1
```

## 📚 参考リンク

- [Phase 15 Roadmap](../../docs/development/roadmap/phases/phase-15/)
- [PyVM Usage Guidelines](../../docs/reference/pyvm-usage-guidelines.md)
- [Plugin System](../../docs/reference/plugin-system/)
- [2本柱実行方式](../../CLAUDE.md#2本柱実行方式)

---

## 🎯 Conventions

**All tests source lib/test_runner.sh and use preflight_plugins.**

この規約により、重複・ズレを防止し、運用しやすいスモークテストシステムを実現します。
#### **plugins** - プラグイン専用（任意）
- 安定検証用に最小フィクスチャプラグイン（`nyash-fixture-plugin`）を優先利用
- 実在プラグイン（Counter/Math/String）は存在すれば追加で実行（無ければSKIP）
