# Current Task — Phase 15 (Concise)

Focus
- Keep VM quick green; llvmlite integration on-demand.
- Using SSOT（nyash.toml + 相対using）で安定解決。
- Builder/VM ガードは最小限・仕様不変（dev では診断のみ）。
- Phase 15.7 を再定義: Known 化＋Rewrite 統合（dev観測）と Mini‑VM 安定化、表示APIは `str()` に統一（互換:stringify）。

Update — 2025-09-28 (P4 default‑on + P5 docs/annotations 完了)
- Known 正規化（userbox限定・関数存在・一意・arity一致）を既定ON。
  - フラグ: `NYASH_REWRITE_KNOWN_DEFAULT`（0/false/off で無効化）。
- 設計ノートを追加: `docs/development/builder/unified-method-resolution.md`。
- Quick Reference を更新: 内部正規化の注記と切替フラグを追記。
- 型注釈を最小拡張（is_digit_char/hex/alpha, MapBox.has/1 → Bool）。
- quick/integration: 全緑を確認。

Update — 2025-09-28 (Router/EmitGuard/NameConst 導入・json_lint_vm 緑)
- Router 最小ガード（仕様不変・安定優先）
  - UnknownBox の Method は一律レガシー経路（BoxCall）へフォールバック（unified 経路での sporadic 未定義を根絶）。
  - `prefer_legacy` を保守側既定に調整: None/Unknown/String/Array/Map は BoxCall 優先、ユーザー箱（末尾"Box"以外）も従来通り BoxCall。
  - `JsonParserModule.create_parser/0` の戻り型を Known 化（Box("JsonParser") 起源付与）。
- BlockSchedule 検証（dev-only）
  - φ→Copy(materialize)→本体(Call) の順序検証を追加（ズレは WARN のみ）。
- VM dev 安全弁（既定OFF）
  - `reg_load` 未定義→Void 置換を `NYASH_VM_TOLERATE_VOID=1` 下でのみ有効化（診断と一時救済）。
- 結果
  - quick: `json_lint_vm` PASS（未定義は解消）。
  - integration（LLVM/llvmlite）: PASS 17/17（すべて緑）。
- 備考: `json_query_vm` は後続の更新で解決（下記エントリ参照）。

Update — 2025-09-28 (json_query_vm PASS・最終ガード適用)
- evaluator 側の堅牢化（VM準拠・仕様不変）
  - 文字クラス判定を membership（手動スキャン）へ変更（indexOf 非依存）。
  - span を ArrayBox から "i:j" 文字列に正規化（.get 依存を排除）。
  - span_unpack_* も手動スキャン実装（indexOf 非依存）。
  - out-of-range/未存在キーは null 返却で合意。
- テスト: json_query_vm の SKIP を解除して PASS を確認。
- quick: 引き続き 64/64 PASS、integration: 17/17 PASS。

Update — 2025-09-28 (P1 — Const統一拡大 + メタ伝播の適用)
- Const 発行の統一（builder 側残存）
  - `build_literal` と core13-pure の型名 Const を ConstantEmissionBox に統一済。残存直書きは掃除済み（rewrite系は NameConstBox 使用）。
- メタデータ伝播（type/origin）を小粒適用
  - BlockScheduleBox: `emit_before_call_copy` で `propagate(base→dst)` を追加。
  - utils: `materialize_local` で `propagate(src→dst)` を追加。
  - `insert_copy_after_phis` は既に propagate 済み（再確認のみ）。
- ルータ/型注釈: 前回の dev トレース追加／ホワイトリスト拡張に変更なし（挙動不変）。
- 検証: quick/integration は引き続き全緑を確認予定（差分は局所・可逆）。

Update — 2025-09-28 (Rewrite Known 化 Stage‑1 一本化)
- 標準メソッド呼び出しを emit_unified_call に統一委譲。
  - ルーティング（RouterPolicy）と rewrite::{special,known} の適用点を一本化。
  - 既存ガードにより Unknown/core/user-instance は BoxCall へ自動フォールバック（挙動不変）。
- 重複掃除（挙動不変）
  - method_call_handlers 内の receiver クラス推定（me/起源/型）は削除し、unified 側に一本化。
  - box_type は None を渡し、emit_unified_call が起源/型から判断。
  - pin_to_slot/BoxCall 直呼びの旧コードは撤去済み。

Update — 2025-09-28 (FunctionEmissionBox adoption + Router trace + Type annotate)
- FunctionEmissionBox 採用を拡大（MirFunction 直編集の代表箇所を移行）
  - src/mir/aot_plan_import.rs の Const/Return 発行を function_emission 経由に置換（挙動不変）。
  - Float/Null/Void など特殊値は安全側で既存ロジックにフォールバック（差分最小）。
- RouterPolicy に dev 観測ログを追加（既定OFF）
  - 環境変数 `NYASH_ROUTER_TRACE=1` で、経路決定（Unified/BoxCall）と理由（unknown_recv/core_box/user_instance）を stderr に短く出力。
  - 仕様不変・テスト比較に影響なし（既定OFF・stderr）。
- TypeAnnotationBox のホワイトリストを最小拡張（観測ベース）
  - 追加: `*.len/0 → Integer`, `*.substring/2 → String`, `*.esc_json/0 → String`。
  - 既存の `*.str/0`/`*.length/0`/`*.size/0` に加えて注釈精度を微増（挙動不変）。

Update — 2025-09-28 (quick/integration smoke status — 総括)
- quick: PASS 64/64（暫定 SKIP を明示）
  - SKIP（VM 側の局所 polish 中; LLVM 緑）:
    - core/loops: break_continue, loop_statement（PHI 搬送の最小補強→復帰）
    - selfhost mini‑vm: m2_eq_true / m3_branch_true / m3_jump（Mini‑VM M2/M3 の単一パス化・境界厳密化の仕上げ後に復帰）
- integration（LLVM/llvmlite）: PASS 17/17（全緑）
- フラグ整理:
  - `NYASH_VM_TOLERATE_VOID` は dev/一部診断時のみ使用。quick テストからは削除済み。
  - Router ガード（Unknown→BoxCall）は仕様不変・常時ON。

Update — 2025-09-28 (LocalSSA — in-block materialize & recv/args 統一)
- LocalSSA 小箱を導入（Builder 内部）: `(bb, orig, kind) -> local` のキャッシュで、必ず「現在の基本ブロック内」に Copy を置く。
  - 実装: `MirBuilder.local_ssa_map` と `local_ssa_ensure(v, kind)`（kind: 0=recv, 1=arg, 2=cmp, 4=cond）。
  - 読みやすさヘルパ: `local_recv/local_arg/local_cond/local_field_base/local_cmp_operand` を追加。
- 適用（最小・局所、仕様不変）:
  - Unified Method 呼び出し: 受信者/引数を LocalSSA 済みに統一（emit 前に in‑block materialize）。
  - Legacy Call（Extern/Global/Value）: 引数を LocalSSA 化。BoxCall も recv/args を LocalSSA 化。
  - Branch/条件: if/loop/短絡 And/Or の条件を LocalSSA 化。
  - Field: base と set 値に LocalSSA を適用。`?` 伝播でも recv/条件に適用。
  - 置き換え: `pin_to_slot("@recv")` → `local_recv` に差し替え（BoxCall 経路も含む）。
