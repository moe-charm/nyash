# Current Task (2025-09-11) — Phase 15 LLVM‑only

Summary
- LLVM is the authoritative path; VM/Cranelift/Interpreter are not MIR14‑ready.
- Keep fallbacks minimal; fix MIR annotations first.
- ExternCall(console/debug) auto‑selects ptr/handle by IR type.
- StringBox NewBox i8* fast path; print/log choose automatically.
- Implement multi-function lowering and Call lowering for MIR14.

Compact Roadmap (2025‑09‑12)
- Focus: LLVM AOT → Flow hardening, PHI(sealed)安定化, LoopForm導入, BuilderCursor厳格化。
- Now:
  - Fallback terminator整備、PHI(sealed)はsnapshot参照へ、castはpred終端直前に限定。
  - LoopForm Step 2.5/3（検出2段/dispatch骨格）完了。非破壊（Break集約のみ）。
  - BuilderCursor: post‑terminator挿入を即panic（strings/arith_ops/memへ適用済）。
- Next (short):
  1) BuilderCursor厳格化の適用拡大（externcall→newbox→arrays→maps→call）。
  2) Sealed SSA を既定ONに一本化（finalize_phis停止、seal_blockで完結）。NYASH_LLVM_PHI_SEALED は未設定時=ON。
  3) LoopForm header PHI正規化の安定化（latch→header ON 時も verifier green）。
  4) body→dispatchを単純ボディで常用化（段階ゲート）。
  5) 計測: dispatch-only PHI/ゼロ合成減少、post‑terminator検知ゼロ継続。
- Flags:
  - `NYASH_ENABLE_LOOPFORM=1`（非破壊ON）
  - `NYASH_LOOPFORM_BODY2DISPATCH=1`（実験: 単純ボディのbody→dispatch）
  - `NYASH_LOOPFORM_LATCH2HEADER=1`（PHI正規化後に有効化）

Update — 2025-09-12 (LLVM flow + BB naming)
- codegen/mod.rs match arms cleanup:
  - BinOp: unify to instructions::lower_binop(...)
  - BoxCall: delegate solely to instructions::lower_boxcall(...); removed unreachable legacy code
- BasicBlock naming made function‑scoped and unique: create_basic_blocks now prefixes with function label (e.g., Main_join_2_bb23)
- Terminator fallback: when a MIR block lacks a terminator, emit a conservative jump to the next block (or entry if last)
- Build: cargo build --features llvm passes
- AOT emit status: improved (bb name collision resolved), but verifier still flags a missing terminator in Main.esc_json/1 (e.g., Main_esc_json_1_bb88)
  - Likely cause: flow lowering edge case; MIR dump is correct, LLVM lowering missed a terminator
  - Plan below addresses this with hardened flow lowering and instrumentation

Hot Update — 2025-09-12 (quick)
- Flow: moved function verify to post‑lower; added terminator fallback only when MIR lacks one
- PHI(sealed): seal_block isolates cast insertion by saving/restoring the insertion point in the predecessor block and wires incoming only for the current predecessor (zero-synth as a last resort, logged). Avoids duplicate incoming and builder state leaks.
- Compare: allow ptr↔int comparisons for all ops via ptr→i64 bridge
- Strings: substring now accepts i64(handle) receiver (i2p); len/lastIndexOf stable
- Arrays: method_id未注入でも get/set/push/length を NyRT 経由で処理
- BoxCall: println→env.console.log フォールバック、同モジュール関数呼び出し/名前指定 invoke_by_name 経路を追加
- PHI: 文字列/Box等を含む場合は i8* を選択する型推定に改善
- 現在のブロッカー: esc_json/1 で「phi incoming value missing」
  - 対応: emit_jump/emit_branch の incoming 配線をログ付きで点検し、値未定義箇所（by‑name/fast‑path戻り）を補完

Hot Update — 2025‑09‑12 (Plan: LLVM wrapper via Nyash ABI)
- 背景: Rust/inkwell のビルド時間と反復速度が課題。LLVM生成を Nyash から呼べる ABI に抽象化し、将来 Nyash スクリプトで LLVM ビルダー実装へ移行する。
- 方針: Rust 実装は当面維持（1–2日で dep_tree_min_string をグリーンに）。併走で llvmlite(Python) を「検証ハーネス」として導入し、PHI/Loop 形の仕様検証→ Rust へ反映。
- 入口: Nyash ABI で LLVM emit をラップ。
  - モード切替フラグ: `NYASH_LLVM_USE_HARNESS=1`（ON時は llvmlite ハーネスに委譲）。
  - I/O 仕様: 入力=MIR(JSON/メモリ), 出力=.o（`NYASH_AOT_OBJECT_OUT` に書き出し）。
- 受け入れ: harness ON/OFF で dep_tree_min_string の出力一致（機能同値）。

Scaffold — 2025‑09‑12 (llvmlite harness)
- Added tools/llvmlite_harness.py (trivial ny_main returning 0) and docs/LLVM_HARNESS.md.
- Use to validate toolchain wiring; extend to lower MIR14 JSON incrementally.

Scaffold — 2025‑09‑12 (Resolver i64 minimal)
- Added src/backend/llvm/compiler/codegen/instructions/resolver.rs with `Resolver::resolve_i64(...)` and per-block cache.
- docs/RESOLVER_API.md documents goals/usage; wiring to replace `localize_to_i64` callsites comes next.

Hot Update — 2025‑09‑12 (Structural Invariants v1)
- Core invariants adopted to eliminate dominance violations structurally:
  - Resolver-only value access: forbid direct `vmap.get(...)` in lowering; always use `Resolver::{resolve_i64, resolve_ptr, resolve_f64}`.
  - Localization discipline: PHIs are created at BB head; casts/incoming coercions are inserted at predecessor end via `BuilderCursor::with_block` (never inside PHI site).
  - Strings rule: across blocks keep StrHandle(i64) only; convert to StrPtr(i8*) at call sites inside the same BB. Return values of string ops are stored as i64 handles.
  - LoopForm rule: preheader is mandatory; header condition resolved via Resolver; dispatch-only PHI; optional latch→header normalization is gated.
  - BuilderCursor guard: deny post‑terminator insertion; avoid raw builder calls from lowering sites.
