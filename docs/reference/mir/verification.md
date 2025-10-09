MIR Verification — Core Checks (Phase 19)

Overview
- Purpose: fail fast on MIR shapes that are known-invalid or semantically ambiguous across backends.
- Scope: SSA/dominance/CFG/barrier checks, legacy forbiddance, and semantic bans that must be fixed in builders.

Key Checks
- Box Compare forbidden:
  - Forbid `Compare(Eq|Ne)` when either operand is a Box. Equality must be normalized at MIR to a call:
    - Preferred: `lhs.equals/1(rhs)`
    - Universal: `Call{ callee=Extern("nyrt.ops.op_eq") }`

- Static self fields forbidden:
  - Forbid `BoxCall { method in {getField,setField}, box_val = Const(String(..)) }`.
  - Rationale: static box is a namespace. `me.field` read/write is not supported (no runtime state).
  - Builders must fail early on `me.field`/`me.field =` inside static boxes. The verifier is the safety net.

Notes
- ExternCall is retired. Builders and normalizers must use `Call{ callee=Extern(..) }` for externs.
- LLVM harness inlines `nyrt.ops.op_eq` (i64 icmp → zext) for parity without requiring a C kernel symbol.

