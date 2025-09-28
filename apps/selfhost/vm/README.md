Layer Guard — selfhost/vm

Scope and responsibility
- Minimal Ny-based executors and helpers for self‑hosting experiments.
- Responsibilities: trial executors (MIR JSON v0), tiny helpers (scan/binop/compare), smoke drivers.
- Forbidden: full parser implementation, heavy runtime logic, code generation.

Imports policy (SSOT)
- Dev/CI: file-using allowed; drivers may embed JSON for tiny smokes.
- Prod: prefer `nyash.toml` mapping under `[modules.selfhost.*]`.

Notes
- MirVmMin covers: const/binop/compare/ret (M2). Branch/jump (M3) are supported minimally; phi is out of scope.
- Keep changes minimal and spec‑neutral; new behavior is gated by new tests.

Segment extraction (JSON v0)
- Instruction objects are sliced by brace‑balanced scanning:
  - For each `"op":"…"` found within a block’s `"instructions":[ … ]`, the object start is the nearest `{` before the `"op"` within the block; the end is the matching `}` by depth.
  - This guarantees that fields (`dst/lhs/rhs/value`) are read from the same object even when names repeat elsewhere in the block.
- Blocks are bounded by the bracket‑balanced end of the `instructions` array; inter‑block scanning is not performed.

Minimal JSON v0 expected profile
- `const` with `{ "value": { "type":"i64", "value": N } }`
- `binop`: either `op_kind` (Add/Sub/…) or textual `operation` (`+`, `-`, …)
- `compare`: either `cmp` (Eq/Ne/…) or textual `operation` (`==`, `!=`, …)
- `ret { "value": <reg> }`

Edge cases covered by quick/dev smokes
- Multiple `compare` in the same block followed by `ret` of the last result（v0/v1 mixed）
- `ret` placed at block head（未定義の値は0とみなす）/ block tail
- `branch`/`jump` minimal routing（then/else/targetを整数で指定）
