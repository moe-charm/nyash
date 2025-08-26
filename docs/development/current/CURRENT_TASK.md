# 🎯 CURRENT TASK - 2025-08-26（Context Reset / Fresh Focus）

コンテキストを「0%」にリセットし、いま必要なことだけに集中するにゃ。

## ⏱️ 今日のフォーカス（Phase 9.79a: Unified Dispatch + P2P Polish）
- 判断: 統一Box設計は「非侵襲のディスパッチ統一」から入る → P2PBox磨きを同時並行
- 目的: ユニバーサルメソッド（toString/type/equals/clone）をVM/Interpreter前段で統一 + P2PBoxのmulti-node/async UX安定化

### 直近の実行タスク（小さく早く）
1) ユニバーサルメソッドの前段ディスパッチ（非侵襲）
   - VM/Interpreterで`toString/type/equals/clone`を共通ヘルパにマップ（トレイト変更なし）
2) P2PBox磨き（multi-node/async/解除）
   - share/cloneセマンティクス：share=共有, clone=新規（実装済みの明文化）
   - unregisterの安全化（endpoint一致 or refcount）
   - onOnce/off のE2Eテスト追加
   - VM表示整合（getLast*/debug_* の toString/Console）
3) E2Eスモーク更新
   - self→self, two-node ping-pong（安定）
   - asyncデモ（TimeBox併用で確実に出力）

### すぐ試せるコマンド（最小）
```bash
# Rust（Release）
cargo build --release -j32
./target/release/nyash --help

# Plugin デバッグ実行（任意）
NYASH_DEBUG_PLUGIN=1 ./target/release/nyash --backend vm local_tests/extern_console_log.nyash || true

# WASM（Web配布）
cd projects/nyash-wasm && wasm-pack build --target web --out-dir pkg
```

## 現在の地図（Done / Doing / Next）

### ✅ 完了
- PluginHostファサード導入・移行（create/invoke/extern）
- TLVヘッダ/引数/ハンドルの共通化（`plugin_ffi_common.rs`）
- Interpreter分割の導線: `eval.rs` / `calls.rs` / `methods_dispatch.rs` 抽出
- ログ静音の基盤: `idebug!`（NYASH_DEBUG=1 で有効）を calls/core/statements に適用
- MIR modular builder ゲート追加（feature: `mir_modular_builder`）/ 整合パッチ投入

### 🚧 進行中（小タスク）
- Interpreterログ統一の残り（`delegation.rs` など）
- PluginHost の `resolve_method` キャッシュ化（I/O削減）

### ⏭️ 次アクション（今日～明日）
- 9.79a-M1: ユニバーサル前段ディスパッチ（VM/Interpreter）/ 回帰確認
- 9.79a-M2: P2P unregister安全化 + onOnce/off E2E + async安定
- 9.79a-M3: VM表示整合/ Docs更新（言語ガイド・P2Pリファレンス）

## 決定事項（Unified Box設計メモ）
- ドキュメント: `docs/ideas/other/2025-08-25-unified-box-design-deep-analysis.md`
- 判断: まずはディスパッチャ層でユニバーサルメソッドを統一（トレイト変更なし）
- P2Pは共有セマンティクス（share=共有, clone=新規）を維持しつつ unregister 正式化へ

## 参考リンク（唯一参照/ゲート）
- MIR命令セット（26命令）: `docs/reference/mir/INSTRUCTION_SET.md`
- Phase 9.79（P2P）: `docs/development/roadmap/phases/phase-9/phase_9_79_p2pbox_rebuild.md`
- Phase 9.79a（Unified Dispatch + P2P Polish）: `docs/development/roadmap/phases/phase-9/phase_9_79a_unified_box_dispatch_and_p2p_polish.md`
- Phase 9.78h（前段完了）: `docs/development/roadmap/phases/phase-9/phase_9_78h_mir_pipeline_stabilization.md`
- Phase 10（Cranelift JIT主経路）: `docs/development/roadmap/phases/phase-10/phase_10_cranelift_jit_backend.md`

## Doneの定義（P2PBox 最小）
- `LocalLoopback` で ping/pong が安定
- P2PBox API（start/stop/send/broadcast/reply/on）が固まる
- ResultBox経由でエラーが伝搬（E2E テスト含む）
- ログは既定静音（環境変数でデバッグオン）

## Parking Lot（後でやる）
- NyashValue enum導入（即値最適化）
- トレイト階層化（Comparable/Arithmetic etc.）
- メタプログラミング・パイプライン演算子
- `mir_modular_builder` をデフォルト化（パリティ後）