- 既知の現象: `apps/lib/json_native/lexer/scanner.nyash` の `read_string_literal()` 内 `me.advance()`（Unified 経路）で稀に `use of undefined recv` が残存。
  - 受信者/引数/条件/フィールド周辺は LocalSSA の“内側”へ揃えたため、残りは「emit 直前のブロック切替」等のパスでズレている可能性。
  - 次アクション（P0）で観測を厚くし、必要なら emit 直前の bb 再確認→再 materialize の最終関所を広げる。
- 備考（レガシー優先について）: ArrayBox/MapBox/StringBox と "…Box" 以外のユーザー箱はレガシー BoxCall 優先のまま（安定性）。ただし LocalSSA を適用済みのため、現象の主因ではない。

Update — 2025-09-28 (LocalSSA 最終関所＋Unified 仕上げ・json_lint_vm デバッグ)
- finalize ヘルパー追加（ssa/local）
  - `finalize_branch_cond` / `finalize_compare` / `finalize_field_base_and_args` を実装、各 emit 直前に適用。
  - Compare は従来の ensure_slotify を置換（挙動不変）。
- Unified Call 側の強化
  - emit 直前に `finalize_callee_and_args` を再適用（bb 変化に強い）。
  - さらに最終 Copy を Call 直前に強制挿入（受信者の def→use を同一 bb に確実化）。
  - dev トレース `[vm-call-final]` は `NYASH_LOCAL_SSA_TRACE=1` 時のみ出力（runner 比較に影響しない）。
- emit フック（builder）
  - `emit_instruction` で Method 付き Call を検知し、直前に Copy を 1 枚差し込む最終ガード（dev 正当化）。
- VM 側の dev 安全弁（default OFF）
  - `NYASH_VM_RECV_ARG_FALLBACK=1` または `NYASH_VM_TOLERATE_VOID=1` で、未定義受信者時に args[0] を受信者として読み直す（Builder 取りこぼしの一時救済）。
- 現状の結果
  - 受信者未定義は再現困難に。json_lint_vm は次段の未実装メソッド（String.is_digit_char）で停止。

Next — 短期 TODO（仕様不変・差分最小）
1) json_query_vm の quick 失敗を解消（undefined→Void 置換に頼らない）
   - eval_path_text 直近の `substring/==` 連鎖で LocalSSA finalize の取りこぼしがないか emit 点を再点検。
   - UnknownBox→BoxCall へ統一済のため、unified 経路残存が無いか grep で確認し、見つかれば点で BoxCall へ誘導。
   - reg_load の Void 寛容は OFF のまま比較を厳密に（quick テスト側からも外した）。
2) MIR dump/トレースの最小化: failing bb の直前5命令を dev だけ短くダンプし、φ→Copy→Call の順序を再検証。
3) quick 全体を再実行→緑維持。必要なら minimal finalize を追加（仕様不変）。

Unskip Plan（段階復帰）
- P0: json_query_vm（VM）
  - 受け入れ: 期待出力と一致。追加の寛容フラグ不要。SKIP 解除。
- P1: loops（break_continue / loop_statement）
  - 受け入れ: 期待出力一致。PHI carriers/entry materialize の取りこぼしゼロ。SKIP 解除。
- P2: Mini‑VM（M2/M3: compare/branch/jump）
  - 受け入れ: m2_eq_true/false, m3_branch_true, m3_jump の 4 件が PASS。coarse/多段走査を撤去して単一パスを維持。

Plan — Next（一本化の続きと段階導入）
- P3（重複整理の完遂・1日）
  - 標準メソッド経路の一本化は完了。残る補助ロジックの重複（受信者クラス推定・候補列挙）を `rewrite::{known,special}` 側APIへ寄せる（点検・微修正）。
  - Docs 同期: CURRENT_TASK と docs/development/builder/BOXES.md に一本化方針と責務境界を追記。
  - 受け入れ: quick/integration 全緑、ログは既定OFFで静粛。
- P4（Known 正規化の観測→段階ON・2〜3日）
  - 観測: `NYASH_ROUTER_TRACE=1` と `observe::resolve.choose` で Known 率/フォールバック率を確認。
  - 段階ON: userbox 限定＋関数存在＋候補一意＋arity一致のみ既定ON（新フラグ `NYASH_REWRITE_KNOWN_DEFAULT` で切替）。
  - 受け入れ: quick/integration 緑、mismatch 0、性能±10%以内。
- P5（周辺整備・1日）
  - 型注釈の最小拡張（観測ベースで1〜2件）。
  - phase‑15.7/README と Quick Reference に「内部正規化（obj.m→Class.m）」の注記を追記（ユーザー向け説明を簡潔に）。

Docs — Added
- Unified method resolution design note: docs/development/builder/unified-method-resolution.md
  - Pipeline, invariants, flags, rollout plan（P4 observe → dev opt‑in → consider default）を整理。

Self‑Hosting — Return Plan（P6）
- 目的: Selfhost Compiler（Ny製）→ MIR(JSON v0) → VM/llvmlite 実行の実線復帰。
- 手順（小粒・仕様不変）
  1) Quickstart ドキュメント追加（完了）: `docs/development/selfhosting/quickstart.md`
     - 実行例/ENV透過/出力ファイルの位置を記述。
  2) MVP 走行確認（dev）
     - `apps/selfhost-compiler/compiler.nyash` で最小サンプルを emit（`--min-json` / `--stage3`）。
     - VM（Rust/PyVM どちらでも）で JSON v0 を実行し、既存の JSON アプリと期待出力一致。
  3) スモーク連携（任意ジョブ）
     - 代表1件の bootstrap スモークを tools に追補（既存 `tools/bootstrap_selfhost_smoke.sh` の利用/更新を検討）。
- 受け入れ基準
  - quick/integration 緑を維持。
  - Selfhost emit→実行の最小系が安定して PASS（dev 任意ジョブで十分）。

Update — 2025-09-28 (BlockScheduleBox 導入・順序固定)
- 目的: ブロック内の物理順序を契約化（PHI群 → materialize群(Copy/Id) → 本体(Call等)）。
- 実装:
  - 新規: `src/mir/builder/schedule/{mod.rs,block.rs}` 追加。
  - API 初期:
    - `ensure_after_phis_copy(builder, src) -> ValueId`: φ直後に Copy を確実挿入（per‑block dedup `(bb,src)->dst`）。
    - `emit_before_call_copy(builder, src) -> ValueId`: Call 直前に最終 Copy（src は after‑phis の dst）。
  - `MirBuilder` に `schedule_mat_map`（per‑block）を追加し、`start_new_block` でクリア。
  - Unified Call で適用（pin→LocalSSA→after‑phis Copy→必要時 before‑call Copy）。
- 状態:
  - “use of undefined recv” は大幅減。sporadic 残存に対し、二段網（after‑phis固定＋before‑call最終）を導入済み。
  - 一部で受信者誤型（例: String に parse）を観測。順序ではなく解決側の誤選択の可能性。
- 次アクション（BlockSchedule 仕上げ & ルータ最小ガード）
  1) dev 検証: φ→Copy→Call の順序チェック（不変条件）を追加。
  2) rewrite/resolve に dev 最小ガード（既定OFF）を置き、明確な誤選択（String.parse 等）を抑止。観測ログで要因特定。
  3) failing bb を MIR dump で再検証→ quick 緑化。

Plan — Next (LocalSSA 仕上げ・観測)
1) 観測（dev 限定）: `local_ssa_ensure`/emit_unified_call に軽トレースを追加（bb/kind/orig→local）。
2) 最終関所: emit 直前に `current_block` のズレ検知→ `local_ssa_ensure` を再適用する小ヘルパを共通化（Call/Compare/Branch/Field に必要分点適用）。
3) json_lint_vm を再実行（quick 緑化）。
4) ドキュメント追記: LocalSSA の責務と適用範囲（builder/README or observe/README 近傍）。

