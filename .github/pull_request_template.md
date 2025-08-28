### Box-First Check
- [ ] 境界は1箇所に集約（変換はここだけ）
- [ ] 設定は JitConfigBox 経由（env直読みなし）
- [ ] フォールバック常設（panic→VM/CPython）
- [ ] 観測追加（stats.jsonl / CFG dot）

### DoD（完了条件）
- [ ] ゴールデン3件（成功/失敗/境界）更新
- [ ] 回帰CI green（env直読み検出なし）
- [ ] stats: fallback率・理由が記録される