- Acceptance A2.5 (added): sealed=ON (default) with dominator violations = 0 on `apps/selfhost/tools/dep_tree_min_string.nyash`.

Next Plan（精密タスク v2）
- Deny-Direct purge: eliminate remaining `vmap.get(...)` in lowering modules（maps.rs, arith.rs, arith_ops.rs, externcall/env.rs, boxcall/{invoke,marshal,fields}.rs, mem.rs ほか）。
- Enforce string handle invariant end-to-end（helpers/Resolverに軽量StrHandle/StrPtr型or helperを追加、戻りは常にi64、呼び出し直前だけinttoptr）。
- L-Dispatch-PHI checker（dev）: dispatch以外のPHI検出でpanic/ログ。preheader必須の検証も追加。
- Seal配線の堅牢化: snapshot優先・pred末端cast挿入の徹底、フォールバックゼロ合成の縮小（非param値のvmapフォールバックを禁止）。
- Regression: `dep_tree_min_string` オブジェクト生成→ LLVM verifier green（支配違反ゼロ）; Deny-Direct grep=0 をCIチェックに追加。

Plan — Context Boxing（Lower APIの“箱化”）
- 背景: 降下APIの引数が過多（10+）で、責務が分散しがち。Nyashの“箱理論”に従い、関連情報をコンテキストBoxにまとめて境界を作る。
- 目的: 責務の一元化・不変条件の型による強制・引数爆発の解消・再発防止。
- 主要な“箱”
  - LowerFnCtx<'ctx, 'b>（関数スコープ）
    - 保持: `codegen`, `func`, `cursor`, `resolver`, `vmap`, `bb_map`, `preds`, `block_end_values`, `phis_by_block`, `const_strs`, `box_type_ids` など
    - 役割: 関数内の全 lowering に共通する情報とユーティリティ（ensure_i64/i8p/f64、with_block ラッパ 等）
  - BlockCtx<'ctx>（ブロックスコープ）
    - 保持: `cur_bid`, `cur_llbb`、必要に応じて `succs`
    - 役割: その場の挿入点管理、pred終端直前挿入の窓口
  - InvokeCtx（呼び出し情報）
    - 保持: `method_id`, `type_id`, `recv_h`, `args`
    - 役割: plugin invoke（by‑id/by‑name）の統一入口
  - StringOps（軽量型/ヘルパ）
    - 型: `StrHandle(i64)`, `StrPtr(i8*)`
    - 規約: ブロック間は StrHandle のみ／call直前のみ StrPtr 生成。戻りは常に StrHandle

- API の最終形（例）
  - `lower_boxcall(ctx: &mut LowerFnCtx, blk: &BlockCtx, inst: &BoxCallInst) -> LlResult<()>`
  - `try_handle_tagged_invoke(ctx: &mut LowerFnCtx, blk: &BlockCtx, call: &InvokeCtx) -> LlResult<()>`
  - `StringOps::concat(ctx, blk, lhs, rhs) -> LlResult<StrHandle>`

- 構造の不変条件との対応付け
  - Resolver‑only: `LowerFnCtx` 経由でのみ値取得可能（VMap の直接 `get` は不可にする）
  - 局所化の規律: `BlockCtx.with_block_pred(..)` 等で pred末端挿入を強制
  - dispatch‑only PHI: dev チェッカ `PhiGuard::assert_dispatch_only` を追加
  - preheader必須: LoopForm 生成時に assert（dev）

- マイグレーション手順（段階的）
  1) `LowerFnCtx/BlockCtx/InvokeCtx` を導入し、`lower_boxcall` と invoke 経路を最初に箱化
  2) Strings を `StringOps` に集約（戻り=StrHandle）。既存callサイトから直 i8* を排除
  3) BinOp/Compare/ExternCall を順次 `LowerFnCtx+BlockCtx` 受けに移行
  4) dev チェッカ（dispatch‑PHI/preheader）をONにし、構造を固定

- 受け入れ（Context Boxing 対応）
  - `lower_boxcall`/invoke/strings が “箱化API” を使用し、関数シグネチャの引数が 3 箱以内
  - Resolver‑only/Deny‑Direct 維持（grep 0）
  - 代表ケースで verifier green（D‑Dominance‑0）

Docs — LLVM layer overview (2025‑09‑12)
- Added docs/LLVM_LAYER_OVERVIEW.md and linked it with existing docs:
  - docs/LOWERING_LLVM.md — concrete lowering rules and RT calls
  - docs/RESOLVER_API.md — value resolution (sealed/localize) API and cache
  - docs/LLVM_HARNESS.md — llvmlite harness scope and interface
  Use as the canonical reference for invariants: Resolver-only reads, cast placement, cursor discipline, sealed SSA, and LoopForm shape.

Hot Repro — esc_json/1 PHI 配線（2025‑09‑12）
- 対象: apps/selfhost/tools/dep_tree_min_string.nyash
- 実行（LLVM）:
  - Sealed OFF: `NYASH_CLI_VERBOSE=1 NYASH_LLVM_TRACE_PHI=1 NYASH_DISABLE_PLUGINS=1 ./target/release/nyash --backend llvm apps/selfhost/tools/dep_tree_min_string.nyash`
  - Sealed ON:  `NYASH_LLVM_PHI_SEALED=1 NYASH_CLI_VERBOSE=1 NYASH_LLVM_TRACE_PHI=1 NYASH_DISABLE_PLUGINS=1 ./target/release/nyash --backend llvm apps/selfhost/tools/dep_tree_min_string.nyash`
- 観測:
  - Sealed OFF: Main.esc_json/1 で PHI incoming 不足（`PHINode should have one entry for each predecessor`）。
  - Sealed ON: `phi incoming (seal) value missing`（pred側終端値の取得ができていない）。別途 Terminator 欠落も検知→終端フォールバックを実装して解消済み。
