# NyKernel と Plugins — 最小ランタイムとプラグイン体系（Phase 15.5）

Status: Proposed (受け口のみ; 既定OFF)  
ADR: docs/development/adr/adr-001-no-corebox-everything-is-plugin.md

## 目的
- Kernel を最小化（箱を持たない）し、機能はすべて Plugin で提供する。
- VM/LLVM 双方から同一 ABI（`ny_new_box` / `ny_call_method`）で箱を扱う。

## 起動シーケンス（標準形）
1) NyKernel init（GC/Handle/TLV/Extern/PluginRegistry）
2) hako.toml（または互換: nyash.toml）読み込み
   - `plugins.bootstrap`（静的束）を登録
   - `plugins.dynamic`（.so/.dll）があれば dlopen 登録
3) Plugin Verify（必須メソッド/TLV/ABI）
4) 実行（VM/LLVM → `ny_call_method`）

ブートの不変条件（重要）
- Provider Lock（型→提供者の対応表）が確定するまで、いかなる Box 生成も禁止。
- Kernel ログ（生バイト）で初期エラーを出力し、`StringBox` 等の Box をログ用途に使わない。
- Verify に失敗した場合は、Kernel ログで理由を表示して即終了する。

## Provider/Type 分離（概要）
- Stable Type Name（STN）: `StringBox`, `IntegerBox` など。コード上の型名は不変。
- Provider ID（PVN）: `kernel:string@1.0` / `acme:string@2.1` など実装提供者。
- TOML で STN→PVN をバインドし、置換は TOML 側で行う。

Provider Lock（ロック）
- 起動時に `types.<Type>` の provider を決定し、Provider Lock を作成・固定する。
- Lock 前の `ny_new_box` / `ny_call_method` はエラー（E_PROVIDER_NOT_LOCKED）。
- Handle は `{ type_id, provider_id }` を保持し、デバッグビルドでは不一致検知時に panic（本番では混入しない設計）。

## ポリシー（例）
- `plugin-first`（デフォルト）: 動的プラグインの上書きを許可
- `compat_plugin_first`: 静的→動的のフォールバックを許可（移行期）
- `static_only`（本番）: 静的のみ許可

Interop（同一型の異 Provider 混在）
- 既定は混在禁止（forbid）。同一プロセス内で 1 Type = 1 Provider を維持する。
- 研究・開発用途でのみ `explicit/auto` を許可できるが、本番非推奨。
  - explicit: 明示 API による変換のみ許可（UTF‑8 などの正規形式を介する）
  - auto: 暗黙変換を許可し、変換回数・バイト数をメトリクスに集計（本番非推奨）

## 現状の段階
- 受け口/ドキュメントの整備を先行（挙動は不変）。
- using は SSOT+AST に移行済み（prod は file-using 禁止）。
- VM fallback の個別救済は暫定（短期で Bootstrap Pack へ移行し撤去）。
 - VM fallback（MIR interpreter）の役割は「軽量デバッグ実行器」：フロントエンド（Parser/Using/AST→MIR）の健全性をすばやく確認するために維持。機能は最小限に留め、プラグイン/本流VM/LLVM の実装が主となる（本番・性能評価には使用しない）。

関連ドキュメント
- hako.toml/nyash.toml のスキーマと例: docs/reference/config/nyash-toml.md
- using（SSOT/AST/Profiles）: docs/reference/language/using.md
