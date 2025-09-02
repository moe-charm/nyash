# Phase 11.8 MIR Cleanup – Plan (Core‑13)

目的
- MIR を「最小の接着剤」に純化し、BoxCall へ集約。
- 最適化は VM/JIT の boxcall 経路に集中（脱仮想化・境界消去・Barrier）。

スコープ
- BoxCall 集約: ArrayGet/Set, RefGet/Set → BoxCall（get/set/getField/setField）。
- 維持: BinOp/Compare は MIR に残す（定数畳み込み/分岐簡約を最大化）。
- 効果: EffectMask の正確化、WriteBarrier の確実化。

段階導入トグル（env）
- `NYASH_MIR_ARRAY_BOXCALL=1` … ArrayGet/Set → BoxCall を有効化
- `NYASH_MIR_REF_BOXCALL=1` … RefGet/Set → BoxCall を有効化
- `NYASH_MIR_CORE13=1` … Core‑13 セットの一括有効（将来拡張）

実装ステップ
1) Optimizer パス（デフォルト OFF）
   - ArrayGet/Set → BoxCall に変換
   - RefGet/Set → BoxCall に変換
   - 変換後の Effect/Barrier を整合
2) VM: execute_boxcall の fast‑path
   - (type_id, method_id) で Array/Field を高速化
   - WriteBarrier の確実化
3) JIT: lower_boxcall の fast‑path
   - Array: GEP+Load/Store（Bounds/Barrier含む）
   - Field: 内部表現に応じた inlining（失敗時 plugin_invoke）
4) Smokes/Bench
   - array_access_sequential / array_access_random / field_access / arithmetic_loop
   - 基準: 速度 ±5%, メモリ ±10%, MIR サイズ -20% 目標
5) 検証
   - SSA 保持（Phi 導入後の整合）
   - 意味保存（before/after 等価）

非スコープ（当面）
- 算術/比較の BoxCall 化（最適化効率低下を避け据え置き）

完了基準
- トグル ON でスモークとベンチが基準を満たす
- VM/JIT ともに fast‑path が発火し、BoxCall 経路での最適化が確認できる

関連
- TECHNICAL_SPEC.md（詳細仕様）
- docs/development/runtime/ENV_VARS.md（環境変数索引）