- 原因仮説: Sealed ON で `seal_block` が pred終端時点の値（value_at_end_of_block）ではなく関数作業用 vmap を参照しているため、未定義扱いになっている。

Next Steps（Sealed SSA 段階導入）
1) block_end_values を導入し、各BB降下完了時に vmap スナップショットを保存。`seal_block` は pred のスナップショットから in_vid を取得。（完了）
2) Sealed=ON を既定にし、emit_* 側の配線を停止（`finalize_phis` 無効化）。（実装済/整備中）
3) BuilderCursor を lowering 全域に適用（externcall/newbox/arrays/maps/call）。
4) Sealed=ON で apps/selfhost/tools/dep_tree_min_string.nyash を再確認（PHIログ=ON）。
5) 足りない型整合（String/Box/Array→i8*）があれば `coerce_to_type` を拡張。
6) グリーン後、LoopForm BODY→DISPATCH を単純ボディで常用化。

TODO — Sealed SSA 段階導入（実装タスク）
- [x] block_end_values 追加（LLVM Lower 内の per-BB 終端スナップショット）
  - 追加先: `src/backend/llvm/compiler/codegen/mod.rs`
  - 形式: `HashMap<BasicBlockId, HashMap<ValueId, BasicValueEnum>>`
  - タイミング: 各BBの命令をすべて Lower した「直後」、終端命令を発行する「直前」に `vmap.clone()` を保存
  - 目的: `seal_block` で pred 終端時点の値を安定取得する（現在の vmap 直接参照をやめる）
- [x] `seal_block` をスナップショット参照に切替
  - 対象: `src/backend/llvm/compiler/codegen/instructions/flow.rs::seal_block`
  - 取得: `block_end_values[bid].get(in_vid)` を用いて `val` を取得
  - フォールバック: もしスナップショットが無ければ（例外ケース）従来の `vmap` を参照し、警告ログを出す
  - ログ: `NYASH_LLVM_TRACE_PHI=1` 時に `[PHI] sealed add pred_bb=.. val=.. ty=.. (snapshot)` と明示
- [x] 非 sealed 経路の維持（回帰防止）
  - `emit_jump/emit_branch` は sealed=OFF の時のみ incoming を追加（現状仕様を維持）
  - sealed=ON の時は incoming 配線は一切行わず、`seal_block` のみで完結
- [x] 型整合（coerce）の継続強化
  - 対象: `src/backend/llvm/compiler/codegen/instructions/flow.rs::coerce_to_type`
  - 方針: PHI の型は i8* 優先（String/Box/Array を含む場合）。ptr/int 混在は明示 cast で橋渡し
  - 検討: i1 ブリッジ（bool）の zext/trunc の置き場所は PHI 外側に寄せる（必要時）
- [ ] 代表スモークの回帰
  - 再現対象: `apps/selfhost/tools/dep_tree_min_string.nyash`
  - 実行: `NYASH_LLVM_PHI_SEALED=1 NYASH_LLVM_TRACE_PHI=1 NYASH_DISABLE_PLUGINS=1 ./target/release/nyash --backend llvm apps/selfhost/tools/dep_tree_min_string.nyash`
  - 期待: `PHINode should have one entry for each predecessor` が解消し、OFF/ON で等価な結果

補足（実装メモ）
- `block_end_values` の寿命はコード生成のライフタイムに束縛されるため、`BasicValueEnum<'ctx>` の所有は問題なし（`Context` が生きている間は有効）
- 収集は `compile_function` の BB ループ内で行い、`phis_by_block` と同スコープで管理すると取り回しが良い
- 将来の拡張として `value_at_end_of_block(var, bb)` ヘルパを導入し、sealed/unsealed を内部で吸収する API 化を検討

Hot Plan — Structured Terminators（RAII BuilderCursor）
- 目的: 末端処理を設計で保証（未終端や終端後挿入を構造で不可能に）し、終端まわりのフォールバック重複を削減する
- ポリシー: 「開いているブロックにのみ命令を挿入できる。終端を置いた瞬間に閉じる。閉じたブロックへの挿入は即panic」

TODO — BuilderCursor 導入と段階適用
- [ ] 新規: `builder_cursor.rs` を追加（`BuilderCursor` / `BlockState` / `with_block`）
  - API: `at_end(bid, llbb)`, `emit_instr(f)`, `emit_term(f)`, `assert_open(bid)`
  - 実装: inkwell::Builder を参照で保持し、`closed_by_bid: HashMap<BasicBlockId,bool>` を管理
- [ ] 既存終端APIを置換
  - 変更: `emit_return / emit_jump / emit_branch` を `BuilderCursor::emit_term` 経由に
  - 呼び出し元: codegen/mod.rs 側から `&mut BuilderCursor` を渡す
- [ ] 位置ずれの解消（最小）
  - cast 等の補助命令は「pred終端直前」へ（position_before）を維持
  - 既存の entry_builder 経路で位置ずれが起きないよう、必要箇所のみ `with_block` を使用
- [ ] 関数終端の最終保証
  - `finalize_function`: 未終端BBには `unreachable` を挿入（最後の砦）
- [ ] 代表スモークの回帰（Sealed=ON）
  - 期待: 未終端エラー・終端後挿入エラーの消滅。PHI配線は snapshot により安定

Note
- フェーズ1では「終端APIと位置ずれの構造化」の最小適用に留め、フォールバック（最終unreachable）を併用
- フェーズ2で with_block の適用範囲を広げ、余剰なガード・分岐フォールバックを削除していく（ソースは小さくシンプルに）

Progress — 2025-09-12（Sealed + RAII 最小導入）
- Sealed: block_end_values（BB内定義のみをフィルタ）を導入し、incoming は pred 終端時点の snapshot から配線
- Cast 挿入: pred 終端直前（position_before）に限定、終端後挿入を回避
- BuilderCursor: emit_return/emit_jump/emit_branch を構造化（closed ブロックへの挿入を禁止）
- Call/BoxCall: 実引数を callee の型へ coerce（i2p/p2i/zext/trunc 等）
- Const String: nyash_string_new を entry ブロックで Hoist し、支配性違反を解消
- Fallback（暫定）: PHI 欠落や lhs/rhs missing に対し型ゼロを合成して進行（ログ付）

