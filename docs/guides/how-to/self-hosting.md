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
   - 実行: `NYASH_PLUGIN_POLICY=off ./target/release/nyash --backend vm apps/selfhost-minimal/main.nyash`（compat: `NYASH_DISABLE_PLUGINS=1`）
3) クイックスモーク（VM軸）
   - 実行: `tools/smokes/v2/run.sh --profile quick`
4) プラグイン（任意・動的）
   - 実行: `tools/smokes/v2/run.sh --profile plugins`
5) LLVM 統合（任意・AOT/ハーネス）
   - 実行: `tools/smokes/v2/run.sh --profile integration`

Selfhost（公式ランナー経由）
- 親→子の ENV 透過で Ny コンパイラを起動し、最小 JSON を取得します。
- 例（AST ヘッダ非空）:
```
NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_MIN_JSON=1 NYASH_JSON_ONLY=1 \
  timeout 5 ./target/release/nyash --backend vm apps/examples/string_p0.nyash
```
- 例（最小 MIR: const→ret）:
```
NYASH_USE_NY_COMPILER=1 NYASH_NY_COMPILER_MIN_JSON=1 \
  NYASH_NY_COMPILER_CHILD_ARGS="--emit-mir" NYASH_JSON_ONLY=1 \
  timeout 5 ./target/release/nyash --backend vm apps/examples/string_p0.nyash
```

最小 Ny 実行器（MirVmMin）
- 目的: Ny だけで MIR(JSON v0) のごく最小セット（const/binop/compare/ret）を実行できることを確認。
- 実行例（VM）:
  - `./target/release/nyash --backend vm apps/selfhost/vm/mir_min_entry.nyash`
  - 引数で MIR(JSON) を渡すことも可能（単一文字列）。簡単な例は `apps/selfhost/vm/mir_min_entry.nyash` のコメントを参照。

検証
- 期待出力: `Result: 0`（selfhost‑minimal）
- スモーク：全成功（非 0 は失敗）

便利フラグ
- `NYASH_PLUGIN_POLICY=off` 外部プラグイン無効化（compat: `NYASH_DISABLE_PLUGINS=1`）
- `NYASH_CLI_VERBOSE=1` 実行ログ詳細
- `NYASH_USING_DYLIB_AUTOLOAD=1` using.dylib 自動ロード（開発用）

トラブルシュート
- ハング: `timeout 15s ...` を付与、`NYASH_CLI_VERBOSE=1` で詳細
- プラグインエラー: まず `NYASH_PLUGIN_POLICY=off`（compat: `NYASH_DISABLE_PLUGINS=1`）
- ルート相対パスで実行／`cargo clean -p nyash` で個別クリーン

関連
- CI: `.github/workflows/smoke.yml`（JSON/JUnit 出力は v2 ランナーで取得可能）
- マージ運用: `docs/development/engineering/merge-strategy.md`
