# MIR 26-Instruction Diet (Agreed Final Set)

Goal
- Converge on a lean, proven instruction set guided by vm-stats and E2E.
- Preserve hot paths, demote meta, fold type ops, reserve room for growth.

Agreed Final Set (26)
1) Const
2) Copy
3) Load
4) Store
5) BinOp
6) UnaryOp
7) Compare
8) Jump
9) Branch
10) Phi
11) Return
12) Call
13) BoxCall
14) NewBox
15) ArrayGet
16) ArraySet
17) RefNew
18) RefGet
19) RefSet
20) Await
21) Print
22) ExternCall (keep minimal; prefer BoxCall)
23) TypeOp (unify TypeCheck/Cast)
24) WeakRef (unify WeakNew/WeakLoad)
25) Barrier (unify BarrierRead/BarrierWrite)
26) Reserve (future async/error instr)

Hot/Core (keep)
- Data: Const, Copy, Load, Store
- ALU: BinOp, UnaryOp, Compare
- Control: Jump, Branch, Phi, Return
- Calls: Call, BoxCall
- Objects: NewBox
- Arrays: ArrayGet, ArraySet

Likely Keep (usage-dependent)
- Refs: RefNew, RefGet, RefSet (seen in language features; keep unless stats prove cold)
- Async: Await (FutureNew/Set can be Box/APIs)

Meta (demote to build-mode)
- Debug, Nop, Safepoint

Type Ops (fold)
- TypeCheck, Cast → fold/verify-time or unify as a single TypeOp (optional).

External (unify)
- ExternCall → prefer BoxCall; keep ExternCall only where required.

Extended/Reserve
- Weak*: WeakNew, WeakLoad
- Barriers: BarrierRead, BarrierWrite
- 2 Reserve IDs for future async/error instrumentation

Mapping Notes
- HTTP E2E shows BoxCall/NewBox dominate (33–42%), then Const/NewBox, with Branch/Jump/Phi only in error flows.
- FileBox path similarly heavy on BoxCall/NewBox/Const.
- Implication: invest into BoxCall fast path and const/alloc optimization.

Migration Strategy
1) Introduce an experimental feature gate that switches TypeCheck/Cast to TypeOp or folds them.
2) Demote Debug/Nop/Safepoint under non-release builds.
3) Keep ExternCall available, route new external APIs via BoxCall when possible.
4) Track Weak*/Barrier usage; unify under WeakRef/Barrier. Graduate dedicated ops only if vm-stats shows recurring use.

Mapping from Current → Final
- TypeCheck, Cast → TypeOp
- WeakNew, WeakLoad → WeakRef
- BarrierRead, BarrierWrite → Barrier
- (Keep) ExternCall, but prefer BoxCall where possible（ExternCall は最小限に）
- (Keep) Debug/Nop/Safepoint as meta under non-release builds（命令セット外のビルドモード制御に降格）

Verification
- Add MIR verifier rule: use-before-def across merges is rejected (guards phi misuse).
- Add snapshot tests for classic if-merge returning phi.

Appendix: Rationale
- Smaller ISA simplifies VM fast paths and aids JIT/AOT later.
- Data shows hot paths concentrated on calls, const, alloc; control sparse and localized.
- Folding type ops reduces interpreter/VM dispatch; verification handles safety.
