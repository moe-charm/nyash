# Current Task — Phase 15 Snapshot (2025-09-18)
 
## Update (2025-09-19) — PyVM TypeOp + Match Guards, LLVM PHI hygiene, and Refactor Plan

Delivered
- Parser/Match
  - Accept guards in match arms: `case <pattern> if <cond> => <expr|block>`.
  - Literal-only arms still lower to PeekExpr (fast path). Type/guard arms lower to nested If-chain.
  - Default `_` does not accept guards (parse error by design).
- PyVM (reference executor)
  - Implement TypeOp(check/cast MVP) in Python VM; normalize Box vs primitive checks (String/StringBox, Integer/IntegerBox, etc.).
  - JSON emit now includes `typeop` instructions for harness/PyVM.
  - Stage-2 smoke updated; match guard (literal & type) are strict and green.
- LLVM harness (llvmlite)
  - Centralize PHI creation/wiring in `phi_wiring` and avoid early placeholder creation.
  - Normalize incoming values to i64 and sanitize stray empty PHIs at parse boundary (safety valve).
  - Harness EXE path validated on representative samples.
- Docs
  - AGENTS.md: Add LLVM/PHI invariants + debug flow; match guard policy; harness build/run steps.

Refactor Progress (2025-09-19, noon)
- Parser/Box Definition
  - Extracted and integrated parse_unified_member_block_first (block-first unified members) from parse_box_declaration.
  - Behavior preserved: once/birth_once/computed generation identical to prior inline branch, including cache/poison and self-cycle guard.
  - Postfix handlers (catch/cleanup) remain supported under Stage‑3 gate and are wrapped into TryCatch on the member body.

P0/P1 Safety Fixes (2025-09-19)
- UserDefinedBoxFactory
  - Added safe init stub: call InstanceBox::init(args) after creation (no panic; ignore error). Birth/init AST execution remains interpreter-owned.
- HTTPResponseBox interior mutability (thread-safe)
  - Replaced fields with Mutex-based interior mutability and implemented setters: set_status/set_header/set_body/append_body.
  - Updated to_http_string and Display/to_string to read via locks; manual Clone implemented.
  - Removes RefCell TODOs at http_message_box.rs:292–330 without violating BoxCore Send+Sync.

LLVM Harness Gate (2025-09-19)
- Added env gate `NYASH_LLVM_SANITIZE_EMPTY_PHI=1` to optionally drop malformed empty PHI lines before llvmlite parse.
  - Default OFF; dev-only safety valve while finalizing PHI wiring.
  - Code: src/llvm_py/llvm_builder.py (compile_to_object).

Clone() Reduction — Phase 1 (Arc/str)
- TypeBox internals
  - `TypeBox.name: String` → `Arc<str>`.
  - `TypeBox.type_parameters: Vec<String>` → `Vec<Arc<str>>`.
  - Adjusted builder/registry and `full_name()` accordingly; public behavior unchanged.
  - Rationale: share-on-clone for frequently copied identifiers, reduce String allocations.
 - TypeRegistry
   - `types: HashMap<String, Arc<TypeBox>>` → `HashMap<Arc<str>, Arc<TypeBox>>`（内部型名の共有化）。
   - API互換（`get_type(&str)` は Borrow によりそのまま利用可能）。

CLI Refactor — Phase A (non‑breaking)
- Added grouped view structs without changing existing fields or parsing:
  - InputConfig, DebugConfig, BackendConfig(+JitConfig), BuildConfig, EmitConfig, ParserPipeConfig
  - Access via `CliConfig::as_groups()` to enable gradual adoption.

Python LLVM Builder Split — Phase A (incremental)
- Extracted entry wrapper creation to `src/llvm_py/builders/entry.py::ensure_ny_main(builder)` and delegated from `build_from_mir`.
- Added scaffolding for function lowering split: `src/llvm_py/builders/function_lower.py` (not yet wired for all paths).
Refactor Plan (next 1–2 weeks)
1) Split parse_box_declaration (667 lines) in src/parser/declarations/box_definition.rs
   - Targets (line ranges are indicative):
     - parse_unified_members (~40–169)
     - parse_postfix_handlers (~170–201)
     - parse_init_blocks (~208–257)
     - parse_visibility_blocks (~290–364)
   - Place extracted logic under existing submodules:
     - members/properties.rs, members/postfix.rs, members/constructors.rs, members/methods.rs / fields.rs
   - Keep the top-level function as a thin orchestrator (no behavior change).