Open Issues（要対応）
- Sealed配線: 一部合流で『各predに1 incoming』が未充足（synth で穴埋め中）
- Dominance: ループ/合流で稀に「Instruction does not dominate all uses!」が再出
- 位置ずれ: Sealed 内の cast 生成が builder の挿入位置に影響する可能性（要隔離）

Next TODO（優先度順）
1) flow::seal_block の挿入位置を完全隔離
   - 専用Builder or Cursor.with_block で pred 終端直前に cast を挿入（メインbuilder位置を汚さない）
2) preds_by_block を構築し、PHI incoming を実CFGの pred へ正規リマップ
   - snapshot から in_vid を取得し、pred 数ぶんを網羅（synth は最終手段）
   - 検証: incoming=pred数 を assert/ログ
3) with_block の適用拡大（entry_builder/配列alloca等のホットスポット）
   - 位置ずれ温床を解消 → 余剰ガード/フォールバックを削除（コード縮小）
4) 回帰: Sealed=ON/OFF の一致確認（dep_tree_min_string ほか代表）
   - NYASH_LLVM_TRACE_PHI=1 で配線ログ確認、ゼロ合成が消えることを目標

Plan — PHI/SSA Hardening (Sealed SSA)
- Sealed SSA 入れ替え（安全に段階導入）
  - Blockごとに `sealed: bool` と `incomplete_phis: Map<Var, Phi>` を保持
  - 値取得APIを一本化: `value_at_end_of_block(var, bb)` で vmap を再帰解決＋仮PHI生成
  - `seal(bb)` で pred 確定後に incomplete_phis を埋め、`fold_trivial_phi` で単純化
- 配線の方向を整理
  - emit_jump/emit_branch では直接 incoming を配線しない（to 側で必要時に解決）
  - fast/slow 両レーンは同じ Var に書く（合流で拾えるように）
- 型の前処理を一箇所へ
  - Bool は i1 固定（必要なら zext は PHI 外側）
  - ptr/int 混在禁止。必要箇所で ptrtoint/inttoptr を生成し、PHI では等型を保証
- 計測と検証
  - `seal(bb)` 時に incomplete_phis が残れば panic（場所特定）
  - `[PHI] add incoming var=.. pred=.. ty=..` をデバッグ出力
  - verify は関数降下完了後に 1 回
- フォールバックのゲート
  - by-name invoke は環境変数で明示ON（デフォルトOFF）にする方針に切替（ノイズ低減）

Refactor Policy
- 慌てず小さな箱（モジュール）を積む。必要なら随時リファクタリングOK。
- 代替案（必要時のみ）: llvmlite の薄層で最低限の命令面を実装し、dep_tree_min_string を先に通す。

Done (today)
- BoxCall legacy block removed in LLVM codegen; delegation only.
- BinOp concat safety: minimal fallback for i8*+i64/i64+i8* when both sides are annotated String/Box → from_i8_string + concat_hh; otherwise keep concat_ss/si/is.
- String fast‑path: length/len lowered to nyash.string.len_h (handle), with ptr→handle bridge when needed.
- Map core‑first: NewBox(MapBox) routes via nyash.env.box.new("MapBox") → NyRT特例でコアMapを生成。LLVM BoxCall(Map.size/get/set/has) は NyRT の nyash.map.* を呼ぶ。
- Plugin強制スイッチ: NYASH_LLVM_FORCE_PLUGIN_MAP=1 で MapBox のプラグイン経路を明示切替（デフォルトはコア）。
- Docs: ARCHITECTURE/LOWERING_LLVM/EXTERNCALL/PLUGIN_ABI を追加・整備。
- Smokes: plugin‑ret green, map smoke green（core‑first）。
- ExternCall micro‑split 完了: `externcall.rs` を `externcall/` ディレクトリ配下に分割し、
  `console.rs`（console/debug）と `env.rs`（future/local/box）に切り出し。
  ディスパッチは `externcall/mod.rs` に集約（挙動差分なし・0‑diff）。
 - String Fast‑Path 追加（LLVM/NYRT）: `substring(start,end)` と `lastIndexOf(needle)` を実装。
 - Compare: String/StringBox 注釈のときは内容比較（`nyash.string.eq_hh`）にブリッジ。
- LLVM 多関数 Lower の骨格を実装: 全関数を事前宣言→順次Lower→`ny_main` ラッパで呼び出し正規化。
- Call Lowering 追加（MIR14の `MirInstruction::Call`）: callee 文字列定数を解決し、対応するLLVM関数を呼び出し（引数束縛・戻り値格納）。
- BuilderCursor 適用（第1弾）: strings/arith_ops/mem を Cursor 経由に統一。post-terminator 挿入検知を強化。
- Sealed SSA: `finalize_phis` を停止し、`seal_block` に一本化。LoopForm latch→header の header PHI 正規化を追加（ゲート付）。

Hot Update — 2025‑09‑12 (sealed + dominator 修正の途中経過)
- BuilderCursor 全域化 拡大（第一波 完了）
  - externcall(console/env), newbox, arrays, maps, call, compare を Cursor 経由へ移行
  - 既存 strings/arith_ops/mem とあわせて、ほぼ全 lowering が post‑terminator 防止のガード下に
- on‑demand PHI（局所化）導入（flow::localize_to_i64）
  - 目的: 「現在BBで利用する i64 値」を pred スナップショットから PHI 生成して局所定義に置換→支配関係違反を解消
  - 生成位置: BB 先頭（既存PHIの前）に挿入。挿入点は保存/復元
  - 適用先: strings.substring の start/end、strings.concat の si/is、compare の整数比較、flow.emit_branch の条件（int/ptr/float→i1）
