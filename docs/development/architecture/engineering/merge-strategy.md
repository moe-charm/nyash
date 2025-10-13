# Merge Strategy — selfhosting‑dev × Cranelift branches

目的
- selfhosting‑dev（VM/JIT 自己ホスト）と Cranelift 専用ブランチ（AOT/JIT‑AOT）を並行開発しつつ、衝突と複雑な解消作業を最小化する。

ブランチの役割
- `selfhosting-dev`: Ny→MIR→MIR-Interp→VM/JIT の安定化、ツール/スモーク、ドキュメント整備。
- `phase-15/self-host-aot-cranelift`（例）: Cranelift backend の実装・検証。
- `develop`: 定期同期の受け皿。`main` はリリース用。

方針（設計）
- 境界の明確化: Cranelift 固有コード（例: `src/jit/*`, `src/jit/rt.rs` など）は専用ブランチで集中的に変更。selfhosting‑dev は Runner/Interpreter/VM の公共 API に限定。
- Feature gate: 共有面に変更が必要な場合は `#[cfg(feature = "cranelift-jit")]` 等で分岐し、ABI/シグネチャ互換を保つ。
- ドキュメント分離: `CURRENT_TASK.md` はインデックス化し、詳細は `docs/phase-15/*` へトピックごとに分離（本運用により md の大規模衝突を回避）。

方針（運用）
- 同期リズム: selfhosting‑dev → develop へ週1回まとめPR。Cranelift 側も同周期で develop へリベース/マージ。
- 早期検知: 各PRで `rg` による衝突予兆チェック（ファイル/トークンベース）をテンプレに含める。
- rerere: `git config rerere.enabled true` を推奨し、同種の衝突解消を再利用。
- ラベル運用: `area:jit`, `area:vm`, `docs:phase-15`, `merge-risk:high` 等でレビュー優先度を明示。

ファイルオーナーシップ（推奨）
- Cranelift: `src/jit/**`, `src/jit/policy.rs`, `tools/*aot*`, `docs/phase-15/cranelift/**`
- Selfhost core: `src/interpreter/**`, `src/runner/**`, `dev/selfhosting/**`, `tools/smokes/v2/**`
- 共有/IR: `src/mir/**`, `src/parser/**` は変更時に両ブランチへ告知（PR説明で影響範囲を明記）。

実務Tips
- マージ用テンプレ（PR description）
  - 目的 / 影響範囲 / 変更対象ファイル / 互換性 / リスクと回避策 / テスト項目
- 衝突抑止の小技
  - md は章分割して別ファイルに参照化（本運用の通り）
  - 大規模renameは単独PRで先行適用
  - 共有インターフェイスは薄いアダプタで橋渡し（実装詳細は各ブランチ内）