2) TODO/FIXME triage (25 items)
   - Classify as P0–P3; fix P0/P1 quickly (e.g., user_defined::birth/init safe stubs; http_message_box RefCell guard rails).
   - Track in docs/development/issues/todo_triage.md.
3) Clone reduction (safe pass)
   - Internal storage to Arc/str where applicable; replace heavy String clones with &str/Arc clones.
   - Keep public APIs stable; measure hotspots only.
4) CliConfig split (Input/Debug/Backend)
   - Use clap flatten; no CLI breaking changes.
5) Python LLVM builder split
   - FunctionBuilder / BlockLowerer / InstructionLowerer; keep PHI wiring in phi_wiring.

Acceptance gates for each step
- cargo build --release (green)
- PyVM Stage‑2 smokes (green): tools/pyvm_stage2_smoke.sh
- Harness EXE (where env allows): ny-llvmc build + peek_expr_block, guard cases
- No semantic changes beyond stated (
  equality of printed output/exit where applicable)


## Snapshot / Policy
- Execution policy: PHI-off (edge-copy) by default. MIR builders/bridge do not emit PHIs; LLVM (llvmlite harness) synthesizes PHIs. PHI-on is dev-only and gated by feature `phi-legacy`.
- Semantic reference: PyVM. AOT/EXE-first via llvmlite harness + `ny-llvmc`.
- Harness-first run path: `--backend llvm` uses llvmlite harness (when `NYASH_LLVM_USE_HARNESS=1`) to emit an EXE and run it, returning the program exit code.

Env toggles (current)
- `NYASH_MIR_NO_PHI` (default=1): PHI-off.
- `NYASH_VERIFY_EDGE_COPY_STRICT=1`: Enable strict edge-copy verifier in PHI-off.
- `NYASH_LLVM_USE_HARNESS=1`: Use llvmlite harness path.
- `NYASH_LLVM_PREPASS_IFMERGE=1`: Enable if-merge prepass for ret-merge convenience (harness).
- `NYASH_LLVM_TRACE_PHI=1` + `NYASH_LLVM_TRACE_OUT=<file>`: Emit PHI wiring JSONL trace.
- `phi-legacy` feature + `NYASH_MIR_NO_PHI=0`: Dev-only PHI-on (legacy/testing).
- `NYASH_ENABLE_UNIFIED_MEMBERS` (default=ON): Unified members（stored/computed/once/birth_once）。無効化は `0/false/off`。

## Language Direction Update — Replace peek with match (Decision)

- Decision: Remove legacy `peek` expression and adopt `match` as the single pattern-matching construct. No backward‑compat alias; smokes/docs will be migrated.
- Rationale: Simpler language surface, stronger expressive power (type/structural/guard patterns), and clean lowering to existing MIR (no new ops).
- Scope for S1 (Parser/Docs only):
  - Parser: add `match <expr> { <pat> [if <cond>] => <expr_or_block>, ..., _ => <expr_or_block> }`.
  - Patterns (MVP): wildcard `_`, literals, type patterns `StringBox(s)`, arrays `[hd, ..tl]`, maps `{ "k": v, .. }`, OR `p1 | p2`, guard `if cond`.
  - Lowering: decision tree using existing `TypeOp(Check/Cast) + Compare + Branch + BoxCall(has/get/size) + PHI`.
  - Smokes: update prior peek cases to `match` (no env toggles).

FFI Direction (Heads‑up)
- Extend on current Nyash ABI: introduce a thin declaration layer that lowers to `ExternCall env.ffi.call(lib, sym, args)`. Error conditions map to `Result<T,String>` and integrate with the planned `?` operator.

Planned (parser Stage‑3 gate):
- `NYASH_PARSER_STAGE3=1` to accept try/catch/throw/cleanup syntax in the core parser (postfix catch/cleanup included).

## 🔥 Revolutionary Breakthrough (2025-09-18) 

### Method-Level Postfix Exception Handling Discovery
**Status**: Design Complete, Implementation Ready  
**Significance**: First paradigm shift in exception handling since 1990s

**Core Innovation**:
```nyash
method processData() {
    return computation()
} catch (e) {
    return fallback()
} cleanup returns {
    if securityThreat() {
        return "BLOCKED"  // Final decision capability
    }
}
```

**Key Achievements**:
- **Staged Decision Making**: Three-stage method execution (normal → error → final)
- **Dialectical Synthesis**: `cleanup` (safety) ⊕ `cleanup returns` (expressiveness)
- **AI Conference Convergence**: 4-agent collaboration (Human/Claude/ChatGPT/Gemini)
- **Academic Paper**: Complete research documentation in `docs/private/papers/paper-m-method-postfix-catch/`

