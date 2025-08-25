# Phi 正規化プラン（9.78h スキャフォールド）

目的: ループ/分岐における Phi 選択を正道に戻し、借用衝突を避けつつ段階導入する。

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
- 既知の課題: LoopExecutor 経由の借用安全な委譲（Step 2）。

次アクション
- VM 内部の phi 実行を LoopExecutor へ委譲できるよう API を見直し（`get_value` クロージャの借用境界を調整）。
- Builder 側の phi 正規化 TODO を CURRENT_TASK に追記。

