# Phase 10: AOT scaffolding (exploration)

## Summary
- AOT の下ごしらえ（ビルド配線と最小 PoC）。将来の本実装に向け、ターゲット/成果物レイアウト/テストの枠組みを整える。

## Scope
- Cargo features/targets の整理（aot 用 feature/target の予約のみ）
- 生成物レイアウト案（dist/ 下のファイル構成、メタデータ）
- 実行フローの素案（nyash → MIR → AOT 生成 → 実行）

## Tasks
- [ ] AOT feature/target の定義（実装は未着手でOK）
- [ ] dist/ レイアウトのひな形作成（README/Licenses 同梱方針）
- [ ] PoC: ダミー AOT 生成物（プレースホルダ）とテストスクリプト

## Acceptance Criteria
- AOT 用のビルド配線が雛形レベルで通る（ビルド/テスト スケルトン）
- dist/ の標準レイアウトが定義され、CI に載せられる状態

## Out of Scope
- 実際の AOT コンパイル・最適化

## References
- docs/予定/native-plan/copilot_issues.txt（Phase 10）
