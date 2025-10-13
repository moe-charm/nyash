# MirIoBox (Hako ABI) — v0 Draft

Purpose
- Unify MIR emit/read under a single Hako ABI, eliminating formatting drift and making readers tolerant while keeping schema strict.

Scope (Phase A)
- JSON v0 only. Implement read_from_string, validate, and cursor helpers (functions→blocks→instructions→terminator).
- Backends: temporary Hako implementation using existing locator boxes; swap to yyjson-backed host plugin later.

API (minimal)
- `MirIoBox.validate(json) -> Result.Ok(()) | Result.Err(msg)`
- `MirIoBox.functions(json) -> Result.Ok(Map{content:<string>})`
- `MirIoBox.blocks(func_json) -> Result.Ok(Map{content:<string>})`
- `MirIoBox.instructions(block_json) -> Result.Ok(Map{content:<string>, single:0|1})`
- `MirIoBox.terminator(block_json) -> Result.Ok(<string>)` (returns the terminator object JSON)

Schema rules (strict)
- Root: `{kind:"MIR", schema_version:"1.0", functions:[...]}`
- Function: must contain `blocks:[...]`; `entry:int` recommended (Phase A emits it; readers prefer it).
- Block: `id:int`, `instructions:[...]`, `terminator:{...}` required.
- Terminator: one of ret/jump/branch with required fields; `ret.value` may be `null`.

Reader tolerance
- Allow whitespace after `:`. Key ordering non-significant. No trailing commas.

Backends
- Phase A: Hako locators (FunctionLocatorBox, BlocksLocatorBox, InstrsLocatorBox, BackwardObjectScannerBox).
- Phase B: Host plugin (yyjson) under the same ABI names. Hako VM and Rust/LLVM lines call into identical ABI.

Fail-Fast
- Missing required fields → Err.
- Bad references (next_bb not found) → Err.

Validation coverage (smokes)
- op whitespace variants; entry≠0; empty/single instructions; bad refs; terminator missing.