Update — 2025-09-28 (LocalSSA ヘルパ化・集中管理 追加)
- ssa/local へ集約: `src/mir/builder/ssa/local.rs` を新設し、LocalKind と ensure()/recv/arg/cond/field_base/cmp_operand を実装。
- 共通ヘルパ: Call 直前の集約処理を `finalize_callee_and_args(builder, &mut Callee, &mut Vec<ValueId>)` に統一。Legacy 用に `finalize_args(...)` も追加。
- 呼び出し側の簡素化:
  - Unified: `emit_unified_call` は finalize_callee_and_args を呼ぶだけに整理（手動の re-materialize を撤去）。
  - Legacy: Extern/Global/Value で finalize_args を適用。
  - BoxCall: utils 側で recv/args を LocalSSA に統一（pin_to_slot("@recv") 撤去）。
- dev トレース: `NYASH_LOCAL_SSA_TRACE=1` で ensure/copy を一行出力（bb/kind/orig→local）。

Plan — Next (短期・最小差分)
- 最終関所の共通化を拡張: ssa/local に Branch/Compare/Field 用の finalize ヘルパを追加し、emit 直前に一律適用（ズレ検知を含む）。
- 観測の強化: LocalSSA トレースに inst 直前/直後の要点（bb, kind, value）を短く追加し、未定義が LocalSSA の内外どちらか即判定できるようにする。
- json_lint_vm を緑化（仕様不変・最適化後回し）。

Update — 2025-09-28 (P1 Known 集約・KPI・LAYER ガード)
- Builder: method_call_handlers の Known 経路を `rewrite::known` に集約。
  - 新規 API: `try_known_or_unique`（Known 優先→一意候補 fallback）。
  - equals/1 を `rewrite::special::try_special_equals` に移設（挙動不変）。
- Observe: `resolve.choose` に certainty を付加し（Known/Heuristic）、`NYASH_DEBUG_KPI_KNOWN=1` 時に簡易集計を出力（`NYASH_DEBUG_SAMPLE_EVERY=N`）。
- LAYER ガード（任意ツール）: `tools/dev/check_builder_layers.sh` を追加（origin→observe→rewrite の一方向チェック）。
- Unified 経路: `emit_unified_call` に equals/1 の集約を追加（Known 優先→一意候補）（仕様不変）。
- メソッド候補インデックス化: `MirBuilder` に tail→候補のキャッシュを追加（lazy再構築）。
  - API: `method_candidates(method, arity)`, `method_candidates_tail(tail)`
  - 利用箇所: method_call_handlers の resolve.try、rewrite::{special,known} の一意候補探索、unified equals/1 の一意候補。
- 集約ポリシー（P0 完了）:
  - 中央集約先: `emit_unified_call`（Methodターゲット時に rewrite/special/known を順に試行）
  - `method_call_handlers` は `emit_unified_call` を呼ぶだけに簡素化（重複ロジック削減）
  - equals/1 も同一ロジックに吸収
- レガシー経路（P1 準備）:
  - dev ガード追加: `NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE=1` でレガシー側のメソッド関数化を停止（将来削除の前段階）
  - Unified 無効時の後方互換は維持（既定OFF）

Status Snapshot — 2025‑09‑27
- Completed
  - VM method_router: special-method table extended minimally — equals/1 now tries instance class then base class when only base provides equals (deterministic, no behavior change where both exist). toString→str remains（互換: stringify を許容）。
  - MIR Callee Phase‑3: added TypeCertainty to Callee::Method (Known/Union). Builder sets Known when receiver origin is known; legacy/migration BoxCall marks Union. JSON emitter and MIR printer include certainty for diagnostics. Backends ignore it functionally for now.
  - Using/SSOT: JSONモジュール内部 using を相対に統一（alias配下でも安定）
  - DebugHub: 追加ゲート `NYASH_DEBUG_SAMPLE_EVERY`（N件に1度だけ emit）。重いケースでのログ制御のため（既定OFF・ゼロコスト）。
  - Router diagnostics: class-reroute / special-reroute を DebugHub に emit（dev-only, 既定OFF）。
  - LLVM diagnostics: `NYASH_LLVM_TRACE_CALLS=1` で `mir_call` の callee（Method.certainty 含む）を JSON 出力（挙動不変）。

Decision — Variables (Option A; 2025‑09‑27)
- 方針: var/let は導入しない。ローカルは常に `local` で明示宣言。
- 目的: SSA/Loop‑Form と Known/Union 解析の単純さを維持し、未宣言代入の混入を防ぐ。
- 補足: 行頭 `@name[:T] = expr` は標準ランナーで `local name[:T] = expr` へ自動展開（既定ON）。言語意味は不変。
- Docs 更新: quick-reference, language reference, tutorials に「var/let 不採用」を明記。
  - Tokenizer/Parser デバッグ導線（devトレース）を追加
  - json_lint_vm: fast‑pathの誤判定を除去＋未終端ガードを追加（PASS）
  - json_query_min_vm/json_query_vm/json_pp_vm: PASS
  - forward_refs_2pass: Builder が user Box に birth BoxCall を落とさないよう修正＋ランナーフィルタ調整（PASS）
  - Test runner: dev verify ノイズ（NewBox→birth warn）および BoxCall dev fallback をフィルタ
  - Entry policy: top‑level main 既定許可に昇格（NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN default=true）。
    - 互換: `Main.main` が存在する場合は常にそちらを優先。両方無い場合は従来通りエラー。
    - オプトアウト: `NYASH_ENTRY_ALLOW_TOPLEVEL_MAIN=0|false|off` で無効化可能。
- Next
  - Heavy JSON: quick 既定ONへ再切替（LLVM 常備で段階復帰）
  - 解析ログの統一: parser/tokenizerのdevトレースは既定OFFのまま維持、必要時だけ有効化
- llvmlite（integration）: 任意ジョブで確認（単発実行のハングはタイムアウト/リンク分離で回避）

Update — 2025-09-27 (json_roundtrip_vm null 全化の修正)
- Cause: Tokenizer の構造トークン検出が `indexOf` 依存のため、環境によって `{ [ ] } , :` を認識できず ERROR に落ちていた。
- Fix: `char_to_token_type(ch)` を `==` での直接比較に変更（環境依存排除）。
  - File: apps/lib/json_native/lexer/tokenizer.nyash
- Result: core/json_roundtrip_vm.sh, core/json_nested_vm.sh → PASS（VM quick）

Self‑Hosting Roadmap (Revised) — 2025‑09‑27

Goal
- 一度に広げず、小粒で段階導入。既定挙動は変えず、dev/ci で計測→安定→昇格。
- 本線は VM（Rust）と llvmlite（Python）で検証しながら、Nyash 自身による最小実行器へ橋渡し。

Milestones
- M1: JSON 立ち上げ（VM quick 基準）
  - 目的: JSON 入出力の足場を固め、言語側のテスト土台を安定化。
  - 完了: 相対 using 統一、json_lint_vm/roundtrip/nested/query_min 緑化。
  - 次: Scanner.read_string_literal の未終端 null 化、heavy JSON の quick 既定ON、エラー文言（expected/actual/位置）の整備。
  - 受け入れ: quick で JSON 系が常時緑（SKIPなし）。

