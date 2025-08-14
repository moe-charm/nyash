# Phase 9.8: BIDレジストリ + 自動コード生成ツール（WASM/VM/LLVM/言語）

目的（What/Why）
- 外部ライブラリをBox（BID）として配布・発見・利用するための基盤を用意する。
- BID（Box Interface Definition）から各ターゲット（WASM/VM/LLVM/TS/Python）のスタブや宣言を自動生成し、開発者の負担を最小化する。

成果物（Deliverables）
- BIDレジストリ仕様（YAML/JSON スキーマ定義・バージョニング・依存関係・権限メタ）
- コードジェネレータCLI: `nyash bid gen --target wasm|vm|llvm|ts|py <bid.yaml>`
- 生成物（最低限）:
  - WASM: `(import ...)` 宣言テンプレ＋ `importObject.env.*` のホスト実装雛形
  - VM: 関数テーブル定義＋ディスパッチ雛形
  - LLVM: `declare` プロトタイプ群＋ヘッダ雛形（C-ABI前提）
  - TypeScript/Python: ラッパ（FFI呼び出しAPIのプロキシ）
- サンプルBIDからの生成例（console/canvas）

範囲（Scope）
1) スキーマ
   - `version`, `interfaces[]`, `methods[]`, `params`, `returns`, `effect`, `permissions`（9.9の権限連携）
   - 例: `docs/nyir/bid_samples/console.yaml`, `docs/nyir/bid_samples/canvas.yaml`
2) CLI
   - `nyash bid gen --target <t> <bid.yaml>` → `out/<t>/<name>/...` に生成
   - オプション: `--out`, `--force`, `--dry-run`
3) テンプレート
   - 各ターゲット用のMustache/Handlebars相当のテンプレート群
4) ドキュメント
   - `docs/予定/native-plan/box_ffi_abi.md` にBID→生成の流れを追記

受け入れ基準（Acceptance）
- console/canvas のBIDから、WASM/VM/LLVM/TS/Python の最小スタブが生成される
- 生成物を用いて、9.7 のE2E（console.log / canvas.fillRect）が通る
- `--dry-run` で生成前にプレビューが確認できる

非スコープ（Out of Scope）
- 高度な最適化生成、双方向同期、型高級機能（ジェネリクス/オーバーロード）
- 配布サーバやレジストリのネットワーク実装（ローカルファイル前提）

参照（References）
- ABI/BIDドラフト: `docs/予定/native-plan/box_ffi_abi.md`
- NyIR: `docs/nyir/spec.md`
- サンプルBID: `docs/nyir/bid_samples/console.yaml`, `docs/nyir/bid_samples/canvas.yaml`
- 計画: `docs/予定/native-plan/copilot_issues.txt`（9.7/9.8/9.9）

メモ（運用）
- 将来的に「BID→RuntimeImports/ExternCall宣言」の自動接続まで拡張予定（WASM/VM/LLVM）。
- 権限メタ（permissions）は 9.9 のモデルに合わせて必須化を検討。

