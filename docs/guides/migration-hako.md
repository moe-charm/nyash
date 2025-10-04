# .nyash → .hako Migration Guide

目的
- 拡張子を `.nyash` から `.hako` に段階移行する方針を明記し、実務での混乱を避ける。

方針（Phase 15.7+）
- 表示・例から `.hako` に切替（Docs/README/Makefile の例示）。
- 自己ホスト/ライブラリから `.hako` 追加（旧 `.nyash` は短期併存）。
- 参照（nyash.toml/[using]/[modules]/using 行）を段階で `.hako` へ寄せ、完了後 `.nyash` を撤去。

現状（2025-10-04）
- json_native: parser/core/utils/lexer を `.hako` 追加、parser.hako は `.hako` 参照に統一済み。
- selfhost-compiler: parser/*, builder/*, mir/*, emitter/json_v0, interfaces を `.hako` 追加。新規モジュールは `.hako` を参照。
- selfhost/vm: 一部箱を `.hako` 化（Mini‑VM系・Scan系）。
- docs: README/README.ja/AGENTS/CODEX を `.hako` 例示に更新済み。

手順（開発者向け）
1) 新規ファイルは `.hako` のみで作成。
2) 既存 `.nyash` はコピーで `.hako` を追加 → 参照（using/nyash.toml）を `.hako` に変更 → 1 リリース後 `.nyash` を撤去。
3) テスト/スモークは `.hako` を優先（過去の `.nyash` は順次置換）。

互換・注意
- Runner は拡張子に依存しないが、Docs/ツールは `.hako` を既定とする。
- 最終的に `.nyash` は撤去予定（Fail‑Fast）。