- 失敗時IRダンプ: `NYASH_LLVM_DUMP_ON_FAIL=1` で `tmp/llvm_fail_<func>.ll` を出力（関数検証失敗時）

Hot Update — 2025‑09‑12 (Resolver 適用拡大 + sealed 既定ON)
- Resolver: i64/ptr/f64 を実装（per‑functionキャッシュ＋BB先頭PHI）。
- 適用: emit_branch 条件、strings(substring/concat si|is)、arith_ops(整数演算)、compare(整数比較)、externcall(console/env)、newbox(env.box.new_i64)、call の引数解決を Resolver 経由に統一。
- 非sealed配線の削除: emit_jump/emit_branch 内の直接incoming追加を撤去。sealedスナップショット＋Resolverの需要駆動で一本化。
- 既定: `NYASH_LLVM_PHI_SEALED` 未設定=ON（`0` のみOFF）。

Status — VMap Purge (Phase 1 done)
- Done (Resolver化済み):
  - flow: branch 条件（i64→i1）
  - strings: substring/concat(si|is)、lastIndexOf の needle
  - arith_ops: 整数演算の左右オペランド（i64 正規化）
  - compare: 整数比較の左右（i64 正規化）
  - externcall: console(log/warn/error/trace) は handle 経路に統一（resolve_i64）。future.spawn_instance 名は resolve_ptr。
  - env.box.new_i64: int/ptr 引数は resolve_i64（f64→from_f64）
  - arrays/maps: index/key/value の整数/ptr は resolve_i64
  - call: 実引数を callee 期待型へ（int→resolve_i64、ptr→i64→i2p、float→resolve_f64）
  - mem.store: 値を resolve_i64/resolve_f64/resolve_ptr の順で解決
  - boxcall: recv を i64/ptr 両形で取得、direct call 引数を Resolver 経由に統一

- Remaining (vmap.get 参照の置換ターゲット):
  - flow.emit_return: 戻り値の型に応じて resolve_* に統一（シグネチャ拡張）
  - loopform.lower_while_loopform: 条件の vmap 直参照→ resolve_i64（シグネチャ拡張）
  - strings.try_handle_string_method: 一部 vmap 残存箇所の整理（recv は boxcall 側で ptr 統一済み）
  - extern.env: env.box.new の型名（arg0）や一部名引数を resolve_ptr に統一
  - arith/arith_ops/compare: 局所の vmap 判定/ゼロフォールバックの縮小（Resolver 経由へ）
  - snapshot 用の vmap アクセス（compile_module 内のスナップショット作成）は維持（仕様上の例外）

Plan — Next (precise)
1) flow.emit_return を Resolver 化（resolve_i64/resolve_f64/inttoptr へ）。
2) loopform.lower_while_loopform に Resolver/CFG を渡し、条件解決を resolve_i64 に統一。
3) strings の vmap 残を metadata + Resolver へ置換（concat rhs 判別の簡素化）。
4) extern.env.* 名引数を resolve_ptr 化（local.get / box.new 名など）。
5) marshal/fields の vmap 読みを Resolver/型注釈へ段階置換（最小に縮退させる）。
6) LoopForm: preheader 既定化 + 最小 LoopState（tag+i64）導入→ dispatch-only PHI 完了（ゲート）。

Smoke（sealed=ON, dep_tree_min_string）所見
- 進展: PHI 欠落は再現せず、sealed での incoming 配線は安定
- 依然NG: Main.node_json/3 で dominator 違反（Instruction does not dominate all uses!）
  - iadd→icmp/sub/substring/concat 連鎖の一部で、iadd 定義が利用点を支配していない
  - 対応済: 分岐条件/整数比較/substring/concat の整数引数は局所化済み
  - まだの可能性が高い箇所: そのほかの lowering 内で vmap 経由の整数使用（BoxCall/ExternCall/arith_ops 内の再利用点など）

Next（引き継ぎアクション）
1) 局所化の適用拡大（優先）
   - vmap から整数値を読み出して利用する全パスで `localize_to_i64` を適用
   - 候補: arith_ops（BinOpのオペランド再利用箇所）、BoxCall の残りの整数引数、他メソッドの整数パラメータ
   - types.to_bool 直叩きは emit 側での「局所化→!=0」に段階移行
2) Resolver API の一般化
   - 「ValueId→現在BBの値」を返す resolver を導入（まず i64、必要に応じて ptr/f64 へ拡張）
   - 全 lowering から resolver 経由で値取得し、支配関係崩れを根本排除
3) IR 可視化/検証強化
   - 失敗関数の .ll を確認し、局所化漏れの使用点を特定→順次塞ぐ
4) 併走: llvmlite 検証ハーネス（`NYASH_LLVM_USE_HARNESS=1`）
   - PHI/loop/短絡の形を高速に検証→Rust 実装へ反映（機能一致を Acceptance A5 で担保）

Refactor — LLVM codegen instructions modularized (done)
- Goal achieved: `instructions.rs` を段階分割し、責務ごとに再配置（0‑diff）。
- New layout under `src/backend/llvm/compiler/codegen/instructions/`:
  - Core: `blocks.rs`（BB生成/PHI事前作成）, `flow.rs`（Return/Jump/Branch）, `consts.rs`, `mem.rs`, `arith.rs`（Compare）
  - BoxCall front: `boxcall.rs`（ディスパッチ本体）
    - Strings/Arrays/Maps fast‑paths: `strings.rs`, `arrays.rs`, `maps.rs`
    - Fields: `boxcall/fields.rs`（getField/setField）
    - Tagged invoke: `boxcall/invoke.rs`（method_idありの固定長/可変長）
    - Marshal: `boxcall/marshal.rs`（ptr→i64, f64→box→i64, tag分類）
  - Arith ops: `arith_ops.rs`（Unary/Binary、文字列連結の特例含む）
  - Extern: `externcall.rs`（console/debug/env.local/env.box.*）
  - NewBox: `newbox.rs`（`codegen/mod.rs` から委譲に一本化）
