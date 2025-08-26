# 🎯 CURRENT TASK - 2025-08-26（Phase 9.79b Kickoff）

コンテキストを最小化して、次フェーズへの導線だけ残すにゃ。

## ⏱️ 今日のフォーカス（Phase 9.79b → 9.79b.3: Thunks+Poly-PIC）
- 目的: Box（builtin/user/plugin）を数値ID＋スロット＋vtable/thunk統一に移行し、Phase 10(JIT)への足場を確立する。

### 直近タスク（小さく早く）
1) 9.79b.1: Unified Registry IDs + Builder Slotting
   - 型ID/メソッドスロットの導入（レジストリ）✅ 実装
   - ユニバーサルメソッド低スロット予約（0..3）✅ テストで不変確認
   - Builderが解決可能なBoxCallに`method_id`を付与（未解決は遅延）✅ 実装/Printer表示
2) 9.79b.2: VM VTable Thunks + Mono-PIC
   - `execute_boxcall`をvtable+thunkの単一路線へ（ユニバーサル0..3のfast-path追加）✅ スケルトン
   - call-site単位のモノモーフィックPICを追加（Key設計とカウンタ導入・記録まで）✅ スケルトン
   - 次: 安定閾値での直呼び最適化（InstanceBox関数名キャッシュ）✅ 実装（PIC=8で昇格）
   - PluginBoxV2 fast-path（method_id直叩き）✅ 最小TLV（string/int/handle）
   - builtin/plugin/user のslot seed（4〜）✅ lazy seed/予約
   - キャッシュ無効化（version by label）✅ loader/宣言でbump導線

3) 9.79b.3: VM VTable Thunks + Poly-PIC（本実装）
   - TypeMeta＋Thunkテーブル正式化（slot→thunk→target）: builtin/user/plugin 統一（in_progress）
   - PICをpoly（2〜4件）に拡張＋version検証: ヒット/ミス/昇格/evict統計（in_progress）
   - Diagnostics: Registry dump / MIRDebugInfo / PIC・VT統計 / cache bumpログ（in_progress）

### すぐ試せるコマンド
```bash
cargo build --release -j32
./target/release/nyash examples/p2p_self_ping.nyash
./target/release/nyash examples/p2p_ping_pong.nyash
```

## 現在の地図（Done / Next）

### ✅ 完了（9.79a）
- ユニバーサル前段ディスパッチ（toString/type/equals/clone）Interpreter/VM
- P2P unregister安全化・onOnce/off E2E・self/two-nodeスモーク
- IntentBoxのpayload糖衣（MapBox/JSONBox直渡し可）
- Docs: P2Pリファレンス/サンプル

### ⏭️ 次（9.79b）
- 9.79b.1: `phase_9_79b_1_unified_registry_ids_and_builder_slotting.md` ✅ 最小スコープ達成（method_id導入）
- 9.79b.2: `phase_9_79b_2_vm_vtable_thunks_and_pic.md` ✅ ミニマム完了（ユニバーサル/PIC/Plugin fast-path）
- 9.79b.3: `phase_9_79b_3_vm_vtable_thunks_and_pic.md` → 足場固め（TypeMeta/Thunk + Poly-PIC + Diagnostics）

## 統一Box設計メモ（唯一参照）
- `docs/ideas/other/2025-08-25-unified-box-design-deep-analysis.md`
  - 数値ID/スロット/Thunk/PIC/DebugInfoの全体像

## 参考リンク
- MIR命令セット: `docs/reference/mir/INSTRUCTION_SET.md`
- Phase 9.79a（完了）: `docs/development/roadmap/phases/phase-9/phase_9_79a_unified_box_dispatch_and_p2p_polish.md`
- Phase 9.79b（計画）:
  - `docs/development/roadmap/phases/phase-9/phase_9_79b_1_unified_registry_ids_and_builder_slotting.md`
  - `docs/development/roadmap/phases/phase-9/phase_9_79b_2_vm_vtable_thunks_and_pic.md`
- Phase 10（Cranelift JIT主経路）: `docs/development/roadmap/phases/phase-10/phase_10_cranelift_jit_backend.md`

## Parking Lot（後でやる）
- NyashValue即値最適化・演算子特化
- トレイト階層化（Comparable/Arithmetic etc.）
- オブジェクトリテラル糖衣（feature `object_literal`）提案: `docs/ideas/improvements/2025-08-26-object-literal-sugar.md`
