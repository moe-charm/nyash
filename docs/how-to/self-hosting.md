# Self‑Hosting — How‑To（前提→手順→検証）

目的
- Ny → MIR → VM/JIT の自己ホスト経路を最短手順で動かす。

前提
- Rust（stable）: `cargo --version`
- Bash + ripgrep（WSL/Unix 推奨）

手順（v2 推奨）
1) ビルド
   - 実行: `cargo build --release`
2) 最小 E2E（VM、plugins 無効）
   - 実行: `NYASH_DISABLE_PLUGINS=1 ./target/release/nyash --backend vm apps/selfhost-minimal/main.nyash`
3) クイックスモーク（VM軸）
   - 実行: `tools/smokes/v2/run.sh --profile quick`
4) プラグイン（任意・動的）
   - 実行: `tools/smokes/v2/run.sh --profile plugins`
5) LLVM 統合（任意・AOT/ハーネス）
   - 実行: `tools/smokes/v2/run.sh --profile integration`

最小 Ny 実行器（MirVmMin）
- 目的: Ny だけで MIR(JSON v0) のごく最小セット（const/binop/compare/ret）を実行できることを確認。
- 実行例（VM）:
  - `./target/release/nyash --backend vm apps/selfhost/vm/mir_min_entry.nyash`
  - 引数で MIR(JSON) を渡すことも可能（単一文字列）。簡単な例は `apps/selfhost/vm/mir_min_entry.nyash` のコメントを参照。

検証
- 期待出力: `Result: 0`（selfhost‑minimal）
- スモーク：全成功（非 0 は失敗）

便利フラグ
- `NYASH_DISABLE_PLUGINS=1` 外部プラグイン無効化
- `NYASH_CLI_VERBOSE=1` 実行ログ詳細
- `NYASH_USING_DYLIB_AUTOLOAD=1` using.dylib 自動ロード（開発用）

トラブルシュート
- ハング: `timeout 15s ...` を付与、`NYASH_CLI_VERBOSE=1` で詳細
- プラグインエラー: まず `NYASH_DISABLE_PLUGINS=1`
- ルート相対パスで実行／`cargo clean -p nyash` で個別クリーン

関連
- CI: `.github/workflows/smoke.yml`（JSON/JUnit 出力は v2 ランナーで取得可能）
- マージ運用: `docs/development/engineering/merge-strategy.md`
