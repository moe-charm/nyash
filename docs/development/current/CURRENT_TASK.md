# 🎯 CURRENT TASK - 2025-08-29（Phase 10.1 革新的転換）

Phase 10.10 は完了（DoD確認済）。**重大な発見**：プラグインシステムを活用したJIT→EXE実現の道を発見！

## 🚀 革新的発見：プラグインBox統一化

### 核心的洞察
- 既存のプラグインシステム（BID-FFI）がすでに**完全なC ABI**を持っている
- すべてのBoxをプラグイン化すれば、JIT→EXEが自然に実現可能
- "Everything is Box" → "Everything is Plugin" への進化

## ⏱️ 今日のサマリ（Array/Map プラグイン経路の安定化→10.2へ）
- 実装: Array/Map のプラグイン（BID-FFI v1）を作成し、nyash.toml に統合
- Lower: `NYASH_USE_PLUGIN_BUILTINS=1` で Array(len/get/push/set), Map(size/get/has/set) を `emit_plugin_invoke(..)` に配線
- サンプル: array/map デモを追加し VM 実行で正常動作確認
- 次: 10.2（Craneliftの実呼び出し）に着手

## 現在地（Done / Doing / Next）
- ✅ Done（Phase 10.10）
  - GC Switchable Runtime（GcConfigBox）/ Unified Debug（DebugConfigBox）
  - JitPolicyBox（allowlist/presets）/ HostCallのRO運用（events連携）
  - CIスモーク導入（runtime/compile-events）/ 代表サンプル整備
- 🔧 Doing（Phase 10.1 新計画）
  - ArrayBoxのプラグイン化PoC開始
  - JIT lowering層の統一設計
  - リファクタリング作業は継続（core_hostcall.rs完了）
- ⏭️ Next（Phase 10.1 実装）
  - Week1: ArrayBoxプラグイン化と性能測定
  - Week2: 主要ビルトインBoxの移行
  - Week3: スタティックリンク基盤構築
  - Week4: EXE生成実証

## リファクタリング計画（機能差分なし）
1) core_hostcall 分割（イベントlower＋emit_host_call周辺）
   - 追加: `src/jit/lower/core_hostcall.rs`
   - `mod.rs`/`core.rs` のモジュール参照を更新
   - 確認: `cargo check` → `bash tools/smoke_phase_10_10.sh`
2) core_ops 分割（算術/比較/分岐）
   - 追加: `src/jit/lower/core_ops.rs`
   - CLIF配線やb1正規化カウンタは移動のみ
   - 確認: `cargo check` → 代表JITデモ2本を手動確認
3) 仕上げ
   - 1ファイル ~1000行以内目安を満たすこと
   - ドキュメント差分は最小（本CURRENT_TASKのみ更新）

### DoD（Refactor）
- `cargo check` が成功し、`tools/smoke_phase_10_10.sh` がGreen
- ログ/イベント出力がリファクタ前と一致（体感差分なし）
- `core.rs`/`builder.rs` の行数削減（目安 < 1000）

## Phase 10.1 新計画：プラグインBox統一化
- 参照: `docs/development/roadmap/phases/phase-10.1/` （新計画）
- 詳細: `docs/ideas/new-features/2025-08-28-jit-exe-via-plugin-unification.md`
- Week1（概要）
  - ArrayBoxプラグイン実装とテスト
  - JIT→Plugin呼び出しパス確立
  - パフォーマンス測定と最適化
  
## Phase 10.5（旧10.1）：Python統合
- 参照: `docs/development/roadmap/phases/phase-10.5/` （移動済み）
- ChatGPT5の当初計画を後段フェーズへ

## すぐ試せるコマンド（現状維持の確認）
```bash
# Build（Cranelift込み推奨）
cargo build --release -j32 --features cranelift-jit

# Smoke（10.10の代表確認）
bash tools/smoke_phase_10_10.sh

# HostCall（HH直実行・read-only方針）
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_EVENTS=1 \
  ./target/release/nyash --backend vm examples/jit_map_get_param_hh.nyash
NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 \
  ./target/release/nyash --backend vm examples/jit_policy_whitelist_demo.nyash

# GC counting（VMパス）
./target/release/nyash --backend vm examples/gc_counting_demo.nyash

# compileイベントのみ（必要時）
NYASH_JIT_EVENTS_COMPILE=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_EVENTS_PATH=events.jsonl \
  ./target/release/nyash --backend vm examples/jit_map_get_param_hh.nyash

# Plugin demos（Array/Map）
(cd plugins/nyash-array-plugin && cargo build --release)
(cd plugins/nyash-map-plugin && cargo build --release)
NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend vm examples/array_plugin_demo.nyash
NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend vm examples/array_plugin_set_demo.nyash
NYASH_CLI_VERBOSE=1 ./target/release/nyash --backend vm examples/map_plugin_ro_demo.nyash

## ⏭️ Next（Phase 10.2: JIT実呼び出しの実体化）
- 目的: Craneliftの `emit_plugin_invoke` を実装し、JITでも実体のプラグインAPIを呼ぶ
- 方針:
  - シム関数 `extern "C" nyash_plugin_invoke3_i64(type_id, method_id, argc, a0, a1, a2) -> i64` を実装
    - a0: 受け手（param index／負なら未解決）
    - args: i64 を TLV にエンコードして plugin invoke_fn へ橋渡し
    - 戻り: TLV（i64/Bool）の最初の値を i64 に正規化
  - CraneliftBuilder: `emit_plugin_invoke` で上記シムを import→call（常に6引数）
  - 対象: Array(len/get/push/set), Map(size/get/has/set) の i64 1〜2引数経路
```

## 参考リンク
- Phase 10.1（新）: `docs/development/roadmap/phases/phase-10.1/README.md` - プラグインBox統一化
- Phase 10.5（旧10.1）: `docs/development/roadmap/phases/phase-10.5/README.md` - Python統合
- Phase 10.10: `docs/development/roadmap/phases/phase-10/phase_10_10/README.md`
- プラグインAPI: `src/bid/plugin_api.rs`
- MIR命令セット: `docs/reference/mir/INSTRUCTION_SET.md`

## Checkpoint（再起動用メモ）
- 状態確認: `git status` / `git log --oneline -3` / `cargo check`
- スモーク: `bash tools/smoke_phase_10_10.sh`
- 次の一手: core_hostcall → core_ops の順に分割、毎回ビルド/スモークで確認
