# Phase 14: Packaging/CI polish

## Summary
- Windows/Linux の配布パッケージ化と CI 整備。利用者がすぐ使えるバイナリを提供し、ビルドの再現性を担保する。

## Scope
- CI: GitHub Actions で Windows(MSVC) / WSL + cargo-xwin のマトリクス
- リリース成果物: dist/nyash(.exe) + README + LICENSE （必要なら examples/）
- 署名/ハッシュ（任意）：SHA256 発行・検証手順

## Tasks
- [ ] actions ワークフロー作成（キャッシュ/マトリクス/アーティファクト）
- [ ] dist 出力スクリプト（バージョン埋め込み）
- [ ] リリースノートの雛形追加（CHANGELOG or GitHub Releases）

## Acceptance Criteria
- Actions が緑で、アーティファクトが自動生成・ダウンロード可能
- dist/ の内容が README に記載通り

## Out of Scope
- コードサイン（必要になったら追補）

## References
- docs/予定/native-plan/copilot_issues.txt（Phase 14）
