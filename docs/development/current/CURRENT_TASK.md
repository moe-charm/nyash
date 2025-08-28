# 🎯 CURRENT TASK - 2025-08-29（Phase 10.1 キックオフ＋リファクタ）

Phase 10.10 は完了（DoD確認済）。Phase 10.1 に入る前に、JIT Lower 周辺の分割リファクタを小刻みに完了させ、スモークを維持したまま移行します。

## ⏱️ 今日のサマリ
- 目的: 10.1 着手前のリファクタ（機能差分なし）を完了し、Week1を開始する。
- スコープ: `src/jit/lower/core.rs / builder.rs` の整理のみ。挙動変更なし、ビルドとスモークは常にGreenを維持。

## 現在地（Done / Doing / Next）
- ✅ Done（Phase 10.10）
  - GC Switchable Runtime（GcConfigBox）/ Unified Debug（DebugConfigBox）
  - JitPolicyBox（allowlist/presets）/ HostCallのRO運用（events連携）
  - CIスモーク導入（runtime/compile-events）/ 代表サンプル整備
- 🔧 Doing（Refactor before 10.1）
  - `extern_thunks.rs` 抽出済（builder → `src/jit/lower/extern_thunks.rs`）
  - `cfg_dot.rs` 抽出済（core → `src/jit/lower/cfg_dot.rs`）
- ⏭️ Next（Phase 10.1 Kickoff）
  - Week1開始（Python統合の環境・入り口整備）
  - 10.10の回帰はCIスモークで継続監視

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

## Phase 10.1 キックオフ
- 参照: `docs/development/roadmap/phases/phase-10.1/`
- Week1（概要）
  - 10.1a: 計画再確認（I/O境界・GIL/FFI方針）
  - 10.1b: 環境設定（最小ブリッジ・検証手順）
  - 10.1c: パーサー統合の入口作成（Box-Firstで薄く）
  - 10.1d: Core最小経路（Phase 1機能）

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
```

## 参考リンク
- Phase 10.1: `docs/development/roadmap/phases/phase-10.1/README.md`
- Phase 10.10: `docs/development/roadmap/phases/phase-10/phase_10_10/README.md`
- MIR命令セット: `docs/reference/mir/INSTRUCTION_SET.md`

## Checkpoint（再起動用メモ）
- 状態確認: `git status` / `git log --oneline -3` / `cargo check`
- スモーク: `bash tools/smoke_phase_10_10.sh`
- 次の一手: core_hostcall → core_ops の順に分割、毎回ビルド/スモークで確認