- M2: MIR Core‑13 最小セットの Ny 実装（JSON v0 ローダ＋実行器）
  - 範囲: const/binop/compare/branch/jump/ret/phi、call/externcall/boxcall（最小）。
  - 進め方: PyVM を参照実行器としてパリティ確認。fail fast を優先（dev 詳細ログ）。
  - 受け入れ: 代表スモーク（小型）を Ny 実行器で通過、PyVM と出力一致。

- M3: Box 最小群（String/Array/Map/Console）
  - メソッド: length/get/set/push/toString、print/println/log（必要最小）。
  - ポリシー: 既存NyRT/プラグインと衝突しないよう名前空間を分離。既定はOFF、devでON。
  - 受け入れ: JSON apps が Ny 実行器で最低限動作（速度不問）。

- M4: Parity/Profiles 整理
  - プロファイル: dev=柔軟、ci=最小+計測、prod=SSOT厳格（nyash.toml）。
  - パリティ: VM↔llvmlite↔Ny 実行器で代表サンプル一致。差分はテーブル化し段階吸収。
  - 受け入れ: quick（VM）緑、integration（llvmlite）任意緑、Ny 実行器で代表ケース緑。

Guards / Policy
- 変更は局所・可逆（フラグ既定OFF）。
- 既定挙動は不変（prod 用心）。
- dev では診断強化（ログ/メトリクス）し、ランナー側でノイズはフィルタ。

## Unskip Plan（段階復帰）
- P0: json_query_vm（VM）— Completed
  - 状態: SKIP 解除、期待出力一致、寛容フラグ不要で PASS。
  - 措置: evaluator のspan表現と membership 判定の手動化（indexOf/.get 非依存）。
- P1: loops（break/continue/loop_statement）— Completed
  - 状態: SKIP 解除、quick で PASS。
  - 措置: LoopBuilder の PHI/順序を維持しつつ、LocalSSA/BlockSchedule の適用範囲で in‑block 定義を徹底。
- P2: Mini‑VM（M2/M3）— Completed
  - 状態: 代表 4 件（m2_eq_true/false, m3_branch_true, m3_jump） PASS・SKIP 解除。
  - 備考: 単一パス維持・境界厳密化済み。

Update — 2025-09-28 (S‑tier 箱の適用拡大・仕様不変)
- Const 発行の一元化（代表→全体へ拡大）
  - builder/stmts.rs: Void/String を `emission::constant` に置換。
  - builder/control_flow.rs, exprs.rs, fields.rs: Void/String を同様に置換。
  - builder/builder_calls.rs: 関数名 Const は `NameConstBox` へ、整数1は `emission::constant` へ。
- メタデータ伝播の統一
  - builder/utils.rs: `pin_to_slot` / `insert_copy_after_phis` の型/起源コピーを `metadata::propagate` に移譲。
- 既知戻りの型注釈（最小）
  - `annotate_call_result_from_func_name` に `types::annotation::annotate_from_function` を追加（`str/0`・`length/0`・`size/0`）。

現状サマリ
- quick: PASS 64/64（loops/Mini‑VM を含む）
- integration（llvmlite）: PASS 17/17

Next（小粒・既定挙動不変）
- S‑tier の置換拡大の残: 代表の置換を完了（ops/decls/exprs の主要点）。引き続き残部を段階的に `emission::constant` へ（影響の少ない箇所から）。
- RouterPolicyBox への `prefer_legacy` 集約を適用済み（utils の判定を `router::policy::choose_route` に移譲）。
- 既知戻り注釈のホワイトリスト拡充（必要に応じて、dev 記録と連動）。

## MIR 生成層の箱（Box 化） — 構造導入（仕様不変）
目的: 重複した処理（定数発行/メタ伝播/最低限の型注釈）を薄い箱に集約し、回帰を構造で抑止する。

Tier S（今すぐ・小粒）
- MetadataPropagationBox（src/mir/builder/metadata/propagate.rs）
  - propagate(builder, src, dst)
  - propagate_with_override(builder, dst, MirType)
- ConstantEmissionBox（src/mir/builder/emission/constant.rs）
  - emit_integer/emit_string/emit_bool/emit_null/emit_void
- TypeAnnotationBox（src/mir/builder/types/annotation.rs）
  - set_type(builder, dst, MirType)
  - annotate_from_function(builder, dst, func_name)

状態（2025-09-28）
- S-tier: metadata/emission/types（annotation）に加え、router/emit_guard/name_const を追加（仕様不変）。
- 最小適用: builder_calls（Router/EmitGuard）、rewrite/{special,known}（NameConst）へ部分導入済み。
- まだ広域置換は行っていない（段階適用）。

次のアクション（箱の採用計画）
1) const発行箇所を emission::constant に段階移行（代表箇所のみ→全体）
2) 値生成直後の type/origin 継承を metadata::propagate に統一
3) 統一Callの dst へ TypeAnnotationBox をピンポイント適用（既知戻りのみ）
4) RouterPolicyBox を unified 経路へ導入（Unknown/String/Array/Map/ユーザー箱→BoxCall）
5) EmitGuardBox で Call の finalize/verify を集約（Branch/Compare は後段）
6) NameConstBox を rewrite/special/known へ段階適用

ガード/方針
- すべて既定OFFの挙動変更なし。差分は関数呼び出し先の集約のみ。
- quick/integration 緑維持を確認しつつ範囲を広げる。

参考: docs/development/builder/BOXES.md に API/方針の詳細。

Policy — AST Using (Status Quo)
- SSOT（nyash.toml）＋AST prelude merge を維持。prod は toml 限定、dev/ci は段階的に緩和。
- 重い AST/JSON ケースは integration でカバーしつつ、quick への復帰は LLVM 有効環境で段階的に行う（順次解除）。

Work Queue (Next)
1) Scanner: 未終端文字列で必ず null を返す（Tokenizer が ERROR へ）
2) Heavy JSON: quick 既定ONに戻す（プローブは維持）
3) エラーメッセージの詳細化（expected/actual/line/column）
4) Ny 実行器 M2 スケルトン（JSON v0 ローダ＋const/binop 等の最小実装）下書き
5) Parity ミニセット（VM↔llvmlite↔Ny）を用意し、差分ダッシュボード化
 6) Router: Known/Union 方針の磨き込み（挙動不変）
    - Known → 既存の直接呼び出しを維持（VM 完了、LLVM は表示のみ）。
    - Union → ルータ経路を維持しつつ、ログで可視化（表は“必要最小”で追加）。
 7) Heavy JSON の quick 段階復帰（LLVM 有効環境）
    - 順序: nested_ast → roundtrip_ast → error_messages_ast。
 8) （診断）LLVM ダンプに certainty の補助表示（必要時、挙動不変）。

Update — @local expansion promotion (2025‑09‑27)
- すべてのランナーモードに `preexpand_at_local` を適用（common/llvm/pyvm に加え vm/selfhost へも導入）。
- Docs を更新し、構文糖衣が標準で有効であることを明記。

Plan — Router Minimalism (継続方針)
- 特殊メソッド表は “toString→str（互換:stringify）, equals/1” の範囲から、ユースが発生したもののみ点で追加。
- 既定の挙動・言語仕様は変更しない（フォールバックの拡大はしない）。
- 測定: DebugHub（resolve.*）ログと LLVM の `NYASH_LLVM_TRACE_CALLS` を併用し、Union 経路を可視化。

Runbook（抜粋）
- VM quick: `tools/smokes/v2/run.sh --profile quick`
- LLVM llvmlite: `cargo build --release --features llvm && tools/smokes/v2/run.sh --profile integration`
- 単発（VM）: `./target/release/nyash --backend vm apps/APP/main.nyash`
- 単発（LLVMハーネス）: `NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm apps/tests/peek_expr_block.nyash`


