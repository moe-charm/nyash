# Phase 15.7: Known化＋Rewrite統合（dev観測）と Mini‑VM 安定化（dev限定）

目的
- Builderでの Known 化と Instance→Function の統一（Known 経路）を優先し、実行系（VM/LLVM/Ny）を単純化する。
- 早期観測（resolve.try/choose, ssa.phi）を dev‑only で整備し、Union の発生点を特定可能にする。
- 表示APIを `str()` に統一（互換: `stringify()`）し、言語表面のブレを解消する（挙動不変）。
- Mini‑VM（Ny）を安全に安定化（M2/M3代表ケース）。NYABI Kernel は“下地のみ”（既定OFF）。

背景
- Instance→Function 正規化の方針は既定ON。Known 経路は関数化し、VM側は単純化する。
- resolve.try/choose（Builder）と ssa.phi（Builder）の観測は dev‑only で導入済み（既定OFF）。
- Mini‑VM は M2/M3 の代表ケースを安定化（パス/境界厳密化）。
- VM Kernel の Ny 化は後段（観測・ポリシーから段階導入、既定OFF）。

Unified Call（開発既定ON）
- 呼び出しの統一判定は、環境変数 `NYASH_MIR_UNIFIED_CALL` が `0|false|off` でない限り有効（既定ON）。
- メソッド解決/関数化を `emit_unified_call` に集約し、以下の順序で決定:
  1) 早期 toString/stringify→str
  2) equals/1（Known 優先→一意候補; ユーザーBox限定）
  3) Known→関数化（`obj.m → Class.m(me,…)`）／一意候補フォールバック（決定性確保）
- レガシー側の関数化は dev ガードで抑止可能: `NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE=1`（移行期間の重複回避）

スコープ（やること）
1) Builder: Known 化 + Rewrite 統合（Stage‑1）
   - P0: me 注入・Known 化（origin 付与/維持）— 軽量PHI補強（単一/一致時）
   - P1: Known 経路 100% 関数化（obj.m → Class.m(me,…)）。special は `toString→str（互換:stringify）/equals` を統合
   - 観測: resolve.try/choose / ssa.phi を dev‑only で JSONL 出力（既定OFF）。`resolve.choose` に `certainty` を付加し、KPI（Known率）を任意出力（`NYASH_DEBUG_KPI_KNOWN=1`, `NYASH_DEBUG_SAMPLE_EVERY=N`）。

2) 表示APIの統一（挙動不変）
   - 規範: `str()` / `x.str()`（同義）。`toString()` は早期に `str()` へ正規化
   - 互換: `stringify()` は当面エイリアスとして許容
   - QuickRef/ガイドの更新（plus混在の誘導も `str()` に統一）

3) Mini‑VM（MirVmMin）安定化（devのみ）
   - 厳密セグメントによる単一パス化、M2/M3 代表スモーク常緑（const/binop/compare/branch/jump/ret）
   - パリティ: VM↔LLVM↔Ny のミニ・パリティ 2〜3件

4) NYABI（VM Kernel Bridge）下地（未配線・既定OFF）
   - docs/abi/vm-kernel.md（関数: caps()/policy.*()/resolve_method_batch()）
   - スケルトン: apps/selfhost/vm/boxes/vm_kernel_box.nyash（policy スタブ）
   - 既定OFFトグル予約: NYASH_VM_NY_KERNEL, *_TIMEOUT_MS, *_TRACE

非スコープ（やらない）
- 既定挙動の変更（Rust VM/LLVMが主軸のまま）
- PHI/SSAの一般化（Phase 16 で扱う）
- VM Kernel の本配線（観測・ポリシーは dev‑only/未配線）

リスクと軽減策
- 性能: 境界越えは後Phaseに限る（本Phaseは未配線）。Mini‑VMは開発補助で性能要件なし。
- 複雑性: 設計は最小APIに限定。拡張は追加のみ（後方互換維持）。
- 安全: すべて既定OFF。Fail‑Fast方針。再入禁止/タイムアウトを仕様に明記。

受け入れ条件（Acceptance）
- quick: Mini‑VM（M2/M3）代表スモーク緑（const/binop/compare/branch/jump/ret）
- integration: 代表パリティ緑（llvmlite/ハーネス）
- Builder: resolve.try/choose と ssa.phi が dev‑only で取得可能（NYASH_DEBUG_*）
- 表示API: QuickRef/ガイドが `str()` に統一（実行挙動は従前と同じ）
- Unified Call は開発既定ONだが、`NYASH_MIR_UNIFIED_CALL=0|false|off` で即時オプトアウト可能（段階移行）。

実装タスク（小粒）
1. origin/observe/rewrite の分割方針を CURRENT_TASK に反映（ガイド/README付き）
2. Known fast‑path の一本化（rewrite::try_known_rewrite）＋ special の集約
3. 表示APIの統一（toString→str、互換:stringify）— VM ルータ特例の整合・ドキュメント更新
4. MirVmMin: 単一パス化・境界厳密化（M2/M3）・代表スモーク緑
5. docs/abi/vm-kernel.md（下書き維持）・スケルトン Box（未配線）

トグル/ENV（予約、既定OFF）
- NYASH_VM_NY_KERNEL=0|1
- NYASH_VM_NY_KERNEL_TIMEOUT_MS=200
- NYASH_VM_NY_KERNEL_TRACE=0|1

ロールバック方針
- Mini‑VMの変更は apps/selfhost/ 配下に限定（本線コードは未配線）。
- NYABIは docs/ と スケルトンBoxのみ（実行経路から未参照）。
- Unified Call は env で即時OFF可能。問題時は `NYASH_MIR_UNIFIED_CALL=0` を宣言してレガシーへ退避し、修正後に既定へ復帰。

補足（レイヤー・ガード）
- builder 層は origin→observe→rewrite の一方向依存を維持する。違反検出スクリプト: `tools/dev/check_builder_layers.sh`

関連（参照）
- Phase 15（セルフホスティング）: ../phase-15/README.md
- Phase 15.5（基盤整理）: ../phase-15.5/README.md
- Known/Rewrite 観測: src/mir/builder/{method_call_handlers.rs,builder_calls.rs}, src/debug/hub.rs
- QuickRef（表示API）: docs/reference/language/quick-reference.md
- Mini‑VM: apps/selfhost/vm/boxes/mir_vm_min.nyash
- スモーク: tools/smokes/v2/profiles/quick/core/

更新履歴
- 2025‑09‑28 v2（本書）: Known 化＋Rewrite 統合（dev観測）、表示API `str()` 統一、Mini‑VM 安定化へ焦点を再定義
- 2025‑09‑28 初版: Mini‑VM M3 + NYABI下地の計画
