# Box-First Enforcement Kit（運用メモ）

このプロジェクトでは「箱を先に積む（Box-First）」を最優先にし、実装速度のボトルネックを“下の箱（境界・足場）不足”で詰まらせない方針を採用します。

## PR テンプレ（.github/pull_request_template.md）
PR で以下のチェックを通すことを習慣化します。

```
### Box-First Check
- [ ] 境界は1箇所に集約（変換はここだけ）
- [ ] 設定は JitConfigBox 経由（env直読みなし）
- [ ] フォールバック常設（panic→VM/CPython）
- [ ] 観測追加（stats.jsonl / CFG dot）

### DoD（完了条件）
- [ ] ゴールデン3件（成功/失敗/境界）更新
- [ ] 回帰CI green（env直読み検出なし）
- [ ] stats: fallback率・理由が記録される
```

## CI ガード（.github/workflows/box_first_guard.yml）
現状は「アドバイザリ（continue-on-error）」で運用。違反箇所を可視化します。

- 直の `std::env::var(` を `src/jit/config.rs` と `src/jit/rt.rs` 以外で禁止（アドバイザリ）
- B1 署名のスイッチ箇所以外での `B1` 文字列の出現を禁止（アドバイザリ）
- 将来的に `stats.jsonl` 出力の有無も検査予定

必要になったら `continue-on-error: false` にして強制化します。

## “下の箱”不足の早期警報（運用ルール）
進みが悪い／壊れやすい兆候が出たら、まず以下から最小1個だけ足して再挑戦：

- BoundaryBox（変換一本化）
- JitConfigBox（設定の箱）
- ObservabilityBox（json/dot出力）
- Effect Token（副作用の明示）

## Box-Fitness ミニ指標（PRに1行）

- `boundary_changes=1`（変換点の個数）
- `env_reads=0`（env直読の個数）
- `fallback_paths>=1`（逃げ道の数）
- `stats_fields>=5`（記録の粒度）

この4つを満たせていれば「箱の足場は十分」の合図。