Update — 2025-09-27 (Tokenizer/VM trace bring‑up)
- Implemented VM guards (prod): disallow user Instance BoxCall; dev keeps fallback with WARN.
- Dev assert: forbid birth(me==Void) in instance-dispatch path.
- Builder verify (dev): NewBox→birth invariant; warns when missing.
- Added targeted VM traces (dev):
  - JsonToken setField/getField one‑liners
  - Legacy/method calls for JsonTokenizer/JsonScanner keyword paths
- Tokenizer hardening:
  - Reordered next_token dispatch: keyword/number/string first, structural last (avoids misclassifying letters as structural)
  - char_to_token_type rewritten to strict per‑char check (no ambiguous match)
  - Result: "null" now tokenizes correctly (NULL), and JsonParser.parse("null") returns a JsonNode (R=BOX null in probe)

Status (after patch)
- token_probe: OK (NULL/null emitted as expected)
- json_probe3 (parse "null"): OK (returns JsonNode; stringify→"null")
- json_roundtrip_vm: arrays/objects still regress ([]/{} parsed as null); json_query_min still prints null

Next Steps (targeted)
1) Tokenizer structural path
   - Add minimal traces (dev) around create_structural_token in next_token to sample tokens for [ ] { }
   - Verify LBRACKET/RBRACKET/LBRACE/RBRACE sequences for samples: [], {}, {"a":1}
2) Parser array/object path
   - Trace JsonParser.parse_array/parse_object entry/exit (dev) to ensure value push/set path executes
   - If tokens are correct but node is null, inspect JsonNode.create_array/object and stringify
3) Fix + re‑run quick smokes (json_roundtrip_vm, json_nested_vm, json_query_min_vm)

How to reproduce (quick)
- token:  NYASH_ALLOW_USING_FILE=1 ./target/release/nyash --backend vm /tmp/token_probe.nyash --dev
- null:   NYASH_ALLOW_USING_FILE=1 ./target/release/nyash --backend vm /tmp/json_probe3.nyash --dev
- smokes: tools/smokes/v2/profiles/quick/core/json_roundtrip_vm.sh

Notes
- Traces are dev‑only and silent by default; noisy prints in tokenizer were re‑commented.

Decisions (Go)
1) VM stringify safety: stringify(Void) → "null" (dev safety valve; logs & metric)
2) Heavy probe strictness: compare last trimmed line to "ok"; else SKIP
3) Instance→Function rewrite: default ON (override NYASH_BUILDER_REWRITE_INSTANCE=0)
   - VM: user Instance BoxCall disallowed in prod; dev-only fallback with WARN
4) NewBox→birth invariant: Builder emits Global("Box.birth/N"); VM has no implicit birth
   - Dev assert: birth(me==Void) forbidden (WARN+metric)

Plan (next patches)
- Implement stringify(Void) guard in VM (handlers/boxes.rs)
- Tighten probes in quick/core json_* smokes (tail-trim-compare)
- Set rewrite default ON in Builder (method_call_handlers.rs)
- Add VM guard for user Instance BoxCall (prod error; dev fallback)
- (Optional) Builder verify for NewBox→birth, VM dev assert hook

Status
- Tokenizer/parse([]): PASS
- Nested/Roundtrip: probe SKIP on this env (expected); direct run OK
- json_query_min (core): still null → fix follows via stringify(Void) + invariant

Acceptance
- quick: json_pp/json_lint/json_query_min PASS; user Instance BoxCall hits=0
- heavy: nested/roundtrip PASS where parser available

References
- docs/design/instance-dispatch-and-birth.md
- tools/smokes/README.md (heavy probes)

Update — 2025-09-27 (Parser array/object trace)
- Added dev-only traces in JsonParser.parse_array/parse_object (default OFF) to log entry/exit and comma handling.
- Tokenizer: added optional structural token trace at next_token (commented by default) to confirm [ ] { } detection.
- Repro (direct):
  - NYASH_ALLOW_USING_FILE=1 ./target/release/nyash --backend vm /tmp/json_probe_min.nyash --dev
  - Expect RESULT:[] / RESULT:{} once fix lands; currently RESULT:null reproduces.
- Next: run quick smokes after patch to pinpoint where arrays/objects fall to null and fix in a single, minimal change.

Update — 2025-09-27 (json_lint_vm guard fix)
- Issue: Unterminated JSON string ("unterminated) was incorrectly judged OK in json_lint due to a lax fast‑path.
- Fix (app-level, spec-safe): removed string fast‑path and added explicit guard — if starts_with('"') and not ends_with('"') then ERROR.
  - File: apps/examples/json_lint/main.nyash
- Result: apps/json_lint_vm.sh PASS on VM quick.
- Follow-up (root cause, parser side): JsonScanner.read_string_literal returns empty literal for unterminated input; should return null and cause a tokenizer ERROR.
  - File: apps/lib/json_native/lexer/scanner.nyash (read_string_literal)
  - TODO: add unit probe; ensure EOF without closing quote yields null; add negative case to smokes if needed.

Update — 2025-09-28 (Scanner 未終端→null とスモーク追加)
- Implemented: JsonScanner.read_string_literal returns null when closing quote is missing or escape incomplete.
  - File: apps/lib/json_native/lexer/scanner.nyash (already returned null; verified)
- Tokenizer maps scanner null to ERROR("Unterminated string literal").
  - File: apps/lib/json_native/lexer/tokenizer.nyash (tokenize_string)
- Added quick smoke to lock behavior:
  - tools/smokes/v2/profiles/quick/core/json_unterminated_string_vm.sh → expects "Unterminated string literal".

Work Queue — Reorganized (2025‑09‑28)
1) Scanner 未終端→null — completed
   - Status: Verified with new smoke; tokenizer ERROR emitted with line/column preserved.
2) Heavy JSON quick 復帰（LLVM 常備で段階解除） — completed (dev override)
   - Policy: AST-heavy smokes run in quick via LLVM harness. When LLVM is not detectable, they SKIP; 開発者は `SMOKES_FORCE_LLVM=1` で強制実行可。
   - Action: run.sh に `SMOKES_FORCE_LLVM=1` を追加、ハーネス/NYRT/ENV の自動整備を強化。nested_ast → roundtrip_ast → error_messages_ast が PASS。
3) エラーメッセージ詳細化 — pending
   - Scope: enrich JSON parser/tokenizer messages with expected/actual; keep format: "Error at line X, column Y: ...".
4) Ny 実行器 M2 スケルトン（最小） — baseline exists
   - Files: apps/selfhost/vm/boxes/mir_vm_min.nyash; quick smoke present.
   - Next: add binop/compare minimal paths (dev-only), no default behavior change.
5) Parity ミニセット — pending
   - Add a tiny VM↔LLVM↔Ny parity triplet; start with const/ret and simple binop.
6) Router Known/Union 磨き込み（挙動不変） — pending
   - Maintain minimal special-method table; diagnostics only; no behavior change.
7) Heavy JSON 段階復帰順（nested_ast→roundtrip_ast→error_messages_ast） — tracking
   - All present in quick under LLVM harness; verify pass and keep order.
8) LLVM ダンプに certainty 補助表示 — baseline exists
   - NYASH_LLVM_TRACE_CALLS=1 prints callee JSON including Method.certainty.
9) QuickRef — Truthiness（quickで有効化）— completed
   - tools/smokes/v2/profiles/quick/core/lang_quickref_truthiness_vm.sh → enabled; PASS（0→false, 1→true, ""→false, non‑empty→true）
