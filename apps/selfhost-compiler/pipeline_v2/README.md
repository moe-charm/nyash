Selfhost Compiler — Pipeline v2 (Box-First Skeleton)

Scope
- Thin, non-invasive box skeletons to express the pipeline structure in Nyash.
- Emit-only focus: parsing and JSON emission are performed; execution remains with the parent Runner.
- No new capabilities or extern calls are introduced in Phase 15.7 (spec invariant).

Boxes
- FlowEntryBox (emit-only): entry to PipelineV2. Accepts Stage‑1 JSON and returns MIR(JSON).
- ExecutionPipelineBox: orchestrates ParserBox → EmitterBox; optional BackendBox tag only.
- BackendBox (stub): records backend name ("vm"|"llvm"|"pyvm"); no execution.
- MirBuilderBox (stub/IF): future lowering and optimization entry (not wired in Phase 15.7).

Responsibilities
- Parse: use existing ParserBox (apps/selfhost-compiler/boxes/parser_box.hako).
- Emit（boxed）:
  - EmitReturnBox: return(Int) の最小 JSON 生成
  - EmitBinopBox: binop(lhs,rhs,kind)
  - EmitCompareBox: compare/branch/jump/ret（materialize有り/無し）
  - LocalSSABox: 材化（copy 挿入）ポリシーの集約
  - 既存の emit_mir_flow.hako は段階的に委譲→削減（互換維持）
  - Dev観測: `NYASH_EMIT_TRACE=1` を想定した最小トレース（現状は無条件1行出力。最終JSON行は変わらず）
- Execute: delegated to Rust Runner (parent→child). This directory must NOT perform execution.
  - Note: actual execution helper is provided under `apps/selfhost/vm/flow_runner.hako`.

Non-goals (Phase 15.7)
- No extern/FFI invocations to call backends from Ny code.
- No plugin or file I/O beyond what ParserBox already supports (keep emit path pure).

Usage (dev only, emit-only)
- Create an ExecutionPipelineBox and call `run_source(src, stage3_flag)`.
- The method prints a single JSON object to stdout and returns 0.
- Quiet acceptance is ensured by `NYASH_JSON_ONLY=1` (parent runner sets it for child).

Tracing (dev)
- PipelineV2 flow exposes a trace-enabled entry:
  - `PipelineV2.lower_stage1_to_mir(ast_json, prefer_cfg)` — default (trace=0)
  - `PipelineV2.lower_stage1_to_mir_trace(ast_json, prefer_cfg, trace)` — when `trace==1`, emit boxes print a single-line `[emit] ...` before JSON.
  - ExecutionPipelineBox は emit-only 経路であり、trace の布告は Runner 引数透過で後段導入予定（既定OFF）。

JSON v1 (MirCall) — experimental
- PipelineV2 provides an opt-in path to emit unified call form:
  - `PipelineV2.lower_stage1_to_mir_v1(ast_json, prefer_cfg)` — emits op:`mir_call` with `callee` payload (Global/Method/Constructor) and `args` array.
  - `PipelineV2.lower_stage1_to_mir_v1_compat(ast_json, prefer_cfg)` — emits v1 then adapts to legacy v0 (call/boxcall/newbox) via `selfhost.common.json.mir_v1_adapter`.
- Notes
  - Mini‑VM (MirVmMin) tolerates `op:"mir_call"` by treating its result as 0 (shape-only). Use the compat path to execute on MirVmMin.
  - Default smokes remain on v0. Additional quick shape smokes exist under `tools/smokes/v2/profiles/quick/selfhost/*_v1_shape_vm.sh`.

Fail-Fast
- If parsing returns null/empty, print an error to stderr and return non-zero.
- Do not print non-JSON lines when `NYASH_JSON_ONLY=1`.

LAYER GUARD
- This folder handles orchestration only.
- It MUST NOT:
  - perform runtime execution of MIR
  - manipulate plugins or unified registry
  - read files directly (allow the caller to pass source text)

Future (post 15.7)
- Wire MirBuilderBox for Ny→MIR(JSON v0) lowering with minimal op set.
- Introduce Backend execution delegation behind explicit flags (externs), guarded and opt-in.
