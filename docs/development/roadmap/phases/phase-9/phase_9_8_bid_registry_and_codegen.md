# Phase 9.8: BIDレジストリ + 自動コード生成ツール（WASM/VM/LLVM/言語）

Status: Planned（Phase 8.6完了後に着手）
Last Updated: 2025-08-25

目的（What/Why）
- 外部ライブラリをBox（BID）として配布・発見・利用するための基盤を用意する。
- 当面は nyash.toml にBID情報を“埋め込む”方式で回し、将来は外部BID(manifest)参照＋自動生成へ段階拡張する。

成果物（Deliverables）
- BIDレジストリ仕様（YAML/JSON スキーマ定義・バージョニング・依存関係・権限メタ）
- コードジェネレータCLI: `nyash bid gen --target wasm|vm|llvm|ts|py <bid.yaml>`
- 生成物（最低限）:
  - WASM: `(import ...)` 宣言テンプレ＋ `importObject.env.*` のホスト実装雛形
  - VM: 関数テーブル定義＋ディスパッチ雛形
  - LLVM: `declare` プロトタイプ群＋ヘッダ雛形（C-ABI前提）
  - TypeScript/Python: ラッパ（FFI呼び出しAPIのプロキシ）
- サンプルBIDからの生成例（console/canvas）

範囲（Scope：段階的）
A) すぐやる（埋め込みBID）
   - nyash.toml に最小BID情報（署名・効果・権限）を記述し、ランタイムローダが読み込む
   - ExternCall/Plugin呼び出し時にBIDの`effects/permissions`を参照して実行可否を判定
B) 次にやる（参照BID）
   - nyash.toml から外部BID（bid.yaml 等）を参照・マージ可能にする（アグリゲータ）
C) 自動生成（安定後）
   - CLI: `nyash bid gen --target <t> <bid.yaml>` → `out/<t>/<name>/...` に生成
   - テンプレート: WASM(importObject), VM(関数テーブル), LLVM(declare), TS/Python(RTEラッパ)
   - ドキュメント: `docs/予定/native-plan/box_ffi_abi.md` にBID→生成の流れを追記

受け入れ基準（Acceptance：段階的）
- A: nyash.toml の BID 情報だけでランタイム実行・権限判定が可能（外部BIDなしでも動作）
- B: 外部BID(manifest)を nyash.toml から参照・マージできる
- C: console/canvas のBIDから、WASM/VM/LLVM/TS/Python の最小スタブが生成される（`--dry-run` 対応）

非スコープ（Out of Scope）
- 高度な最適化生成、双方向同期、型高級機能（ジェネリクス/オーバーロード）
- 配布サーバやレジストリのネットワーク実装（ローカルファイル前提）

参照（References）
- ABI/BIDドラフト: `docs/予定/native-plan/box_ffi_abi.md`
- NyIR: `docs/nyir/spec.md`
- サンプルBID: `docs/nyir/bid_samples/console.yaml`, `docs/nyir/bid_samples/canvas.yaml`
- 計画: `docs/予定/native-plan/copilot_issues.txt`（9.7/9.8/9.9）

メモ（運用）
- 対応する形式が増えたら、まず nyash.toml にBIDを追記（“その都度対応”の方針）
- 将来的に「BID→RuntimeImports/ExternCall宣言」の自動接続まで拡張予定（WASM/VM/LLVM）。
- 権限メタ（permissions）は 9.9 のモデルに合わせて必須化を検討。
