# Phi 正規化プラン（9.78h スキャフォールド）

目的: ループ/分岐における Phi 選択を正道に戻し、借用衝突を避けつつ段階導入する。

> ステータス更新（2025-09-26）: Step 3 まで実装済みで、MIR ビルダーは既定で PHI-on になったよ。以下のプランはアーカイブとして残しているよ。

段階プラン（80/20）
- Step 1: 実行系での選択復帰（完了）
  - `previous_block` に基づき `inputs[(bb==prev)]` を選択。見つからない場合は先頭をフォールバック。
  - デバッグ: `NYASH_VM_DEBUG_PHI=1` で選択ログ。
- Step 2: LoopExecutor 連携
  - `VM::loop_execute_phi` を `LoopExecutor::execute_phi` に委譲（安全な借用構成に整理）。
  - `record_transition(from,to)` をもとにヘッダ検出・イテレーション情報を活用。
- Step 3: 正規 SSA への復帰
  - Builder 側で phi 挿入・seal・predecessor 更新を正道で実装。
  - Verifier に phi 一貫性（定義支配/マージ使用）チェックを追加・厳格化。
- Step 4: ログ削減とテスト
  - 代表ケース（loop/if-merge/while）をスナップショット化。
  - 既定で静音、`NYASH_VM_DEBUG_PHI` のみで詳細。

実装状況（2025-08-26）
- Step 1 完了: `VM::loop_execute_phi` が `previous_block` による選択に対応。
- Step 2 スケルトン導入: `LoopExecutor` へphi実行を委譲し、`control_flow::record_transition(from,to)` で `previous_block` と遷移を記録。VM本体の分岐時に呼び出し済み。
- 既知の課題: `LoopExecutor` のヘッダ検出/イテレーション管理の強化（いまは簡易）。

次アクション
- `LoopExecutor` のヘッダ判定とイテレーション可視化を拡充（`is_loop_header` の実装、`NYASH_VM_DEBUG_PHI` 出力拡張）。
- Builder 側の phi 正規化 TODO を CURRENT_TASK に追記（seal/pred更新・Phi先頭挿入の確認用ユニットテスト追加）。
