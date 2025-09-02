# Nyashパッケージマネージャー設計書 v1.0

## 🎯 概要

Nyashのコード共有エコシステムを支える、シンプルで直感的なパッケージマネージャー「nypm (Nyash Package Manager)」の設計。

## 📊 設計原則

1. **シンプルさ優先** - npmの良い部分を参考に、複雑さを避ける
2. **Everything is Box** - パッケージもBoxの集合として扱う
3. **明示性** - 依存関係は常に明確に
4. **高速性** - 並列ダウンロード、効率的なキャッシュ

## 🔧 基本コマンド

### パッケージのインストール

```bash
# 依存関係をインストール
nyash install

# 特定パッケージをインストール
nyash install awesome-math
nyash install awesome-math@1.2.0

# 開発依存として追加
nyash install --dev test-framework

# グローバルインストール
nyash install -g nyash-formatter
```

### パッケージの公開

```bash
# パッケージを公開
nyash publish

# ドライラン（実際には公開しない）
nyash publish --dry-run

# アクセス制御付き公開
nyash publish --access public
```

### その他のコマンド

```bash
# パッケージ初期化
nyash init

# 依存関係の更新
nyash update
nyash update awesome-math

# パッケージの削除
nyash uninstall awesome-math

# 依存関係ツリーの表示
nyash list
nyash list --depth=0

# パッケージ検索
nyash search math

# パッケージ情報表示
nyash info awesome-math
```

## 📦 パッケージ構造

### ディレクトリ構成

```
my-awesome-package/
├── nyash.toml          # パッケージマニフェスト
├── src/
│   ├── index.ny        # メインエントリーポイント
│   └── lib/
│       └── utils.ny
├── tests/
│   └── test_main.ny
├── docs/
│   └── README.md
├── examples/
│   └── basic_usage.ny
└── .nyashignore        # 公開時の除外ファイル
```

### nyash.toml仕様

```toml
[package]
name = "awesome-math"
version = "1.0.0"
description = "高度な数学計算ライブラリ"
author = "Nyash Developer <dev@example.com>"
license = "MIT"
repository = "https://github.com/user/awesome-math"
keywords = ["math", "calculation", "algebra"]

# メインエントリーポイント
main = "src/index.ny"

# 最小Nyashバージョン
nyash = ">=1.0.0"

[dependencies]
# 実行時依存
basic-utils = "^2.0.0"
string-helpers = "~1.5.0"

[dev-dependencies]
# 開発時のみ必要
test-framework = "^3.0.0"
mock-library = "^1.2.0"

[scripts]
# カスタムスクリプト
test = "nyash test tests/"
build = "nyash compile src/"
lint = "nyash-lint src/"
```

### バージョン指定

```toml
# 正確なバージョン
"1.2.3"

# 互換性のあるバージョン（推奨）
"^1.2.3"  # >=1.2.3 <2.0.0

# 近似バージョン
"~1.2.3"  # >=1.2.3 <1.3.0

# 範囲指定
">=1.0.0 <2.0.0"

# ワイルドカード
"1.2.*"   # >=1.2.0 <1.3.0
```

## 🗂️ ローカルレジストリ

### nyash_modules構造

```
project/
├── nyash.toml
├── src/
│   └── main.ny
└── nyash_modules/      # 依存パッケージ格納場所
    ├── awesome-math/
    │   ├── nyash.toml
    │   └── src/
    ├── string-helpers/
    │   ├── nyash.toml
    │   └── src/
    └── .cache/         # ダウンロードキャッシュ
```

### パッケージ解決アルゴリズム

1. 現在のディレクトリの`nyash_modules/`をチェック
2. 親ディレクトリを再帰的に探索
3. グローバルインストールディレクトリをチェック
4. 見つからない場合はエラー

## 🌐 中央レジストリ

### レジストリAPI

```
GET  /packages/{name}              # パッケージ情報取得
GET  /packages/{name}/versions     # バージョン一覧
GET  /packages/{name}/{version}    # 特定バージョン情報
POST /packages                     # パッケージ公開
GET  /search?q={query}            # パッケージ検索
```

### パッケージメタデータ

```json
{
  "name": "awesome-math",
  "version": "1.0.0",
  "description": "高度な数学計算ライブラリ",
  "author": {
    "name": "Nyash Developer",
    "email": "dev@example.com"
  },
  "repository": "https://github.com/user/awesome-math",
  "downloads": {
    "last_day": 150,
    "last_week": 1200,
    "last_month": 5000
  },
  "versions": ["1.0.0", "0.9.0", "0.8.0"],
  "dependencies": {
    "basic-utils": "^2.0.0"
  },
  "tarball": "https://registry.nyash.dev/awesome-math-1.0.0.tgz"
}
```

## 🔒 セキュリティ

### パッケージ署名

```toml
# nyash.toml
[package.signature]
algorithm = "ed25519"
public_key = "..."
```

### 整合性チェック

```
nyash_modules/
└── awesome-math/
    ├── nyash.toml
    └── .nyash-integrity  # SHA256ハッシュ
```

### 権限システム

- **read**: パッケージの参照（デフォルト：全員）
- **write**: パッケージの更新（デフォルト：作者のみ）
- **admin**: 権限管理（デフォルト：作者のみ）

## 🚀 高度な機能

### ワークスペース

```toml
# ルートnyash.toml
[workspace]
members = [
    "packages/core",
    "packages/utils",
    "packages/cli"
]
```

### プライベートレジストリ

```toml
# .nyashrc
[registries]
default = "https://registry.nyash.dev"
company = "https://npm.company.com"

[scopes]
"@company" = "company"
```

### オフラインモード

```bash
# キャッシュからインストール
nyash install --offline

# キャッシュの事前ダウンロード
nyash cache add awesome-math@1.0.0
```

## 📈 パフォーマンス最適化

### 並列ダウンロード

- 最大10パッケージ同時ダウンロード
- HTTP/2による効率的な接続再利用

### インテリジェントキャッシュ

```
~/.nyash/cache/
├── packages/
│   └── awesome-math-1.0.0.tgz
├── metadata/
│   └── awesome-math.json
└── index.db  # SQLiteインデックス
```

### 差分更新

- パッケージ更新時は差分のみダウンロード
- バイナリdiffアルゴリズム使用

## 🛣️ 実装ロードマップ

### Phase 1: MVP（4週間）
- [ ] 基本的なinstall/publishコマンド
- [ ] nyash.tomlパーサー
- [ ] シンプルな依存解決
- [ ] ローカルファイルシステムレジストリ

### Phase 2: 中央レジストリ（6週間）
- [ ] HTTPSレジストリAPI
- [ ] ユーザー認証システム
- [ ] パッケージ検索
- [ ] Webインターフェース

### Phase 3: 高度な機能（8週間）
- [ ] ワークスペースサポート
- [ ] プライベートレジストリ
- [ ] セキュリティ機能（署名・監査）
- [ ] 差分更新

## 🎯 成功指標

1. **使いやすさ**: 3コマンド以内で基本操作完了
2. **高速性**: npm比で2倍以上の速度
3. **信頼性**: 99.9%のアップタイム
4. **エコシステム**: 1年で1000パッケージ

## 📚 参考実装

- **npm**: UIとワークフローを参考
- **Cargo**: 依存解決アルゴリズム
- **pnpm**: 効率的なディスク使用
- **Deno**: セキュリティモデル

---

*Everything is Box - パッケージマネージャーもBoxを運ぶ*