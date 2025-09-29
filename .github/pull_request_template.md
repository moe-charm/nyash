### Box-First Check
- [ ] 境界は1箇所に集約（変換はここだけ）
- [ ] 設定は JitConfigBox 経由（env直読みなし）
- [ ] フォールバック常設（panic→VM/CPython）
- [ ] 観測追加（stats.jsonl / CFG dot）

### DoD（完了条件）
- [ ] ゴールデン3件（成功/失敗/境界）更新
- [ ] 回帰CI green（env直読み検出なし）
- [ ] stats: fallback率・理由が記録される

### Selfhosting‑dev Gate（このブランチ向け）
- [ ] `bash tools/selfhost_vm_smoke.sh` が PASS（plugins 無効）
- [ ] `docs/development/engineering/merge-strategy.md` の境界方針を満たす（Cranelift実装差分は専用ブランチ）
- 影響範囲: runner / interpreter / vm / tools / docs
- Feature gates（該当時）: `cranelift-jit`, その他（記述）
