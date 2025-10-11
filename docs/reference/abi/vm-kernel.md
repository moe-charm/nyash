# NYABI: VM Kernel Bridge (Draft)

Scope
- Provide a minimal, stable ABI for delegating selected VM policies/decisions to Ny code.
- Keep default behavior unchanged (OFF by default). Ny Kernel is a development aid only.

Design Principles
- Coarse-grained: call per feature, not per instruction (avoid hot-path crossings).
- Fail‑Fast: any error returns immediately; no silent fallbacks in dev.
- Safe by default: re‑entry forbidden; each call has a deadline (timeout).
- Backward compatible: API evolves via additive fields and optional methods only.

API (v0, draft)
- VmKernel.caps() -> { version: i64, features: [string] }
- VmKernel.stringify_policy(type: string) -> string
  - Returns: "direct" | "rewrite_stringify" | "fallback"
- VmKernel.equals_policy(lhs_type: string, rhs_type: string) -> string
  - Returns: "object" | "value" | "fallback"
- VmKernel.resolve_method_batch(reqs_json: string) -> string
  - Input JSON: { reqs: [{class, method, arity}], context?: {...} }
  - Output JSON: { plans: [{kind, target, notes?}], errors?: [...] }

Error & Timeout
- All calls run with a per‑call deadline (NYASH_VM_NY_KERNEL_TIMEOUT_MS; default 200ms when enabled).
- On timeout/NY error, Rust VM aborts the bridge call (OFF path remains intact).

Re‑entry Guard
- A thread‑local flag prevents re‑entering the Ny Kernel from within an ongoing Ny Kernel call.
- On violation, the bridge errors immediately (Fail‑Fast).

Data Model
- Strings for structured data (JSON) across the boundary to avoid shape drift.
- Primitive returns (i64/bool/string) for simple policies.

Toggles (reserved; default OFF)
- NYASH_VM_NY_KERNEL=0|1
- NYASH_VM_NY_KERNEL_TIMEOUT_MS=200
- NYASH_VM_NY_KERNEL_TRACE=0|1

Acceptance (v0)
- With bridge OFF, behavior is unchanged on all smokes.
- With bridge ON and a stub kernel, behavior is still unchanged; logging shows calls and zero decisions.
- Bridge API documented and skeleton Ny box exists (not wired by default).

Notes
- Router batching is critical to avoid per‑call overhead.
- Keep JSON schemas tiny and versioned; include a top‑level "v" if necessary.