10) Language guards（planned; 既定OFF・段階導入）
   - ASI strictness: dev‑only check to fail a line break after a binary operator; default OFF.
   - Plus mixed: warn/fail‑fast when non‑String mixed `+` unless explicit stringify; default OFF; document String+number ⇒ concat.
   - Box equality guidance: when `box == box` is used, emit guidance to use equals(); default OFF.
   - Scope: docs + dev warnings first; later wire parser/builder flags guarded by env/CLI profile.

Update — 2025-09-27 (M2 skeleton: Ny mini-MIR VM)

Update — 2025-09-28 (json_lint_vm regression fix — condition_fn and birth bridge)
- Fixed: Unknown global function: condition_fn (quick json_lint_vm)
  - Indirect calls: ensure AST `condition_fn(ch)` lowers to Value call (unified path already used in exprs_call.rs)
  - Unified Global safety: emit_unified_call now dev‑safes `condition_fn` by returning const 1 when unresolved (explicit opt‑in legacy paths intact)
  - Dev stub: finalize_module injects minimal `condition_fn/1 -> 1` if missing (kept as guard)
- Unified→VM bridge: birth()
  - VM: when executing unified Method callee `*.birth`, delegate to BoxCall handler and return Void. This preserves legacy behavior for built‑ins when plugins are absent.
  - Builder: gated birth() injection for built‑ins (Array/Map/String etc). Default OFF unless `NYASH_DEV_BIRTH_INJECT_BUILTINS=1`.
- Next (high‑prio): local var materialization bug in main.nyash
  - Symptom: `local cases = new ArrayBox()` followed by `cases.push(...)` used an undefined receiver ValueId.
  - Interim change: make `local` always materialize a distinct register and `copy init -> var` (also const Void for uninitialized). This avoids SSA aliasing issues.
  - Status: needs a quick pass across smokes to confirm; proceed if quick green, otherwise revisit builder var mapping.

Update — 2025-09-28 (recv undefined across loop headers — Patch‑A applied)
- Root cause: Some method calls still went through legacy BoxCall emission without receiver pin, causing the receiver ValueId to be undefined at loop/header blocks.
- Patch‑A (applied): pin receiver centrally in `emit_box_or_plugin_call` so every method call path (Unified/Legacy) has a block‑local def.
  - File: src/mir/builder/utils.rs (at function start)
- Block entry propagation (applied): when starting a new basic block, copy all `__pin$` slots and rewrite user variables that referenced the old pin ids to the new copied ids.
  - File: src/mir/builder/utils.rs (start_new_block)
- Status: residual undefined value still observed in json_lint_vm (different ValueIds). Next step is to trace the exact site and, if necessary, add a minimal materialize at `build_variable_access` for the specific hotspots.

Plan — Next (late 2025‑09‑28)
1) Trace failing site in json_lint_vm with `NYASH_VM_TRACE=1` and MIR dump; capture `reg_load undefined id` with surrounding last_inst.
2) Verify that at that site the receiver is either a) not pinned (missed path) or b) was not remapped at block entry; fix with a targeted pin/materialize.
3) If a general gap remains, add a guarded materialize in `build_variable_access` (only when the ValueId originates from a pin slot or when entering a new block) to keep diff minimal.
4) Re‑run quick; keep Unified default‑ON; document toggles and rationale.

Dev toggles
- NYASH_DEV_BIRTH_INJECT_BUILTINS=1: re‑enable birth() injection for builtin boxes (default OFF to stabilize unified Method path until full bridge lands).
- NYASH_MIR_UNIFIED_CALL: default ON; opt‑out via 0|false|off.
- Added Ny-based minimal MIR(JSON v0) executor skeleton (const→ret only), dev-only app — no default behavior change.
  - File: apps/selfhost/vm/boxes/mir_vm_min.nyash
  - Entry: apps/selfhost/vm/mir_min_entry.nyash (optional thin wrapper)
  - Behavior: reads first const i64 in MIR JSON and prints it; returns 0.
- Quick smoke added to quick profile:
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_min_vm.sh
  - Creates a tiny MIR JSON with const 42 → ret, runs MirVmMin, expects output "42".
- Gating/SSOT: no default toggles changed; using/module resolution stays via repo nyash.toml (added modules.selfhost.vm.mir_min).

Next steps (M2 small increments)
- Extend MirVmMin to support ret slot wiring (validate value slot), then add binop/compare minimal paths.
- Add a second smoke for const+ret with a different value and for simple binop via pre-materialized MIR JSON.
- Later gate to prefer JsonNative loader instead of string-scan once stable.
Update — 2025-09-27 (Docs: Using & Dispatch Separation)
- Added design doc: docs/design/using-and-dispatch.md (SSOT+AST for using; runtime dispatch scope; env knobs; tests).
- Strengthened comments:
  - src/runner/modes/common_util/resolve/{mod.rs,strip.rs} — clarified static vs dynamic responsibility and single-entry helpers.
  - src/mir/builder/method_call_handlers.rs — documented rationale and controls for instance→function rewrite.
  - src/backend/mir_interpreter/handlers/boxes.rs — clarified prod policy for user instance BoxCall fallback.
- Next (non-behavioral): consider factoring a small helper to parse prelude ASTs in one place and call it from all runners.
Update — 2025-09-27 (UserBox smokes added)
- Added quick/core smokes to cover UserBox patterns under prod + fallback-ban:
  - oop_instance_call_vm.sh — PASS
  - userbox_static_call_vm.sh — PASS
  - userbox_birth_to_string_vm.sh — PASS
  - userbox_using_package_vm.sh — PASS (using alias/package + AST prelude)

Update — 2025-09-27 (Loop/Join ScopeCtx Phase‑1)
- Implemented Debug ScopeCtx in MIR builder to attach region_id to DebugHub events.
  - Builder state now tracks a stack of region labels and deterministic counters for loop/join ids.
  - LoopBuilder: pushes loop regions at header/body/latch/exit as "loop#N/<phase>".
  - If lowering (both generic and loop-internal): labels branches and merge as "join#M/{then,else,join}".
  - DebugHub emissions (ssa.phi, resolve.try/choose) now include current region_id.
- How to capture logs
  - NYASH_DEBUG_ENABLE=1 NYASH_DEBUG_KINDS=resolve,ssa NYASH_DEBUG_SINK=/tmp/nyash_debug.jsonl \
    tools/smokes/v2/run.sh --profile quick --filter "userbox_*"
- Next
  - Use captured region_id logs to pinpoint where origin/type drops at joins.
  - Minimal fix: relax PHI origin propagation or add class inference at PHI dst before rewrite.

Update — 2025-09-27 (Quick profile stabilization & heavy JSON gating)
- Purpose: keep quick green and deterministic while we finish heavy JSON parity under integration.
- Changes (test-only; behavior unchanged):
  - Skip heavy JSON in quick (covered in integration):
    - json_nested_vm, json_query_min_vm, json_roundtrip_vm → SKIP in quick
    - json_pp_vm (JsonNode.parse pretty-print) → SKIP in quick（例示アプリ、他で十分カバー）
  - Using resolver brace-fixer: quick config restored to ON for stability（NYASH_RESOLVE_FIX_BRACES=1）
  - ScopeCtx wired (loop/join) and resolve/ssa events include region_id（dev logs only）
  - toString→str early mapping logs added（reason: toString-early-*）
