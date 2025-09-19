# Phase 17 — 開発予定表（LoopForm Self‑Hosting & Polish）

期間目安: 4週間（総量は実アプリ進行に合わせて調整）

Week 1 — LoopForm MVP（while, break/continue無し）
- 目標: whileループを“キャリア（タプル）”へ正規化するユーザーマクロを実装（Nyash/PyVM）。
- 成果物:
  - apps/macros/examples/loop_normalize_macro.nyash（MVP）
  - ゴールデン: while基本/2変数キャリア/更新式の正規化
  - スモーク: selfhost-preexpand で自動適用→PyVM/LLVMの一致
- 受け入れ基準:
  - ループヘッダのPHIが先頭グループ化され、空PHIが存在しない
  - 代表ケースでPyVM/LLVMの出力一致

Week 2 — break/continue 対応＋キャリア自動抽出
- 目標: ループ内のbreak/continueを LoopForm の構造に沿って安全に配置。更新対象変数の集合からキャリア自動推定。
- 成果物:
  - マクロ拡張: break/continue/ネストの最小対応
  - ゴールデン: break/continue混在、未更新変数保持
- 受け入れ基準:
  - 分岐経路の合流でキャリアが常にwell‑typed
  - 変数の外側スコープ値が期待通り

Week 3 — for/foreach（限定）＋設計ドキュメント深掘り
- 目標: for/foreach を while へ前処理→LoopFormへ正規化。
- 成果物:
  - 追加パターン: for (init; cond; step) / foreach (x in xs)
  - ドキュメント: loopform-design.md（制約/限界/今後のMIR 4命令案）
- 受け入れ基準:
  - for/foreachの代表ケースで一致

Week 4 — Polishing & 実アプリ適用
- 目標: 実アプリ/自己ホストにLoopFormを適用して安定運用。
- 成果物:
  - スモーク/ゴールデン拡充、CIゲート最小化（fast/min）
  - ガイド更新（guides/loopform.md）
- 受け入れ基準:
  - 実アプリの主要ユースケースが緑

リスク/対策
- 複雑なネスト/例外: MVPでは簡略（try/finallyとの相互作用は先送り）→ドキュメントに制約明記
- 性能: コンパイル時のみの変換負荷。実行性能はMIR/LLVMへ委譲→ベンチで観測
- 仕様逸脱: 凍結遵守。必要時は限定的なDocs変更で表現

