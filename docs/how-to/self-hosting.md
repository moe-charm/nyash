# Self‑Hosting — How‑To（前提→手順→検証）

目的
- Ny → MIR → VM/JIT の自己ホスト経路を最短手順で動かす。

前提
- Rust（stable）: `cargo --version`
- Bash + ripgrep（WSL/Unix 推奨）

手順
1) ビルド（JIT有効）
   - 実行: `cargo build --release --features cranelift-jit`
2) 最小 E2E（VM、plugins 無効）
   - 実行: `NYASH_DISABLE_PLUGINS=1 ./target/release/nyash --backend vm apps/selfhost-minimal/main.nyash`
3) コアスモーク
   - 実行: `bash tools/jit_smoke.sh`
4) selfhost‑minimal スモーク
   - 実行: `bash tools/selfhost_vm_smoke.sh`
5) 追加（任意）
   - ブートストラップ: `bash tools/bootstrap_selfhost_smoke.sh`
   - ラウンドトリップ: `bash tools/ny_roundtrip_smoke.sh`

検証
- 期待出力: `Result: 0`（selfhost‑minimal）
- スモーク：全成功（非 0 は失敗）

便利フラグ
- `NYASH_DISABLE_PLUGINS=1` 外部プラグイン無効化
- `NYASH_CLI_VERBOSE=1` 実行ログ詳細
- `NYASH_JIT_THRESHOLD=1` JIT 降臨テスト

トラブルシュート
- ハング: `timeout 15s ...` を付与、`NYASH_CLI_VERBOSE=1` で詳細
- プラグインエラー: まず `NYASH_DISABLE_PLUGINS=1`
- ルート相対パスで実行／`cargo clean -p nyash` で個別クリーン

関連
- CI: `.github/workflows/smoke.yml`
- マージ運用: `docs/CONTRIBUTING-MERGE.md`
