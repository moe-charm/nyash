# Smokes — How‑To（前提→手順→検証）

目的
- 代表スモークを素早く回して、回帰を検知する。

前提
- リリースビルド済み: `cargo build --release`
- LLVM を用いた AOT/ハーネス系は integration プロファイルで必要に応じて利用

手順（v2 ランナー推奨）
1) クイック確認（VM/動的プラグイン）
   - 実行: `tools/smokes/v2/run.sh --profile quick`
   - 代表的な言語機能・using の確認。冗長ログはフィルタ済み
2) プラグイン検証（VM/動的）
   - 実行: `tools/smokes/v2/run.sh --profile plugins`
   - フィクスチャ .so は自動ビルド・配置を試行（無ければ SKIP）
3) 統合確認（LLVM/LlvmLite ハーネス含む）
   - 実行: `tools/smokes/v2/run.sh --profile integration`
   - 必要に応じて PHI-on/off の比較や AOT 代表ケースを実行

手動スモーク（例）
- Core (LLVM): `examples/llvm11_core_smoke.nyash`
- Async (LLVM only):
  - `apps/tests/async-await-min/main.nyash`
  - `apps/tests/async-spawn-instance/main.nyash`
  - `apps/tests/async-await-timeout-fixed/main.nyash`（`NYASH_AWAIT_MAX_MS=100`）

アーカイブ（非推奨）
- 旧ランナー（JIT/Cranelift 時代）は削除または archive に移動済み。v2 ランナーのみを使用

便利フラグ
- `NYASH_LLVM_USE_HARNESS=1`: integration プロファイルで llvmlite ハーネスを使う
- `NYASH_MIR_NO_PHI=1`, `NYASH_VERIFY_ALLOW_NO_PHI=1`: レガシー PHI-off（edge-copy）モード
- `NYASH_USING_DYLIB_AUTOLOAD=1`: using kind="dylib" の自動ロードを有効化（dev 向け・既定OFF）

検証
- 0 で成功、非 0 で失敗（CI 連携可）
