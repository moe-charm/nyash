# Self‑Hosting Dev — One‑Page Guide (selfhosting‑dev)

目的
- Ny → MIR → MIR‑Interp → VM/JIT の自己ホスト経路を最短手順で動かし、安定化する。
- Cranelift AOT/JIT‑AOT の詳細は別管理（リンク参照）。

前提
- Rust（stable）: `cargo --version`
- Bash/grep/rg（ripgrep）
- Linux/WSL/Unix シェル環境（WindowsはWSL推奨、PowerShellのps1も一部あり）

クイックスタート
1) ビルド（JIT有効）
   - `cargo build --release --features cranelift-jit`
2) 最小E2E（VM/JIT, plugins無効）
   - `NYASH_DISABLE_PLUGINS=1 ./target/release/nyash --backend vm apps/selfhost-minimal/main.nyash`
   - 期待: `Result: 0`
3) スモーク一式（コア）
   - `bash tools/jit_smoke.sh`
4) selfhost‑minimal 専用スモーク
   - `bash tools/selfhost_vm_smoke.sh`
5) ブートストラップ（任意）
   - `bash tools/bootstrap_selfhost_smoke.sh`
6) ラウンドトリップJSON（任意）
   - `bash tools/ny_roundtrip_smoke.sh`

便利フラグ（抜粋）
- `NYASH_DISABLE_PLUGINS=1` : 外部プラグイン無効化でコア安定化
- `NYASH_CLI_VERBOSE=1` : 実行ログ詳細化
- `NYASH_JIT_THRESHOLD=1` : JIT 降臨を確実化（ベンチ目的で使用）
- `NYASH_LOAD_NY_PLUGINS=1` : Ny プラグインロード（安定化後に段階的に試験）

トラブルシュート
- タイムアウト/ハング: `timeout 15s ...` を付ける、`NYASH_CLI_VERBOSE=1` で原因把握
- プラグイン関連エラー: まず `NYASH_DISABLE_PLUGINS=1` を設定してコア確認
- パス不一致: すべて repo ルート相対のパスで実行しているか確認
- ビルドキャッシュ: `cargo clean -p nyash` で個別クリーンを試す

CI 連携（参考）
- 既存Workflow: `.github/workflows/smoke.yml`（JIT/VMコアを含む）
- ローカル再現: `bash tools/smoke_phase_10_10.sh` ほか、上記スモークを順に実行

ブランチ方針/マージ
- 本ブランチは VM/JIT 中心。Cranelift は別ドキュメントへ分離。
- マージ運用・衝突回避は `docs/CONTRIBUTING-MERGE.md` を参照。
- Cranelift タスク詳細: `docs/phase-15/cranelift/CRANELIFT_TASKS.md`

連絡ノート
- 変更は基本 `tools/`, `dev/selfhosting/`, `src/{interpreter,runner,mir,parser}` の最小限に集中。
- 共有インターフェイス変更時は feature gate 等で互換性を維持。

