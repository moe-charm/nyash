# Phase 9.10: NyIR v1 仕様・フォーマット・検証器（Copilot実装用タスク）

目的（What/Why）
- NyashのMIRを公開IR（NyIR v1）として凍結し、あらゆるフロントエンド/バックエンドの共通契約にする。
- 仕様・テキスト/バイナリフォーマット・厳格検証器・ツール群を整備し、移植性と一貫性を保証する。
- 設計の正本は `docs/nyir/spec.md`（Core＋Extの骨子）。本ファイルはCopilotが実装を進めるための具体タスク集。

スコープ（Deliverables）
- 仕様書（骨子は既存）: `docs/nyir/spec.md` に沿ったv1確定版の追補
- フォーマット: `.nyir`（テキスト）, `.nybc`（バイナリ）
- 検証器: `nyir-verify`（CLI/ライブラリ）
- 変換/実行ツール:
  - `nyashel -S`（Nyash→NyIRダンプ）
  - `nyir-run`（NyIRインタプリタ）
  - 参考: `nyir-ll`（NyIR→LLVM IR、Phase 10で拡張）
- Golden NyIR: `golden/*.nyir`（代表サンプルを固定、CIで全バックエンド一致を検証）

実装タスク（Copilot TODO）
1) 仕様固定
   - `docs/nyir/spec.md` に従い、25命令Core＋Effect＋Ownership＋Weak＋Busをv1として明文化。
   - NyIR-Ext（exceptions/concurrency/atomics）の章は骨子のみ維持（別Phase）。
2) `.nyir` パーサ/プリンタ（最小）
   - 構造: moduleヘッダ / features / const pool / functions（blocks, instrs）
   - コメント/メタ/featureビットを許容（v1最小でもOK）
3) `.nybc` エンコーダ/デコーダ（最小）
   - セクション: header / features / const pool / functions / metadata
   - エンコード: LEB128等。識別子はstring table参照
4) `nyir-verify`（厳格検証器）
   - 検査: 所有森/強循環/weak規則/効果整合/到達性/終端性/Phi入力整合
   - 失敗時は明確なエラーを返し、ロード拒否
5) `nyashel -S` をNyIR出力対応に
   - 既存MIRダンプ経路から移行。`.nyir`で出力
6) Goldenサンプル＋CI
   - `golden/*.nyir` 作成（3〜5本）
   - CIで interp/vm/wasm（順次llvm）に投げ、出力一致を確認

受け入れ基準（Acceptance）
- 代表サンプルが `.nyir` で表現・検証・実行可能（`nyir-run`）
- `.nybc` 読み書き往復で等価
- CIでinterp/vm/wasmの結果一致（最小ケース）

非スコープ（Out of Scope）
- NyIR-Ext の実装（例外/非同期/アトミック）は別Phase（9.12/9.13/9.14想定）
- 高度最適化/再配線は対象外（意味保存に限定）

参照（References）
- NyIR 骨子: `docs/nyir/spec.md`
- ABI/BIDドラフト: `docs/予定/native-plan/box_ffi_abi.md`
- 計画（9.7以降フルテキスト）: `docs/予定/native-plan/copilot_issues.txt`

メモ（運用）
- 仕様の正本は `docs/nyir/`。本Issueは実装タスクと受け入れ基準を明快に維持。
- Golden/Diffテストを並行で用意して、バックエンド横断の一貫性を早期に担保。
