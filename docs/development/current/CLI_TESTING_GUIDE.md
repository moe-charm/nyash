## CLI分離テスト導線（軽量版）

目的: CLI変更に引きずられずにコアのMIR/VMを検証できる導線を用意する（構成は今は変えない）。

### 推奨手順
- コアのビルドとゴールデン照合のみで回す:
  - `cargo build --release -j32`
  - `./tools/ci_check_golden.sh`
- 代表E2E（プラグイン前提）のみ任意:
  - `cargo test --features plugins -q -- --nocapture`

### ヘルパースクリプト
- `tools/core_ci.sh`: コアのビルド＋ゴールデン照合を一括実行（CI/ローカル共用）

### 将来の分割方針（メモ）
- Cargo workspace化 or lib/binary分割で `cargo test -p core` を走らせる。
- runner（CLIフラグ/バックエンド選択）変更の影響をコア側に伝播させない。

最終更新: 2025-08-25

