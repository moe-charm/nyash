Selfhost Compiler Interfaces (Draft)

Purpose
- Document minimal, stable interfaces between boxes to keep responsibilities clear and changes local.
- Apply Fail-Fast: interfaces return explicit errors; no implicit fallbacks.

Boxes and Roles
- ParserBox: source text -> Stage‑1 JSON (AST-ish)
  - Methods:
    - stage3_enable(flag: i64) -> Void
    - extract_usings(src: String) -> Void
    - get_usings_json() -> String  // JSON array `[{name, path?}, ...]`
    - parse_program2(src: String) -> String  // Stage‑1 JSON
- EmitterBox: Stage‑1 JSON + usings -> JSON v0 (header+body)
  - Methods:
    - emit_program(json: String, usings_json: String) -> String // JSON v0 line
- Pipeline v2 (emit‑only; Phase 15.7)
  - ExecutionPipelineBox (apps/selfhost-compiler/pipeline_v2)
    - birth(name: String="vm") -> i64  // backend tag only
    - run_source(src: String, stage3_flag: i64=0) -> i64  // prints JSON v0; 0 on success
  - BackendBox (stub)
    - birth(name: String) -> i64
    - get_name() -> String
    - execute(mir_json: String) -> i64   // reserved; returns 0 in Phase 15.7
  - MirBuilderBox (stub)
    - build(ast_json: String) -> String  // pass-through
  - LocalSSABox（new, minimal）
    - ensure_after_phis_copy(insts: ArrayBox, src: i64, dst: i64) -> i64  // append copy(dst,src); returns 0 on success
    - add_copy(insts: ArrayBox, dst: i64, src: i64) -> i64

Contracts
- ParserBox.parse_program2 returns a single-line JSON when `NYASH_JSON_ONLY=1`.
- EmitterBox.emit_program returns a single-line JSON v0 with non-empty header, and `meta.usings` present (possibly empty array).
- Pipeline v2 prints exactly one JSON line to stdout and returns 0 on success.

Control Flow Lowering (Phase 15.7 — P2)
- Goal: Lower simple `if (cond) { then; return X; } else { return Y; }` to MIR with `branch/jump/ret` only, no PHI.
- Invariants (minimal):
  - Condition value is materialized in the current block before `branch`.
  - `then`/`else` blocks both end with `ret` (Phase 15.7 scope)。
  - No fallthrough. Merge block is not required for this minimal form.
  - No SSA PHI needed as both sides terminate.
- Pseudocode
  - Input (AST-like JSON): `If { cond: vC, then: [.. ret vT], else: [.. ret vE] }`
  - Output (MIR):
    - `%c = (cond)`
    - `branch %c -> %bb_then, %bb_else`
    - `%bb_then: ... ret %vT`
    - `%bb_else: ... ret %vE`
- Future (out-of-scope for P2):
  - 非終端 then/else の合流に PHI を用いる（Phase 16以降）。
  - `ensure_cond` による in‑block の Copy/材化最適化（P2で最小版を導入）。

LocalSSA.ensure_cond（最小）
- 目的: 分岐直前に使う値が “このブロック内で定義済み” になるように、必要なら Copy を発行する。
- 仕様（最小）:
  - ブロック先頭の PHI 群の直後に Copy を挿入（PHIの後、最初の通常命令の前）。
  - 呼び出し（call/boxcall）直前にレシーバ/引数に対しても材化を許容（P2では代表ケースのみ）。
  - 戻り値: ローカルに材化された ValueId。
 - 実装（Phase 15.7の範囲）:
   - LocalSSABox.ensure_after_phis_copy を使用。PHIスキップは今後の拡張で実装。

Environment (Runners use these)
- NYASH_USE_NY_COMPILER=1   // enable Ny child compiler path
- NYASH_NY_COMPILER_MIN_JSON=1  // append --min-json to child
- NYASH_NY_COMPILER_CHILD_ARGS="--emit-mir ..."  // child args passthrough
- NYASH_JSON_ONLY=1         // quiet: stdout=JSON only
 - NYASH_QUIET=1            // force quiet logs (no non-essential stderr)
- NYASH_VM_USE_PY=1         // execute MIR via PyVM (default: Rust VM)
- NYASH_NY_COMPILER_TIMEOUT_MS=2000 // child timeout

Non-Goals (Phase 15.7)
- Complex optimization passes, dynamic plugin loading, or large I/O are out of scope.
- Behavior changes to Core VM/LLVM are prohibited; all changes are local and flag-guarded.

Acceptance (dev)
- JSON v0 header non-empty when --min-json is set.
- Minimal MIR(JSON v0) executes under Rust VM with quick smokes.