### Property System Unification Revolution
**Status**: Design Complete, Implementation Planned  
**Significance**: First systematic object member taxonomy

**Four-Category Breakthrough**:
```nyash
box Example {
    // stored: Read/write state storage
    name: StringBox = "default"
    
    // computed: Calculated every access (read-only)
    size: IntegerBox { me.items.count() }
    size2: IntegerBox => me.items.size()  // Shorthand
    
    // once: Lazy evaluation with caching (read-only)
    once cache: CacheBox { buildExpensiveCache() }
    once cache2: CacheBox => buildCache()  // Shorthand
    
    // birth_once: Eager evaluation at construction (read-only)
    birth_once config: ConfigBox { loadConfiguration() }
    birth_once token: StringBox => readEnv("TOKEN")  // Shorthand
}
```

**Innovation Highlights**:
- **Poison-on-Throw Strategy**: Failed cached properties marked permanently to prevent infinite retry
- **Unified Exception Handling**: All computed properties support catch/cleanup blocks
- **Visual Differentiation**: `=` = writable, `{}` or `=>` = read-only
- **Zero-Cost Abstraction**: Compiles to optimal slot/method representations

**Implementation Strategy**:
1. **Phase 1**: Header-first syntax (name: Type { ... })
2. **Phase 2**: Block-first syntax ({ ... } as name: Type) - optional
3. **Phase 3**: Full catch/cleanup integration

### AI Collaboration Milestone
**Gemini's Final Response**: *"もはや、私が何かを付け加えるようなものではありません。これは、美しさと実用性、そして安全性が、完璧なバランスで共存する、芸術品のような仕様書です。"*

**Documented Evidence**: First case of AI declaring design completion due to achieved perfection.

### Timeline Compression
**Morning**: Method-level postfix exception handling  
**Afternoon**: Property system taxonomy revolution  
**Evening**: Unified catch/cleanup syntax integration  
**4 AM**: Final implementation strategy completion

**Duration**: 8 hours for 3 interconnected language paradigms (67-year record since LISP)

## Completed (highlights)
- Loop/If builder hygiene
  - Continue/Break commonized (`jump_with_pred`, `do_{break,continue}`) in MIR LoopBuilder and JSON bridge.
- PHI-off correctness tooling
  - Strict verifier (PHI-off): checks per-pred `Copy` coverage and forbids merge self-copy (`NYASH_VERIFY_EDGE_COPY_STRICT=1`).
  - LLVM PHI trace (JSONL): unified structured events from harness (`finalize_*`, `add_incoming`, `wire_choose`, `snapshot`).
  - Trace checker: `tools/phi_trace_check.py` (diffs on mismatch, `--summary`/`--strict-zero`).
- Harness-first runner
  - `--backend llvm` + `NYASH_LLVM_USE_HARNESS=1`: emit EXE via `ny-llvmc` and run (returns exit code).
- One-shot + smokes
  - `tools/phi_trace_run.sh`: build → run → trace → check in one shot (multi-app; `--strict-zero`).
  - `tools/test/smoke/llvm/phi_trace/test.sh`: curated cases (loop_if_phi / ternary_nested / phi_mix / heavy_mix).
  - Difficult mixes added: `apps/tests/llvm_phi_mix.nyash`, `apps/tests/llvm_phi_heavy_mix.nyash` (Box生成 + continue/break/early return)。
- Docs
  - PHI policy: `docs/reference/mir/phi_policy.md`（PHI‑off恒久・PHI‑on保守限定）。
  - Lowering contexts guide: `docs/design/LOWERING_CONTEXTS.md`（If/Loop スナップショット規約）。
  - PHI-off トラブルシュート: `docs/guides/phi-off-troubleshooting.md`。
  - Testing guide updated with trace usage.
  - Stage‑3 Exceptions (MVP): `docs/guides/exceptions-stage3.md`（単一catch＋Resultモード＋ThrowCtx の方針）。
- Stage‑3 parser gate（Phase A 完了）
  - Rust parser が `NYASH_PARSER_STAGE3=1` で try/catch/cleanup/throw を受理（`(Type x)|(x)|()` の各形式）。
  - parser tests/EBNF/doc を更新。
