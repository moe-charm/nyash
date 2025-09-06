Phase 15 — Self-Hosting (Cranelift AOT) 準備メモ

関連ドキュメント（selfhosting-dev 運用）
- VM/JIT 自己ホストガイド: `docs/self-hosting.md`
- Cranelift/AOT タスク集約: `docs/phase-15/cranelift/CRANELIFT_TASKS.md`

注意: Phase 15 の正本ドキュメントは `docs/development/roadmap/phases/phase-15/` 配下です。全体の入口は `INDEX.md` を参照してください。
→ docs/development/roadmap/phases/phase-15/INDEX.md

目的（Self‑Hosting / VM先行・JITはcompiler‑only）
- Nyash → MIR → VM/JIT（JITは独立実行）経路の自己ホストを実用化。
- AOT/リンクは main 側で推進。本ブランチは最小実装＋観測整備を優先。

現状ステータス（このブランチ）
- 設計ノート: docs/backend-cranelift-aot-design.md
- インタフェース草案: docs/interfaces/cranelift-aot-box.md
- LinkerBox 仕様: docs/interfaces/linker-box.md
- AOTスモーク仕様（擬似出力）: docs/tests/aot_smoke_cranelift.md
- スモーク雛形（DRYRUN 既定）:
  - tools/aot_smoke_cranelift.sh（Unix/WSL）
  - tools/aot_smoke_cranelift.ps1（Windows）

ハンドオフ
- 引き継ぎの全体像と運用メモは docs/handoff/phase-15-handoff.md を参照。

Phase 15 実行計画（2週間の目安）
1) JSON v0 短絡 &&/|| 追加（短絡副作用なしの確認）
2) コレクション最小 hostcall（len/get/set/push/size/has）＋policyガード再確認
3) プラグイン橋の衛生（by-id/by-nameの最小）
4) using/module の最終調整（候補提示は“ほどほど”に）
5) 可観測イベント（observe::lower_hostcall など）整備
6) 安定化と1ページメモ更新

合否基準（本ブランチ）
- 代表smokeで VM/JIT（--jit-direct）が一致し、JSON v0 短絡と collections 最小op が緑。
- イベント出力が一定（hostcall 1回=1件、短絡は分岐採用の記録）。

補足（Do-Not-Do）
- AOT/リンク最適化、GUI拡張、機能拡張の広げ過ぎ、最適化の深追い、新規依存追加はしない。