- Wiring: `instructions/mod.rs` が `pub(super) use ...` で再エクスポート。可視性は `pub(in super::super)`/`pub(super)` を維持。
- Build: `cargo build --features llvm` グリーン、挙動差分なし。

Next (short, focused)
- Call Lowering の分離（完了）: `instructions/call.rs` に分割し、`mod.rs` から委譲。
- 多関数 Lower 検証: selfhost minimal（dep_tree_min_string）を LLVM で通す（必要なら型注釈の微調整）。
- Flow lowering hardening (in progress next):
  - Ensure every lowered block has a terminator; use builder.get_insert_block().get_terminator() guard before fallback
  - Instrument per‑block lowering (bid, has terminator?, emitted kind) to isolate misses
  - Keep fallback minimal and only when MIR.block.terminator is None and LLVM has no terminator
  - Detection strengthened (LoopForm Step 2.5): while-pattern detection allows 2-step Jump chains and selects body/after deterministically; logs include loop_id and chosen blocks.

LoopForm IR — Experimental Plan (gated)
- Goal: Centralize PHIs and simplify terminator management by normalizing loops to a fixed block shape with a dispatch join point.
- Gate: `NYASH_ENABLE_LOOPFORM=1` enables experimental lowering in LLVM path (MIR unchanged in Phase 1).
- Representation: Signal-like pair `{ i8 tag, i64 payload }` (0=Next,1=Break initially). Payload carries loop value (Everything is Box handle or scalar).
- Pattern (blocks): header → body → branch(on tag) → dispatch(phi here only) → switch(tag){ Next→latch, Break→exit } → latch→header.
- Phase 1 scope: while/loop only; Return/Yield signalization deferred.
- Success criteria: PHIs appear only in dispatch; no post-terminator insertions; Sealed ON/OFF equivalence; zero-synth minimized.
- Files:
  - New: `src/backend/llvm/compiler/codegen/instructions/loopform.rs` scaffolding + helpers
  - Wire: `instructions/mod.rs` to expose helpers (not yet used by default lowering)
- MIR readable debug tools:
  - Add --dump-mir-readable to print Nyash‑like pseudo code per function/block
  - Optional DOT output (follow‑up)
- Debug hints in MIR (debug builds only):
  - Add #[cfg(debug_assertions)] fields like MirInstruction::debug_hint and MirMetadata::block_sources
  - Gate emission by env (NYASH_MIR_DEBUG=1)
- Map コア専用エントリ化（env.box.new 特例整理）と代表スモークの常時化（CI）
- types.rs の将来分割（任意）:
  - `types/convert.rs`（i64<->ptr, f64→box）, `types/classify.rs`, `types/map_types.rs`
- 機能拡張（任意・別タスク）:
  - String AOT fast‑paths拡充（substring/indexOf/replace/trim/toUpper/toLower）
  - MapBox 生成のNyRT専用エントリ化（env.box.new特例の解消）
  - Map.get の戻り型注釈の厳密化
  - 代表スモークの追加とCI常時チェック

Risks/Guards
- Avoid broad implicit conversions; keep concat fallback gated by annotations only.
- Ensure nyash.map.* との一致（core Map）; plugin経路は環境変数で明示切替。
- Keep LLVM smokes green continuously; do not gate on VM/JIT.
- BuilderCursor 全域適用前は `codegen.builder` の直接使用が残存し、挿入点の撹乱によるドミナンス違反のリスクあり（対策: 全域 Cursor 化）。

Hot Update — 2025-09-12 (LoopForm Step 2.5/3)
- Context reset: コンテキスト問題でひらきなおし。LoopForm を安全な骨格から段階導入する。
- While 検出強化（Step 2.5）: then/else → header への back-edge を Jump 2 段まで許容し、短い側を body、他方を after に決定（ログに header/body/after/loop_id を表示）。
- dispatch 骨格（Step 3 最小）: dispatch に phi(tag:i8, payload:i64) を作り、switch(tag){ Next(0)→latch, Break(1)→exit } を実装。
  - いまは header(false)=Break のみを dispatch に供給（body→dispatch は既定OFFのまま・安全導入）。
  - latch は unreachable（header の pred を増やさず PHI 整合を保つ）。
- BuilderCursor 強化（局所）: at_end で終端検知/closed 初期化、emit_instr で post-terminator 挿入を即 panic。
- 互換: MIR Const Void は i64(0) に無害化して Lower 継続性を向上。

LoopForm Flags（実験）
- `NYASH_ENABLE_LOOPFORM=1`: LoopForm 検出/配線を有効化（非破壊・Break 集約のみ）。
- `NYASH_LOOPFORM_BODY2DISPATCH=1`: 単純ボディの Jump→header を dispatch へ差替え（tag=0/payload=0 を追加）。
- `NYASH_LOOPFORM_LATCH2HEADER=1`: latch→header を有効化（現状は推奨OFF。header PHI 正規化後にONする）。

Next Flow（これからの流れ＝段階導入）
1) BuilderCursor 厳格化の適用拡大（短期）
   - 直叩き `build_*` を `emit_instr/emit_term/with_block` に段階置換（strings → arith_ops → mem → types）。
   - 軽量トラッカーで post-terminator 挿入を即検知（panic、犯人BB特定）。
2) LoopForm 反復の本線（中期）
   - header PHI 正規化（LoopForm 追加predを含めて「pred数=エントリ数」を保証）。
   - 実装: finalize_phis を LoopForm-aware に拡張（MIR由来pred + 追加pred(latch) をマージ）。
   - 受け渡し: pred 終端直前に局所 cast（既存の `coerce_to_type` を流用）。
   - 受入: `NYASH_LOOPFORM_LATCH2HEADER=1` をONにしても verifier green（PHI欠落なし）を確認。
3) body→dispatch 導線の常用化（中期）
   - 単純ボディから開始（終端が1つ=back-edge のみ）。
   - その後に複数出口/ネスト break/continue を段階解放（tag/payload で正規化）。
4) 可視化と計測（並行）
   - ループごとに dispatch-only PHI を確認（PHI個数/ゼロ合成の削減）。
   - post-terminator 挿入検知のカバレッジをログ化。