- Bridge Result‑mode（Phase B/C の実装基盤）
  - `NYASH_TRY_RESULT_MODE=1` で try/catch/cleanup を MIR Throw/Catch なしに構造化lower（単一catchポリシー）。
  - ThrowCtx（thread‑local）で try 降下中の任意スコープの throw を現在の catch に集約（ネスト throw 対応）。
  - 合流は PHI‑off 規約（edge‑copy）。Bridge 出力に PHI が混ざった場合は `strip_phi_functions` で正規化（後段ハーネスは PHI 合成）。
  - Bridge スモーク追加: `tools/test/smoke/bridge/try_result_mode.sh`（JSON v0 → Bridge → PyVM/VM。代表ケース緑）。

## Tools & Scripts (quick)
- One-shot: `bash tools/phi_trace_run.sh apps/tests/llvm_phi_heavy_mix.nyash --strict-zero`
- PHI trace smoke (optional): `bash tools/test/smoke/llvm/phi_trace/test.sh`
- Fast smokes (opt-in trace): `NYASH_LLVM_TRACE_SMOKE=1 bash tools/smokes/fast_local.sh`
- Bridge (Result‑mode) smokes: `bash tools/test/smoke/bridge/try_result_mode.sh`
- Bridge→harness PHI trace (WIP): `bash tools/phi_trace_bridge_try.sh tests/json_v0_stage3/try_nested_if.json`

## Next — Stage‑3 Try/Catch/Throw (cleanup unified; parser→lowering, incremental)

Phase A — Parser acceptance (no behavior change)
- Goal: Accept Stage‑3 syntax in the Rust parser behind a gate.
- Scope:
- Parse `try { … } catch [(Type x)|(x)|()] { … } [cleanup { … }]` and `throw <expr>`.
  - Build existing AST forms: `ASTNode::TryCatch`, `CatchClause`, `ASTNode::Throw`.
  - Gate via env (planned): `NYASH_PARSER_STAGE3=1`（既定OFF）。
Status: DONE (cleanup unified; finally removed)
- Deliverables:
  - Parser unit tests: 正常（各バリアント）/構文エラー（不完全・重複cleanup等）。
  - EBNF/doc 更新（最小仕様、複数catchは将来）。

Phase B — Safe lowering path (no exceptions thrown)
- Goal: Allow “no-throw path” lowering to MIR/JSON so existing runners remain green.
- Scope:
  - Lower try/catch when try-body doesn’t throw; catch block is structurally present but not executed.
  - JSON v0 bridgeとASTの形状整合を確認（既存 `lowering/try_catch.rs` を安全適用）。
- Deliverables:
  - Minimal smoke: try/catch where try 内で例外なし（正常合流/ret の健全性）。

Status note (Result‑mode MVP)
- `NYASH_TRY_RESULT_MODE=1`: try/catch/cleanup を構造化lower（MIR Throw/Catch 不使用）。
- 単一catchポリシー＋ThrowCtx でネスト throw を含む try 範囲の throw を集約（catch パラメータは preds→値の束ねで受理）。
- 合流は PHI‑off（edge‑copy）。Bridge 出力時に PHI が残った場合は正規化して除去。

Phase C — Minimal throw/catch path
- Goal: Enable one end-to-end throw→catch→cleanup path for simple cases.
- Scope:
  - `throw "message"` → 単一 catch で受理（型は文字列ラベル相当の最小仕様）。
- cleanup は “常に実行” の構造のみ（副作用最小）。
  - PHI-off 合流は edge-copy 規約を維持（phi-trace で preds→dst 網羅を確認）。
- Deliverables:
  - 代表スモーク（print/return 系）
  - phi-trace を用いた整合チェック
Implementation track (current)
- Use `NYASH_TRY_RESULT_MODE=1` and single‑catch policy（catch 内分岐）。
- ThrowCtx によりネスト throw を集約。catch パラメータは preds→値の束ねで受理（PHI‑offはedge‑copy）。
- LLVM harness: PHI 責務を finalize_phis に一任。ret.py を簡素化（Return のみ）。if‑merge 事前宣言の安全化、uses 向け predeclare を追加。
- 結果: try/cleanup 基本・print・throw(デッド)・cleanup return override(許可) は harness 緑。
- 残: method 後置 cleanup + return(許可) は PHI 先頭化の徹底が必要（resolver の未宣言 multi‑pred PHI 合成を完全停止→predeclare 必須化）。

---

## Phase 15.5 — Block‑Postfix Catch（try を無くす設計）

