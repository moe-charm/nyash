# Phase 15 — Self‑Hosting Doc Index

このインデックスは Phase 15（セルフホスティング）の計画・実装ドキュメントへの入口を1箇所にまとめます。状況に応じて随時更新します（正本）。

## 要点（すぐ見る）
- 現在タスク（正本）: ../../../../CURRENT_TASK.md
- 概要と目的: README.md
- 実行計画（常時更新のチェックリスト）: ROADMAP.md
- 推奨シーケンス（手順書）: recommended-sequence.txt
- 詳細計画（長文）: self-hosting-plan.txt
- lld戦略（AOT/リンク統合）: self-hosting-lld-strategy.md

## 設計とインターフェース
- Cranelift AOT 設計: ../../../backend-cranelift-aot-design.md
- Boxインターフェース案（Cranelift）: ../../../../interfaces/cranelift-aot-box.md
- LinkerBox 仕様案: ../../../../interfaces/linker-box.md

## ツール・スモーク
- AOTスモーク雛形: tools/aot_smoke_cranelift.sh / .ps1
- JITスモーク: tools/jit_smoke.sh
- ラウンドトリップ: tools/ny_roundtrip_smoke.sh
- using/namespace E2E: tools/using_e2e_smoke.sh

## 運用メモ/引き継ぎ
- ハンドオフ: ../../handoff/phase-15-handoff.md

注意:
- Phase 15関連の分散した文書は本インデックスから辿れるよう整理しています。新規文書を追加した場合は必ずここに追記してください。