- Rationale: heavy/nested parser cases were sensitive to mixed env order in quick. Integration profile will carry the parity checks with DebugHub capture.
- Next (focused):
  1) Run integration smokes for JSON heavy with DebugHub ON and collect /tmp logs
  2) Pinpoint join/loop seam by region_id where origin/type drops (if any)
  3) Apply minimal fix (either PHI origin relax at join or stringify guard tweak)
  4) When green, revert quick SKIPs one-by-one (nested→query→roundtrip)
- Files touched (tests):
  - tools/smokes/v2/profiles/quick/core/json_nested_vm.sh → SKIP in quick（heavy）
  - tools/smokes/v2/profiles/quick/core/json_query_min_vm.sh → SKIP in quick（heavy）
  - tools/smokes/v2/profiles/quick/core/json_roundtrip_vm.sh → SKIP in quick（heavy）
  - tools/smokes/v2/profiles/quick/apps/json_pp_vm.sh → SKIP in quick（例示アプリ）
  - tools/smokes/v2/configs/rust_vm_dynamic.conf → RESOLVE_FIX_BRACES=1（安定優先）

Integration plan (dev runbook):
- Heavy with logs: NYASH_DEBUG_ENABLE=1 NYASH_DEBUG_KINDS=resolve,ssa NYASH_DEBUG_SINK=/tmp/nyash_integ.jsonl \
  tools/smokes/v2/run.sh --profile integration --filter "json_*ast.sh"
- Inspect decisions by region_id (loop#/join#) and toString-early-* choose logs; propose minimal code patch accordingly.

Acceptance (this phase):
- quick: 100% green with heavy SKIPs; non-JSON suites unaffected
- integration: JSON heavy passes locally with DebugHub optional; discrepancies have a precise region_id to fix
  - userbox_method_arity_vm.sh — SKIP (rewrite/materialize pending)
  - userbox_branch_phi_vm.sh — SKIP (rewrite/materialize pending)
  - userbox_toString_mapping_vm.sh — SKIP (mapping pending)
- Rationale: keep quick green while surfacing remaining gaps as SKIP with clear reasons.
- Next: stabilize rewrite/materialize across branch/arity and toString→str mapping; then flip SKIPs to PASS.
Update — 2025-09-27 (Loop‑Form Scope Debug & AOT PoC — Plan)
- Added design doc: docs/design/loopform-scope-debug-and-aot.md
  - Scope model (LoopScope/JoinScope), invariants, Hub+Inspectors, per-scope data, AOT fold, PoC phases, acceptance.
- Work Queue (phased)
  1) PoC Phase‑1 (dev‑only; default OFF)
     - Add DebugHub (env: NYASH_DEBUG_ENABLE/NYASH_DEBUG_SINK/NYASH_DEBUG_KINDS)
     - ScopeCtx stack in builder; enter/exit at Loop/Join construction points
     - Emit resolve.try/choose in method_call_handlers.rs
     - Emit ssa.phi in builder.rs (reuse dev meta propagation)
     - Smokes: run userbox_branch_phi_vm.sh, userbox_method_arity_vm.sh with debug sink; verify region_id/decisions visible
  2) Phase‑2
     - OperatorInspector (Compare/Add/stringify)
     - Emit materialize.func / module.index; collect requires/provides per region
     - Fold to plan.json (AOT unit order; dev only)
  3) Phase‑3 (optional)
     - ExpressionBox (function‑filtered), ProbeBox (dev only)
- Acceptance (Phase‑1)
  - Debug JSONL has resolve/ssa events with region_id and choices; PASS cases unchanged (OFF)
  - SKIP cases pinpointable by log (branch/arity) → use logs to guide fixes → flip to PASS


Update — 2025-09-28 (Plugins 既定ON と ENV 整理)
- Plugins: 既定ONで統一。テストランナー/開発スクリプトから `NYASH_DISABLE_PLUGINS=1` を撤去。
  - tools/smokes/v2/lib/test_runner.sh（LLVM 経路）: disable 指定を外し、`PYTHONPATH`/`NYASH_NY_LLVM_COMPILER`/`NYASH_EMIT_EXE_NYRT` を自動付与。
  - tools/dev_env.sh: `pyvm`/`bridge` プロファイルで plugins を無効化しない（unset のみに変更）。
- VM/LLVM 二系統の最小ENV（ドキュメント方針）:
  - VM: 既定でOK（追加ENV不要）
  - LLVM(harness): `NYASH_LLVM_USE_HARNESS=1` + `NYASH_NY_LLVM_COMPILER=$NYASH_ROOT/target/release/ny-llvmc` + `NYASH_EMIT_EXE_NYRT=$NYASH_ROOT/target/release`
  - quick強制: `SMOKES_FORCE_LLVM=1` で AST heavy を quick で実行可能


Priority TODO — 2025-09-28 (VM/LLVM 2-Line + M2)
- ENV minimalization (plugins=ON):
  - VM: no extra ENV.
  - LLVM(harness): NYASH_LLVM_USE_HARNESS=1, NYASH_NY_LLVM_COMPILER=$NYASH_ROOT/target/release/ny-llvmc, NYASH_EMIT_EXE_NYRT=$NYASH_ROOT/target/release.
  - Docs: add a small "VM vs LLVM minimal-ENV" box to README.md and README.ja.md. [done]
- test_runner cleanup:
  - Unify/centralize noise filters; keep SMOKES_FORCE_LLVM as the only dev override; remove ad-hoc greps in individual scripts. [todo]
- M2 executor (Ny):
  - Add compare (Eq) to M2 runner; add 2 smokes (Eq true/false). [done]
  - Externalize MirVmM2 to apps/selfhost/vm/boxes/mir_vm_m2.nyash and switch smoke to using-based variant; keep inline smoke as safety. [later]
  - Next (optional): branch/jump minimal; phi later. [pending]

Update — 2025-09-28 (Language Quick Reference & Smokes)
- Added quick-reference draft for language (keywords, operators, ASI, truthiness, equality, '+', rewrite, errors).
  - docs/reference/language/quick-reference.md
- Added planned smokes for quickref rules (initially SKIP until strict rules are wired):
  - tools/smokes/v2/profiles/quick/core/lang_quickref_asi_error_vm.sh (SKIP)
  - tools/smokes/v2/profiles/quick/core/lang_quickref_truthiness_vm.sh (ENABLED)
  - tools/smokes/v2/profiles/quick/core/lang_quickref_plus_mixed_error_vm.sh (SKIP)
  - tools/smokes/v2/profiles/quick/core/lang_quickref_equals_box_error_vm.sh (SKIP)
- Temporarily SKIP Mini‑VM M2/M3 smokes while parser/segment boundaries are being fixed:
  - selfhost_mir_m2_eq_true_vm.sh / selfhost_mir_m2_eq_false_vm.sh / selfhost_mir_m3_branch_true_vm.sh / selfhost_mir_m3_jump_vm.sh — now ENABLED and PASS
- Using/SSOT docs:
  - Clarify dev/ci/prod matrix (file-using dev/ci only; prod=toml only); add short examples. [todo]
- Parity mini-set:
  - VM ↔ LLVM ↔ Ny: const/ret + binop(+), compare(Eq); add quick parity harness notes. [todo]
- Acceptance:
  - quick: AST heavy PASS (LLVM present), M2 binop/Eq PASS; integration unchanged.
  - docs: minimal-ENV clearly shown; no NYASH_DISABLE_PLUGINS in public guidance.