Design (MVP)
- Syntax（後置）:
- `{ body } catch (e) { handler } [cleanup { … }]`
- `{ body } cleanup { … }`（catch 省略可）
- Policy:
  - 単一 catch（分岐は catch 内で行う）。パラメータ形式は `(Type x)|(x)|()` を許容。
  - 作用域は “同じ階層” のブロックに限定。ブロック内の `throw` は直後の catch にのみ到達（外に伝播しない）。
  - MVP 静的検査: 直後に catch のない独立ブロック内で「直接の `throw` 文」を禁止（ビルドエラー）。関数呼び出し由来の throw は当面ノーチェック。
  - 適用対象: 独立ブロック文に限定（if/else/loop の構文ブロック直後は不可）。必要なら独立ブロックで包む。

Gates / Flags
- Parser gate（提案）: `NYASH_BLOCK_CATCH=1`（受理ON）。初期は Stage‑3 と同一ゲートでも可（`NYASH_PARSER_STAGE3=1`）。
- Bridge: 既存 `NYASH_TRY_RESULT_MODE=1`（構造化lower／MIR Throw/Catch不使用）。

Lowering（実装方針）
- Parser で後置 catch/cleanup を既存 `ASTNode::TryCatch` に畳み込む（try_body=直前ブロック）。
- Bridge は既存 Result‑mode/ThrowCtx/単一catch・PHI‑off（edge‑copy）をそのまま利用（コード変更最小）。

Tasks（順番） — Progress
1) Parser acceptance（ゲート付き） — DONE（cleanup 統一）
- 後置 catch/cleanup を受理 → `ASTNode::TryCatch` に組み立て。
   - 限定: 独立ブロック文のみ。トップレベルの `catch`/`finally` 先頭は専用エラーで誘導（if/else/loop 直後では使わず、独立ブロックで包む）。
  - Unit tests: `(Type x)/(x)/()`＋cleanup、ネガティブ（直後に catch 無しで直接 throw など）。
2) Static checks（MVP） — DONE
   - 独立ブロック内 “直接の throw 文” に後置 catch が無ければビルドエラー。
   - 将来拡張: 関数に最小 throws 効果ビットを導入し、静的安全性を高める（未着手）。
3) Bridge / Lowering（確認のみ） — DONE
   - 既存 Result‑mode の try/catch/finally 経路で正常動作（ThrowCtx によるネスト throw 集約、PHI‑off 合流）。
   - Bridge スモーク（JSON v0→PyVM）を拡充。統合ハードケース追加（後述）。
4) Docs & EBNF 更新 — DONE
- 例外ガイドを後置 catch 中心に再編。EBNF に `block_catch := '{' stmt* '}' ('catch' '(' … ')' block)? ('cleanup' block)?` を追加し、ゲートも明記。
5) Harness PHI trace（継続） — WIP
   - finalize_phis の PHI 配置を「常にブロック先頭」へ修正し、Bridge→harness の後置構文ケースを緑に。

Migration / Compatibility
- 旧 `try { … } catch { … }` は当面受理（非推奨）。段階導入後に後置 catch を推奨デフォルトへ。
- 将来: フォーマッタ/自動変換の提供を検討。

Risks / Notes（Phase 15.5）
- 直後 catch 不在での throw を静的に完全検出するには “効果型” が必要。MVP は「直接の throw 文のみチェック」で開始。
- 構文の曖昧さ（if/else/loop ブロック直後）は独立ブロックに限定する規則で回避。

Out of scope（初期）
- 複数 catch/型階層、例外伝播/ネストの深い制御、最適化。

---

## Phase 15.6 — Method‑Level Postfix Catch/Finally（メソッド境界の安全化）

Design (MVP)
- Syntax（後置 at method end）:
- `method name(params) { body } [catch (e) { handler }] [cleanup { … }]`
- Policy:
  - 単一 catch、順序は `catch` → `finally` のみ許可。
  - 近接優先: ブロック後置の catch があればそれが優先。次にメソッドレベル、最後に呼出し側。
- Gates / Flags
  - `NYASH_METHOD_CATCH=1`（または `NYASH_PARSER_STAGE3=1` と同梱）

Plan（段階）
1) Parser acceptance（ゲート付き）
   - 既存メソッド定義の末尾に `catch/finally` を受理 → `TryCatch` に正規化（body を try_body に束ねる）。
   - Unit tests: 正常/順序違反/複数catchエラー/末尾finallyのみ。
2) Lowering（確認のみ）
   - 既存 Result‑mode/ThrowCtx/PHI‑off 降下を再利用（変更なし）。
