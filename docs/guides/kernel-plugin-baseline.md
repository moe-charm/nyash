# Core Kernel vs Plugins — Baseline (Phase 15.7)

Status: active (foundation); Scope: VM/LLVM 共通の設計方針の最小合意

## Goal
- カーネルは最小・安定（GC/Handle/ABI/Loader）。
- それ以外（コレクションや周辺 Box）はプラグインまたはユーザーボックスで差し替え可能。
- 解決順序を一意にし、後からの移行（プラグイン化）を容易にする。

## Layers（責務）
- Kernel（crates/nyash_kernel/）
  - GC/handles（Box ID, Arc<u64> 変換）
  - Host ABI/extern registry（例: time.now_ms 等）
  - Plugin host/loader v2（動的ロード・メタ照会・呼出し橋渡し）
  - NyashBox/BoxCore trait + 共有ユーティリティ
  - Error/Result/Verifier のコア型（Fail‑Fast 基盤）
  - 非対象: 具体的な String/Array/Map 実装（将来も原則保持しない）

- Plugins（.so/.dll, crates/*_plugin）
  - CoreBox 相当の実装提供（String/Array/Map など）
  - 拡張 Box（File/Net/…）
  - ABI 統一（Final ABI 優先、互換あり）

- User Boxes（Nyash コード）
  - アプリや自己ホスト用の薄い箱・適応層（Adapter/Facade）
  - VM 専用の実験的コンポーネント（Selfhost VM の各ハンドラなど）

## Override Chain（優先順）
1) User Box（Nyash）
2) Plugin Box（動的ローダ）
3) Kernel fallback（必要最小）

Notes
- 既存の CoreBox 互換（MapBox/ArrayBox/StringBox）は段階的に Plugin へ移行。Kernel は Null/Missing 等の最小のみ維持を目標。
- `src/box_factory/` の解決は plugin 優先を許容（環境で明示）。互換のため builtin 実装が残っていても、Plugin で上書き可能にする。

## Backend specifics（非カーネル）
- `src/backend/mir_interpreter/handlers/boxes_*.rs` は VM バックエンド固有（Kernel ではない）。
- ここに追加される便宜メソッド（例: substring/charAt）は短期的に OK（Selfhost の bring‑up 用）。将来的には Plugin 側が正式実装。

## Deprecations / Cleanup Plan
- Core String/Array/Map の kernel 側実装は撤退対象（Plugin へ）。
- MapBox.get(missing) の既定は null（2025‑10‑09 既定化）。
- 文字列エラー判定（"Key not found:") を使うロジックは廃止（null チェックへ統一）。

## Acceptance（基盤確認）
- quick スモークが緑（VM/LLVM 代表）
- Plugin あり/なしで解決順が変わらない（User > Plugin > Kernel）
- 自己ホスト最小系（emit-only, M1 bootstrap）が動作

## Pointers（参照）
- Kernel: `crates/nyash_kernel/`
- VM backend handlers: `src/backend/mir_interpreter/handlers/`
- Box factory/override: `src/box_factory/`
- ENV/Policy: `docs/config/env.md`