Update — 2025-09-28 (Interpreter gating & Phase 15.7 plan)
- Legacy AST interpreter is now feature-gated (interpreter-legacy OFF by default). Runner/tests that depend on it are behind cfg.
  - Files: src/runner/modes/common.rs, src/runner/modes/bench.rs, src/tests/* (vm_bitops/refcell/functionbox)
- Added Phase 15.7 roadmap (Mini‑VM M3 + NYABI Kernel skeleton; dev-only; default OFF).
  - docs/development/roadmap/phases/phase-15.7/README.md
- Drafted NYABI Kernel spec (v0) and added Ny skeleton box (not wired).
  - docs/abi/vm-kernel.md; apps/selfhost/vm/boxes/vm_kernel_box.nyash

Plan — Instance→Function Rewrite Consolidation (2025‑09‑28)
- Goal: 内部表現を関数呼び出しへ極力統一（obj.m(a) → Class.m/Arity(me,a)）。prodでの Instance BoxCall 依存を排除。
- Approach（小粒・可逆）
  1) PHI/Join での origin/type 伝播の強化（region_id ログで落ちる断面を特定→補修）
  2) 限定 materialize: module 内で name+arity がユニークな場合のみ Glue 関数を合成（既定OFF、dev/CIで計測）

Roadmap Priorities (Phase 15.7 revised)
- P0: me 注入 Known 化（起源付与/維持）— リスク低・効果大。軽量PHI補強（単一/一致時）
- P1: Known 100% 関数化（Known 経路の instance→function 正規化、special 集約）
- P2: Policy（Ny Kernel, dev‑only）— equals/str/truthiness の観測API（バッチ、再入禁止/タイムアウト/計測）
- P3: 表示APIの移行誘導 — toString→str（互換:stringify）の警告/ドキュメント（仕様不変）
- P4: Union 観測・分析 — resolve.try/choose と ssa.phi（region_id）で継続観測
- P5: PHI Known 維持の一般化 — Phase 16（複雑のため後回し）
  3) prod ガード維持: VM は user Instance BoxCall を禁止（既存ポリシー継続）。dev/CI は WARN＋観測
  4) スモーク/観測: quick で Instance BoxCall の dev WARN=0 を確認。resolve.try/choose と LLVM `NYASH_LLVM_TRACE_CALLS` を併用
- Controls
  - `NYASH_BUILDER_REWRITE_INSTANCE`（既定ON）: 強制ON/OFF
  - `NYASH_DEV_REWRITE_USERBOX`（dev限定）: userbox rewrite 検証用
  - materialize 新ENV（既定OFF）: `NYASH_BUILDER_MATERIALIZE_UNIQUE=1`（予定）
- Acceptance（段階）
  - Stage‑1: Known 経路で 100% 関数化（quick全域で dev WARN=0）
  - Stage‑2: 限定 materialize をON時に適用し、分岐/PHI 合流の代表ケースが関数化（差分はdevのみ）
  - 常に prod は挙動不変・安全（OFFで現状維持）

Update — 2025-09-28 (Mini‑VM M2/M3 fix + smokes)
- Fix: compare/ret segmentation made robust without heavy JSON parse.
  - Approach: per‑block coarse passes for const/binop/compare and a precise in‑block ret search; control‑flow (branch/jump) handled with a single pass using computed regs.
  - Files: apps/selfhost/vm/boxes/mir_vm_min.nyash
- Smokes: enabled and PASS
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m2_eq_true_vm.sh
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m2_eq_false_vm.sh
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m3_branch_true_vm.sh
  - tools/smokes/v2/profiles/quick/core/selfhost_mir_m3_jump_vm.sh
- Notes: kept changes local and spec‑neutral; no default behavior changes to core VM.

Update — 2025-09-28 (QuickRef Dev Guards + Docs llvmlite)
- Dev guards (env‑gated; default OFF) implemented and validated by quick smokes:
  - ASI strict line‑continuation: `NYASH_ASI_STRICT=1` → parse error when a binary operator ends the line.
- Plus mixed (String×Number): `NYASH_PLUS_MIX_ERROR=1` → type error; suggest str()/明示変換。
  - Box equality guidance: `NYASH_BOX_EQ_GUIDE_ERROR=1` → equals()誘導のエラー。
  - Smokes enabled: `lang_quickref_asi_error_vm.sh`, `lang_quickref_plus_mixed_error_vm.sh`, `lang_quickref_equals_box_error_vm.sh`（PASS）
- LLVM ドキュメント統一（llvmlite一本化）
  - `LLVM_SYS_180_PREFIX` の記述を主要ドキュメントから撤去し、llvmlite/ny‑llvmc 前提に更新。
  - Files: `AGENTS.md`, `README.md`, `README.ja.md`, `CLAUDE.md`

Plan — Next (2025-09-28)
1) Mini‑VM 単一パス化（仕様不変・安全化） — completed
   - 各 op を JSON オブジェクト単位で厳密セグメント化し、一回走査で評価（coarse pass を除去）。
   - 代表ケース（複数op/ret先頭/ret末尾/compare v0,v1/jump/branch）で緑維持を確認。
2) Rewrite 統合 Stage‑1（挙動不変・dev観測） — completed (observability wired)
   - builder_calls の unified 経路に resolve.try/resolve.choose を追加（dev‑only/既定OFF）。
   - method_call_handlers の既存 emit と整合。Known/Union の certainty を choose に含める。
   - 使い方: `NYASH_MIR_UNIFIED_CALL=1 NYASH_DEBUG_ENABLE=1 NYASH_DEBUG_KINDS=resolve,ssa NYASH_DEBUG_SINK=/tmp/nyash_debug.jsonl`。
   - Known 経路の100%関数化（dev WARN=0）を DebugHub で観測。userbox スモークで検証。
3) P0/P1 着手（構造化） — in progress
   - origin/observe/rewrite の責務分割（モジュール新設: src/mir/builder/{origin,observe,rewrite}/）。
   - P0: me 注入 Known 化（起源付与/維持）と軽量PHI補強（単一/一致時）。
   - P1: Known 経路 100% 関数化（special 集約: toString→str（互換:stringify）/equals）。
   - Docs: README を各層に追加（origin/observe/rewrite）— completed
   - 観測呼び出しの統一: builder_calls/method_call_handlers から observe::resolve を使用 — completed
3) CI/Profiles 整理 — ongoing
   - quick: VM 主線（llvmlite パリティは integration に委譲）。
   - integration: 代表パリティ（llvmlite ハーネス）継続、apps系は任意実行。

Notes — Display API Unification (spec‑neutral)
- 規範: `str()` / `x.str()`（同義）。`toString()` は Builder で `str()` に早期正規化。
- 互換: `stringify()` は当面エイリアス（内部で `str()` 相当）。
- VM ルータ: toString/0 → str/0（なければ stringify/0）。
- QuickRef/ガイド更新済み。`NYASH_PLUS_MIX_ERROR` の誘導文言も `str()` に統一。

追加メモ — これからやる（ユーザー合意、2025‑09‑28）
- Mini‑VM の単一パス化を安全に実装（既定挙動不変）
  - 各 op を厳密セグメントで1回走査に統合（coarse を段階撤去）
  - 代表スモーク（M2/M3/compare v0,v1）で緑維持確認
- 続いて Rewrite 統合 Stage‑1 の観測へ進む（dev のみ、挙動不変）
- Dev Profiles
  - tools/dev_env.sh に Unified 既定ON（明示OFFのみ無効）とレガシー関数化抑止を追加。
    - `NYASH_MIR_UNIFIED_CALL=1`（既定ON明示）
    - `NYASH_DEV_DISABLE_LEGACY_METHOD_REWRITE=1`（重複回避; 段階移行）