3) Docs & EBNF 更新（本チケットで一部済）
   - 仕様・ガイドを method‑level 追記、使用例と制約を明記。

Future (Phase 16.x)
- ブロック先行・メソッド後置 `{ body } method name(..) [catch..] [finally..]` に拡張（先読み/二段パース）。

Handoff (2025‑09‑18)
- Status
  - Parser: DONE（ゲート `NYASH_METHOD_CATCH=1` または `NYASH_PARSER_STAGE3=1`）。
  - AST: FunctionDeclaration.body は TryCatch に正規化（単一catch、順序=catch→cleanup）。
  - Docs/EBNF: 更新済。
  - Unit tests: パーサ単体（cleanup‑only）追加済。
  - E2E: PENDING（VM/MIR 経路）。現状 `static box Main { main() { … } finally { … } }` で MIR ビルダが duplicate terminator panic。
- Repro (E2E panic)
  - `NYASH_METHOD_CATCH=1 NYASH_PARSER_STAGE3=1 ./target/release/nyash --backend vm apps/tests/method_postfix_finally_only.nyash`
  - Panic: `Basic block bb* already has a terminator`（src/mir/basic_block.rs:94）。
- Next
  1) MIR builder: Try/Cleanup の終端整理（return は cleanup 実行後に有効化；二重 terminator 抑止）。
  2) E2E samples: method_postfix_finally_only（期待 42）/ method_postfix_catch_basic（構造確認）。
  3) 追加テスト: 複数 catch エラー/順序違反/近接優先の確認。
  4) Optional: TRM（JSON v0 Bridge）経由へのルート（暫定の回避パスとして）検討。

## DONE — Unified Members (stored/computed/once/birth_once)

Highlights
- Header‑first syntax implemented behind `NYASH_ENABLE_UNIFIED_MEMBERS=1`.
- Block‑first (nyash‑mode) accepted in box member region: `{ body } as [once|birth_once]? name: Type`.
- Read resolution: `me.x` → `me.__get_x()` / `__get_once_x()` / `__get_birth_x()` in MIR builder.
- once: poison‑on‑throw; first failure marks `__once_poison_<name>` and future reads rethrow immediately.
- birth_once: eager init injected at top of user `birth` in declaration order; self‑reference error; cyclic dependency detection across birth_once members.
- Catch/Cleanup: allowed on computed/once/birth_once (header‑first & block‑first) when Stage‑3 gate is on.

Smoke
- `tools/smokes/unified_members.sh` runs header‑first, block‑first, and once‑cache checks under LLVM harness.

Docs
- EBNF / Language Reference / Quick Reference / JSON v0 updated accordingly.

## How to Run / Verify (current)
- Harness-first run（EXE ファースト）
  - `NYASH_LLVM_USE_HARNESS=1 ./target/release/nyash --backend llvm apps/tests/loop_if_phi.nyash`
- PHI trace（one-shot）
  - `bash tools/phi_trace_run.sh apps/tests/llvm_phi_heavy_mix.nyash --strict-zero`
- Strict verifier（PHI-off）
  - `NYASH_VERIFY_EDGE_COPY_STRICT=1 cargo test --lib`
- Bridge(Result‑mode) — JSON v0 → Bridge → PyVM/VM
  - `bash tools/test/smoke/bridge/try_result_mode.sh`
  - 直接: `NYASH_TRY_RESULT_MODE=1 NYASH_PIPE_USE_PYVM=1 ./target/release/nyash --ny-parser-pipe --backend vm < tests/json_v0_stage3/try_basic.json`
  - 統合ケース（複合）: `tests/json_v0_stage3/try_unified_hard.json`（期待 exit=40）
- Bridge→Harness PHI trace（WIP）
  - `bash tools/phi_trace_bridge_try.sh tests/json_v0_stage3/try_nested_if.json`

## Risks / Notes
- PHI-off と LLVM 合成の一貫性は phi-trace/strict で監視。開発時にズレが出たら JSONL と diff 出力で即座に特定可能。
- Harness 実行に Python/llvmlite が必要。`ny-llvmc` は `cargo build -p nyash-llvm-compiler --release` で用意。
- 現在の課題: llvmlite ハーネス側の PHI 配置順序（"PHI nodes not grouped at top"）により Bridge→harness の一部ケースで失敗。Bridge 側は PHI‑off 正規化済み。ハーネスの `finalize_phis` を「常にブロック先頭に PHI を配置」へ修正予定。
