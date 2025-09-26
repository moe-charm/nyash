# Phase 17 — LoopForm Self‑Hosting & Polish

Status: planning

Purpose
- 固定済みのコア仕様を維持しつつ、Nyash側（ユーザーマクロ＋標準ライブラリ）で LoopForm を先に実装し、ループの正規化を言語レベルで確立する。
- Rust側は既存MIR/LLVMの整流を活用（PHI先頭グループ化の不変条件を活かす）。
- 実アプリ/自己ホストで磨き込みを進め、言語としての使い心地を上げる。

Scope
- LoopForm（while→キャリア正規化）のユーザーマクロ実装とガイド。
- 代表スモーク/ゴールデンの追加（PyVM/LLVMの一致）とPHI健全性チェックの拡充。
- Docsの整備（設計・ガイド・運用ポリシー）。

Out of Scope（機能追加ポーズ遵守）
- Rust側の大規模なIR変更やバックエンド機能追加はしない（必要最小限のバグ修正のみ）。
- 仕様変更は重大不具合を除き行わない。

Guardrails（シンプルさ維持）
- Small‑by‑default: 既定は簡素、プロファイルで拡張。
- ヒューリスティック禁止: 明示登録とAST検出のみ。
- バグは点修正、Docs/テストは積極整備。

Docs
- guides/loopform.md（利用者向け）
- loopform-design.md（設計詳細）
- SCHEDULE.md（開発予定表）
