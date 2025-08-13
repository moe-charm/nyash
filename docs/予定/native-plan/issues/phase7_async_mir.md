# Phase 7: Async model in MIR (nowait/await)

Summary:
- nowait/await を MIR に薄く導入（Future 表現）。スレッドベース実装と整合。

Scope:
- MIR: FutureNew / FutureSet / Await の導入（Scheduling は現状 thread::spawn 準拠）
- Lowering: nowait → Future 作成 + スケジューリング、await → wait_and_get
- VM: 今の FutureBox 実装を利用

Tasks:
- [ ] 命令・builder・vm 実装
- [ ] サンプル/スナップショット

Acceptance Criteria:
- 代表ケースで VM 実行が期待通り（順不同完了→await で正しい結果）

References:
- docs/nyash_core_concepts.md（nowait/await + FutureBox）

