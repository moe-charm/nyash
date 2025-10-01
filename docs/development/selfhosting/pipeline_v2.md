Selfhost Pipeline v2 (Box-First, Emit-Only)

Goal
- Make the selfhost compiler pipeline explicit in Nyash code without changing Core behavior.
- Keep execution in the parent Runner; Ny boxes handle parse→emit and print one JSON line.

Boxes
- ExecutionPipelineBox: orchestrates ParserBox and EmitterBox; optional backend tag.
- BackendBox (stub): stores backend name only (no execution in Phase 15.7).
- MirBuilderBox (stub): reserved for Ny→MIR(JSON v0) lowering; pass-through today.

Constraints (Phase 15.7)
- No extern calls from Ny to invoke Rust VM/LLVM/PyVM; execution is a Runner responsibility.
- No new default-off toggles required for basic emit-only flow.
- Quiet acceptance: when `NYASH_JSON_ONLY=1`, prints exactly one JSON object line to stdout.

Folder Layout
- apps/selfhost-compiler/
  - boxes/           (ParserBox / EmitterBox)
  - pipeline_v2/     (ExecutionPipelineBox / BackendBox / MirBuilderBox)
  - README.md        (safe profile and acceptance)
  - INTERFACES.md    (contracts)

Usage (dev)
- Construct `ExecutionPipelineBox` and call `run_source(src, stage3_flag)`.
- For Runner-driven acceptance, keep using `NYASH_USE_NY_COMPILER=1 ... NYASH_JSON_ONLY=1`.

Smokes (quick)
- Child path (emit-only): `tools/smokes/v2/profiles/quick/selfhost/selfhost_min_json_header_pipeline_v2_vm.sh`
- Direct driver (emit-only): `tools/smokes/v2/profiles/quick/selfhost/selfhost_pipeline_v2_driver_min_json_vm.sh`

Fail-Fast
- Empty/invalid JSON emission must return non-zero and avoid stdout noise.
- Diagnostics go to stderr; quiet mode suppresses non-essential logs.

Future Work (post 15.7)
- Wire MirBuilderBox to output minimal MIR(JSON v0).
- Introduce guarded externs for backend execution (opt-in, default OFF).