Acceptance（段階ごと）
- A1: LoopForm ON でも従来挙動と等価（Break 集約のみ・非破壊、smoke green）。
- A2: BuilderCursor 厳格化で post-terminator が検知ゼロ（panic不発）が続く。
- A2.5: sealed=ON で dep_tree_min_string の dominator 違反ゼロ（IR dump 不要レベル）。
- A3: header PHI 正規化後、latch→header 有効でも verifier green（PHI 欠落なし）。
- A4: body→dispatch を単純ボディで常用化し、dispatch 以外に PHI が出ないことを確認。
- A5: `NYASH_LLVM_USE_HARNESS=1`（llvmlite）と OFF（Rust）の出力が dep_tree_min_string で機能一致。

Execution Plan — Next 48h
1) BuilderCursor 全域適用（externcall/newbox/arrays/maps/call）。
2) Sealed=ON で dep_tree_min_string をグリーン（PHI/ドミナンス違反ゼロ）。
3) （並行）llvmlite 検証ハーネス追加（Nyash ABI 経由、ゲートで切替）。
4) BODY→DISPATCH 常用化（単純ボディ）。

## 🎉 LLVMプラグイン戻り値表示問題修正進行中（2025-09-10）

### ✅ **完了した主要成果**：
1. **プラグイン実装** ✅ - `nyash.plugin.invoke_*`関数はnyrtライブラリに正常実装済み
2. **プラグイン呼び出し** ✅ - 環境変数なしでメソッド呼び出し成功
3. **戻り値型推論修正** ✅ - MIR builder にプラグインメソッド型推論追加
4. **by-id統一完了** ✅ - by-name方式削除、method_id方式に完全統一  
5. **環境変数削減達成** ✅ - `NYASH_LLVM_ALLOW_BY_NAME=1`削除完了
6. **シンプル実行実現** ✅ - `./target/release/nyash --backend llvm program.nyash`
7. **codex TLV修正マージ完了** ✅ - プラグイン戻り値TLVタグ2/6/7対応（2000行修正）
8. **console.log ハンドル対応** ✅ - 不正なi64→i8*変換を修正、ハンドル対応関数に変更
9. **型変換エラー解消** ✅ - bool(i1)→i64拡張処理追加でLLVM検証エラー修正

### 🔬 **現在の動作状況**（2025-09-11 最新）：
- **プラグイン実行** ✅ - CounterBox、FileBox正常実行（副作用OK）
- **型エラー解消** ✅ - LLVM関数検証エラー修正済み
- **コンパイル成功** ✅ - 環境変数なしでLLVMバックエンド動作
- **条件分岐動作** ✅ - `if f.exists("file")` → "TRUE"/"FALSE"表示OK
 - **LLVMコード生成の分割** ✅ - `codegen/` 下にモジュール化（types/instructions）
 - **BoxCall Lower 移譲** ✅ - 実行経路は `instructions::lower_boxcall` に一本化（旧実装は到達不能）
 - **スモーク整理** ✅ - `tools/llvm_smoke.sh` を環境変数整理。プラグイン戻り値スモーク追加

### 🔍 **根本原因判明**：
**Task先生の詳細技術調査により特定**：
- **不正なハンドル→ポインタ変換**: `build_int_to_ptr(iv, i8p, "arg_i2p")` でハンドル値（3）を直接メモリアドレス（0x3）として扱う不正変換
- **修正完了**: console.log を `nyash.console.log_handle(i64) -> i64` に変更、ハンドルレジストリ活用

### 🔍 **残存課題（新）**：

#### 1. **プラグイン戻り値表示問題** 🔶 **←現在調査中**
**症状（修正後も継続）**:
- `CounterBox.get()` → 数値戻り値がprint()で空白
- `FileBox.read(path)` → 文字列戻り値がprint()で空白  
- `FileBox.exists(path)` → if条件では動作、変数格納で失敗
- `local x = 42; print(x)` → 正常（問題はプラグイン戻り値のみ）

**残る問題の推定**:
- print()以外のExternCall処理にも同様の問題がある可能性
- MIR変数代入処理での型情報不整合
- プラグイン戻り値のハンドル→実値変換が不完全

#### 2. **LLVM BinOp 文字列連結まわり（新規スモークで再現）** 🔶
現象:
- 追加スモーク `ny-plugin-ret-llvm-smoke` で LLVM AOT オブジェクト生成中に `binop type mismatch` が発生
- ケース: `print("S=" + t)`（`t` は `StringBox.concat("CD")` の戻り値）

暫定分析:
- `+` 連結の BinOp Lower は (ptr+ptr / ptr+int / int+ptr) を NyRT シム経由で処理する設計
- `t` のLowerが i64 handleのままになっている/もしくは型注釈不足で ptr へ昇格できていない可能性
- `instructions::lower_boxcall` 内の `concat` 特例（f64 fast-path）は撤去済み。以降は tagged invoke → dst 型注釈に応じて i64→ptr へ変換する想定

対応方針:
1) MIR の `value_types` に BoxCall 戻り値（StringBox）の型注釈が入っているか確認（不足時は MIR 側で注入）
2) BinOp 連結経路は fallback として (ptr + 非ptr) / (非ptr + ptr) に対して handle想定の i64→ptr 昇格を再確認
3) 上記fixの後、スモーク `NYASH_LLVM_PLUGIN_RET_SMOKE=1` を通し、VM/LLVM 一致を確認

メモ:
- 現状でも BoxCall は新経路に完全委譲済み。旧実装は到達不能（削除準備OK）。

