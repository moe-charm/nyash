# Builder Diagnostics (dev-only)

Purpose
- Collect the most useful flags and short outputs in one place.
- Default builds remain quiet; use these locally when chasing resolution/SSA issues.

Flags (env)
- NYASH_RESOLVE_TRACE=1
  - Show `[resolve.try]` candidates and basic receiver info.
- NYASH_LOCAL_SSA_TRACE=1
  - Show LocalSSA materialization and `[vm-call-final]` (final callee, recv %id; ty/origin attached).
- NYASH_MAT_TRACE=1
  - Show `[mat-trace]` with the last 5 instructions and a one-liner of receiver/args types.
- NYASH_ROUTER_TRACE=1
  - Show RouterPolicy decisions (route, reason, class, method, arity, certainty).
- NYASH_VARMAP_TRACE=1
  - Show `[varmap] tag=… recv=%ID names=[…] map_size=N` — names currently bound to the receiver ValueId.
- NYASH_BLOCK_SCHEDULE_VERIFY=1
  - Warn if block order breaks the `PHI → materialize → body` contract.

Tips
- Minimal selfhost VM chase:
  ```bash
  NYASH_USING=1 NYASH_ALLOW_USING_FILE=1 NYASH_USING_STRATEGY=prelude \
  NYASH_RESOLVE_TRACE=1 NYASH_LOCAL_SSA_TRACE=1 NYASH_MAT_TRACE=1 NYASH_VARMAP_TRACE=1 \
  ./target/release/nyash --backend vm apps/selfhost-compiler/compiler.hako -- --min-json --stage3   # 互換: .nyash も受理
  ```
- Look for clusters of:
  - `[vm-call-final] … class=DebugBox|ConsoleBox` on string methods → likely receiver mis-binding.
  - Pair with `[varmap]` to see which names alias the receiver.
  - Use `[mat-trace]` to confirm in-block materialization right before the call.

Notes
- All diagnostics are zero-cost when flags are off.
- Keep prod quiet; never leave these flags on in CI unless the job opts in.
