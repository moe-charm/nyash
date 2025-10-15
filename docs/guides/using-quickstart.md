# Using Quickstart — Hakorune (Phase‑1)

目的
- 迷いやすい `using` 設定を、最小ステップで「確実に動く」形にする。

選べる 2 方式（どちらか片方）

1) 開発用（ファイルパス using を許可）
- `hako.toml` の `[env]` に以下を入れる（既定ON）。`NYASH_SKIP_TOML_ENV=1` で無効化可能。
```
[env]
HAKO_USING = "1"
HAKO_USING_STRATEGY = "prelude"
HAKO_ALLOW_USING_FILE = "1"
HAKO_USING_PROFILE = "dev"
```
- 実行はプロジェクトルートで、または `HAKO_ROOT=<repo>` を設定。

2) パッケージ参照（推奨）
- `hako.toml` に using パッケージ/オーバーライドを登録し、コード側は名前で参照：
```
[using]
paths = ["apps", "lib", "."]

[modules.overrides]
selfhost.hakorune_vm.hakorune_vm_core = "selfhost/hakorune-vm/hakorune_vm_core.hako"
```
- ソース: `using selfhost.hakorune_vm.hakorune_vm_core as VM`

便利スイッチ（ワンノブ）
- `source tools/dev_env.sh using` で、上記の HAKO_* 環境を一括で有効化。
- 元に戻す: `source tools/dev_env.sh reset`

トラブルシュート（よくある症状）
- Parse error: Expected identifier (line N)
  - `HAKO_USING=0` のまま `using` を書いている可能性。`HAKO_USING=1`, `HAKO_USING_STRATEGY=prelude` を設定。
  - `hako.toml` が読まれていない（CWDがルートでない）。`HAKO_ROOT` を設定するか、ルートで実行。
- `using: file paths are disallowed`（ガイダンス）
  - dev であれば `HAKO_ALLOW_USING_FILE=1` を有効化。prod ではパッケージ参照に移行。

メモ
- 既存の `NYASH_*` は互換エイリアス。新規ドキュメント/スクリプトは `HAKO_*` を第一表記に。

