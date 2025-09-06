# Cranelift / AOT/JIT‑AOT Tasks (Phase 15)

このドキュメントは Cranelift backend（AOT/JIT‑AOT）関連の課題・進捗を集約します。
selfhosting‑dev ブランチでは VM/JIT 中心で開発するため、詳細はこちらへ集約し、`CURRENT_TASK.md` は軽量化しました。

最終更新: 2025‑09‑06（CURRENT_TASK から分離）

参考リンク
- 旧コンテンツ・完全版アーカイブ: `../../archives/CURRENT_TASK-2025-09-06.md`
- フェーズ概要: `../README.md`

現状サマリ（抜粋）
- StringBox.length/len が 0 になるケースの是正（Lower 二段フォールバック: string.len_h → any.length_h）
- Hostcall registry/extern thunks の追補（`SYM_STRING_LEN_H` 登録）
- AOT でのまれな segfault（DT_TEXTREL 警告）の追跡（TLS/extern 紐付け順）

優先課題（案）
1) Return 材化の強化（JIT‑direct/JIT‑AOT 共通）
2) Cranelift import シンボル解決の検証（`extern_thunks::nyash_string_len_h` の実呼出し保証）
3) AOT ツールチェーン（リンク・フラグ）の最小安定セット定義

運用メモ
- selfhosting‑dev では本ファイルの参照のみ（直接の実装変更は Cranelift 専用ブランチで実施）。
- 共有面（ランナー/IR など）に変更が必要な場合は feature gate と互換 API を優先し、両ブランチが同時に衝突しない形へ調整。

