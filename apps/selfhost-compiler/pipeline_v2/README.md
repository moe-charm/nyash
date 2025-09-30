Selfhost Compiler — Pipeline v2 (Box-First Skeleton)

Scope
- Thin, non-invasive box skeletons to express the pipeline structure in Nyash.
- Emit-only focus: parsing and JSON emission are performed; execution remains with the parent Runner.
- No new capabilities or extern calls are introduced in Phase 15.7 (spec invariant).

Boxes
- ExecutionPipelineBox: orchestrates ParserBox → EmitterBox; optional BackendBox tag only.
- BackendBox (stub): records backend name ("vm"|"llvm"|"pyvm"); no execution.
- MirBuilderBox (stub/IF): future lowering and optimization entry (not wired in Phase 15.7).

Responsibilities
- Parse: use existing ParserBox (apps/selfhost-compiler/boxes/parser_box.hako).
- Emit: use existing EmitterBox (apps/selfhost-compiler/boxes/emitter_box.hako).
- Execute: delegated to Rust Runner (parent→child). This directory must NOT perform execution.

Non-goals (Phase 15.7)
- No extern/FFI invocations to call backends from Ny code.
- No plugin or file I/O beyond what ParserBox already supports (keep emit path pure).

Usage (dev only, emit-only)
- Create an ExecutionPipelineBox and call `run_source(src, stage3_flag)`.
- The method prints a single JSON object to stdout and returns 0.
- Quiet acceptance is ensured by `NYASH_JSON_ONLY=1` (parent runner sets it for child).

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
