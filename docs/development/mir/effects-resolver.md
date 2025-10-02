# MIR Effects Resolver — 箱化の導入（Phase-15 最小）

目的
- Effects 決定（PURE/READ/IO/CONTROL とレガシー ReadHeap/WriteHeap 等）の散在を解消し、単一の薄い箱に集約する。
- まずは Unified Call (`compute_call_effects`) の前段で、既知の extern/method を表引きで決定。未定義は既存ロジックへフォールバック。

導入ポリシー
- 既定 OFF。`NYASH_USE_EFFECT_RESOLVER=1` のときのみ有効化。
- トレース: `NYASH_EFFECT_TRACE=1` で解決ログを stderr へ出力。
- サニティ検証: `NYASH_VERIFY_EFFECTS=1` で Call/BoxCall/ExternCall に PURE が混ざった場合に警告を出力（軽量 Verifier）。
- 追加ルート実験: `NYASH_USE_CALL_ROUTER=1` で CallRoutingBox 骨格を有効化（TimerBox.now_ms → `nyrt.time.now_ms` 直行）。トレースは `NYASH_CALL_ROUTER_TRACE=1`。
- 仕様不変: 既存の `compute_extern_effects`/既定分岐にフォールバックするため、未知の項目で挙動は変わらない。

既知エントリ（初期）
- Extern
  - `nyrt.time.now_ms` → READ（単調時間の読み取り）
  - `env.console.log` → IO
- Method（最小）
  - `ArrayBox.get/length/size` → READ
  - `ArrayBox.set/push/pop` → READ+WriteHeap

実装位置
- `src/mir/builder/effects/`
  - `resolver.rs`: `EffectResolverBox`（テーブル/トレース/公開 resolve）
  - `mod.rs`: 統合関数 `resolve_effects_for_callee(callee)` と env/trace 配線

統合ポイント
- `src/mir/builder/calls/call_unified.rs:compute_call_effects`
  - 環境 `NYASH_USE_EFFECT_RESOLVER=1` の場合、最初に `effects::resolve_effects_for_callee` を試行 → Some(effects) なら採用。
  - None の場合は従来ロジックへフォールバック（既存挙動を維持）。

検証
- Quick: `core/timer_now_ms_vm.sh` が READ（純粋でない＝CSE対象外）で安定。
- 将来: `NYASH_VERIFY_EFFECTS=1` で PURE 混入の警告を出す軽量 Verifier を追加予定（別PR）。
