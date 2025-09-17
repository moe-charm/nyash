# Phase 9.9: ExternCall 権限/ケイパビリティモデル（Sandbox/Allowlist）

目的（What/Why）
- ExternCall（外部API呼び出し）に対する権限制御を導入し、安全な実行を担保する。
- BIDで必要権限を宣言し、ホスト側でAllowlist/設定により許可・拒否できる仕組みを整える。

成果物（Deliverables）
- 権限モデル仕様（permissionカテゴリ、宣言/検証ルール、失権時挙動）
- 実行時制御（WASM/VM/LLVM各実装）
  - WASM: import allowlist（許可された `env.*` のみ解決）
  - VM/LLVM: 関数テーブル/リンク時のゲート（未許可はスタブで拒否）
- 構成手段
  - 設定ファイル（例: `nyash_permissions.toml`）
  - 環境変数/CLIフラグ（`--allow console,canvas` など）
  - （将来）対話プロンプト/UI

範囲（Scope）
- 権限カテゴリ（初版）
  - `console`, `canvas`, `storage`, `net`, `audio`, `time`, `clipboard` など
- BID拡張
  - 各methodに `permissions: [ ... ]` を必須化（v0は任意→将来必須）
- 検証/実行
  - 生成時（BID→コード生成）: 権限メタを埋め込む
  - 実行時: 設定に基づき、未許可のExternCallは即エラー
- 失権時の標準挙動
  - 明示エラー（例: `PermissionDenied: env.canvas.fillRect requires [canvas]`）

受け入れ基準（Acceptance）
- `console` のみ許可した設定で、`console.log` は成功、`canvas.fillRect` は拒否される
- WASM/VM/LLVM いずれでも、未許可呼び出しが正しくブロックされ、メッセージが一貫
- BIDの `permissions` を外す/変えると、生成物の権限制御が反映される

非スコープ（Out of Scope）
- ユーザー対話UI/OSネイティブ権限ダイアログ（将来）
- 細粒度（URL/ドメイン単位など）のネット権限制御（将来）

参照（References）
- BID/ABI: `docs/予定/native-plan/box_ffi_abi.md`
- NyIR/ExternCall: `docs/nyir/spec.md`
- 計画: `docs/予定/native-plan/copilot_issues.txt`（9.7/9.8/9.9）

メモ（運用）
- 9.8 のコードジェネレータに `permissions` を伝播させ、テンプレートに権限チェックを組み込む。
- 権限のデフォルトは「Deny All」（明示許可のみ通す）を推奨。
