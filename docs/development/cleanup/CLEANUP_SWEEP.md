# Cleanup Sweep (Phase 11.8–12 Bridge)

目的
- レガシー/未使用コード・重複実装・旧命名の残骸を段階的に除去し、MIR/VM/JIT の読みやすさと安全性を高める。

優先カテゴリ（初回パス）
- MIR:
  - 旧レガシー命令の痕跡（TypeCheck/Cast/WeakNew/WeakLoad/BarrierRead/BarrierWrite の分岐/診断まわり）。
  - `builder_modularized/*` と `builder/*` の重複（存在時は後者へ収斂）。
- VM/JIT:
  - 直 `std::env::var` の散在（config::env/jit::config へ寄せられるもの）。
  - BoxCall 経路の TODO/旧コメント（経路確定後に削除）。
- Docs/Tools:
  - 古い計画/アーカイブとの重複ページ整理（現行PLAN/TECHNICAL_SPECへ誘導）。

進め方
- まず一覧化（rgベース → PRでまとめて削除/移動）。
- 削除基準: 未参照・未テスト・新仕様と重複でかつ互換層なし。
- 互換が必要な場合は警告/診断ログに降格し、実装は1本に統合。

チェックリスト（暫定）
- [ ] builder_modularized の実使用確認（未使用なら削除/統合）。
- [ ] optimizer の診断/オプション環境変数の棚卸し（ENV_VARS.md へ集約）。
- [ ] wasm backend の RefGet/RefSet 旧実装コメント更新。
- [ ] 直 env 読みを advisory で一覧 → 対象の優先度決め。
- [ ] dead code（テストで未参照）の削除（段階的）。

メモ
- claude code 指摘「37件」は次のスイープで対象抽出 → PRリンク化予定。