### 📊 **修正済みファイル**：
- **src/mir/builder.rs** - プラグインメソッド戻り値型推論追加
- **src/backend/llvm/compiler.rs** - by-name方式削除、method_id統一
- **src/backend/llvm/compiler/codegen/mod.rs** - 分割に伴う委譲（Const/Compare/Return/Jump/Branch/Extern/NewBox/Load/Store/BoxCall）
- **src/backend/llvm/compiler/codegen/types.rs** - 型変換/分類/型マップ（新）
- **src/backend/llvm/compiler/codegen/instructions.rs** - Lower群を実装（新）
- **tools/llvm_smoke.sh** - LLVM18 prefix整理・プラグイン戻り値スモーク追加
- **apps/tests/ny-plugin-ret-llvm-smoke/main.nyash** - プラグイン戻り値スモーク（新）
- **CLAUDE.md** - 環境変数セクション更新、コマンド簡素化
- **README.md / README.ja.md** - 環境変数説明削除
- **tools/llvm_smoke.sh** - テストスクリプト環境変数削除

### 🎯 **次期深堀り調査対象**：
1. **プラグインランタイム戻り値パス詳細調査** - `crates/nyrt/src/lib.rs`
2. **LLVM BoxCall戻り値処理詳細分析** - `src/backend/llvm/compiler/real.rs` 戻り値変換
3. **MIR変数代入メカニズム調査** - BoxCall→変数の代入処理  
4. **print()関数とプラグイン値の相性調査** - 型認識処理
5. **BinOp 連結の保険見直し** - 片方が i64 (handle) の場合に ptr に昇格する安全弾性
6. **BoxCall 旧実装の物理削除** - スモークグリーン後に `mod.rs` の死コードを除去

### ✅ **リファクタリング完了（2025-09-11）**：
**PR #136マージ済み** - LLVMコンパイラモジュール化：
- `compiler.rs` → 4モジュールに分割（mod.rs, real.rs, mock.rs, mock_impl.in.rs）
- プラグインシグネチャ読み込みを `plugin_sigs.rs` に移動
- BoxタイプID読み込みを `box_types.rs` に移動
- PR #134の型情報処理を完全保持

Hot Update — 2025‑09‑12 (Boxing: ctx + dev checks)
- Added `instructions/ctx.rs` introducing `LowerFnCtx` and `BlockCtx` (Resolver-first accessors: ensure_i64/ptr/f64). Env `NYASH_DEV_CHECKS=1` enables extra cursor-open asserts.
- Added `instructions/string_ops.rs` with lightweight newtypes `StrHandle(i64)` and `StrPtr(i8*)` (handle-across-blocks policy). Call sites will adopt gradually.
- Exposed new modules in `instructions/mod.rs` and added a thin `lower_boxcall_boxed(..)` shim.
- Wiring remains unchanged (still calls existing `lower_boxcall(..)`); borrow conflicts will be avoided by migrating call-sites in small steps.
- Scope alignment: Boxing covers the full BoxCall lowering suite (fields/invoke/marshal) as we migrate internals in-place.

Hot Update — 2025‑09‑12 (flow API trim)
- `emit_jump` から `vmap` 引数を削除（sealed前提で未使用のため）。
- `seal_block` から `vmap` 引数を削除（snapshot優先・ゼロ合成で代替）。
- 上記により `compile_module` 内の借用競合（&mut cursor と &vmap の競合）を縮小。

Dev Checks（実装）
- 追加: `NYASH_DEV_CHECK_DISPATCH_ONLY_PHI=1` で LoopForm 登録がある関数の PHI 分布をログ出力（暫定: dispatch-only 厳格検証は今後強化）。
- 既存: BuilderCursor の post-terminator ガードを全域適用済み（emit_instr/emit_term）。

結果（Step 4 検証）
- dep_tree_min_string の LLVM オブジェクト生成が成功（sealed=ON, plugins OFF）。
  - コマンド（例）:
    - `cargo build --features llvm --bin nyash`
    - `NYASH_DISABLE_PLUGINS=1 NYASH_LLVM_OBJ_OUT=tmp/dep_tree_min_string.o target/debug/nyash --backend llvm apps/selfhost/tools/dep_tree_min_string.nyash`
  - 出力: `tmp/dep_tree_min_string.o`（約 10KB）

次の一手（箱化の本適用・安全切替）
- lower_boxcall 内部の段階移行（fields → invoke → marshal）を小スコープで進め、呼び出し側の `&mut cursor` 借用境界と競合しない形で flip。
- flip 後、Deny-Direct（`rg "vmap\.get\("` = 0）を下流のCIメモに追記、必要なら `#[cfg(debug_assertions)]` の簡易ガードを追加。

Note（箱化の段階切替）
- BoxCall 呼び出しを `lower_boxcall_boxed` へ全面切替は、`compile_module` のループ内における `&mut cursor` の借用境界と干渉するため、いったん保留。内部の移行（fields/invoke/marshal）を先に進め、借用の生存域を短縮した上で切替予定。

## 🎯 Restart Notes — Ny Syntax Alignment (2025‑09‑07)

### 目的
- Self‑Hosting 方針（Nyのみで工具を回す前段）に合わせて、Ny 構文とコーディング規約を明示化し、最小版ツールの完走性を優先。

### Ny 構文（実装時の基準）
- 1行1文／セミコロン非使用。
- break / continue を使わない（関数早期 return、番兵条件、if 包みで置換）。
- else は直前の閉じ波括弧と同一行（`} else {`}）。
- 文字列の `"` と `\` は確実にエスケープ。
- 反復は `loop(条件) { …; インクリメント }` を基本とし、必要に応じて「関数早期 return」型で早期脱出。

### 短期タスク（Syntax 合意前提で最小ゴール）
1) include のみ依存木（Nyのみ・配列/マップ未使用）を完走化
   - `apps/selfhost/tools/dep_tree_min_string.nyash`
   - FileBox/PathBox + 文字走査のみで JSON 構築（配列/マップに頼らない）
   - `make dep-tree` で `tmp/deps.json` を出力
2) using/module 対応は次段（構文・優先順位をユーザーと再すり合わせ後）
   - 優先: `module > 相対 > using-path`、曖昧=エラー、STRICT ゲート（要相談）
3) ブリッジ Stage 1 は保留
   - `NYASH_DEPS_JSON=<path>` 読み込み（ログ出力のみ）を最小パッチで用意（MIR/JIT/AOT は不変）
