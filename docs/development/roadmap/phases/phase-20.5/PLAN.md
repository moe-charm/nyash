# Phase 20.5 — Gate Plan (脱Rust 大作戦)

Status: Active
Scope: Minimal, verifiable, gate‑based execution plan

## Executive Summary (5 lines)
- Goal: Escape from Rust by bootstrapping a self‑hosting Hakorune line.
- Strategy: Freeze Rust (v1.1‑frozen) and build upward in small, verifiable gates.
- Shape: Parser → MIR → (Pure VM Foundations) with HostBridge as the only external boundary.
- Proof: Deterministic outputs, parity, and fixed‑point (v1==v2==v3) where applicable.
- Policy: Keep changes minimal, deterministic, and test‑first (SKIP when preconditions unmet).

## Gates (must pass in order)
- Gate A: Parser v1 (Hakorune) produces canonical AST JSON
  - DoD: 10 cases PASS; no non‑determinism; keys sorted
- Gate B: MIR Builder v1 (Hakorune) emits minimal MIR (16 ops)
  - DoD: 16 ops reachable; golden JSON matches; keys sorted
- Gate C: VM Foundations (Pure Hakorune) — 5 ops PoC via HostBridge
  - Ops: const, binop, compare, jump, ret
  - DoD: Simple programs run end‑to‑end; measurable performance
- Gate D: op_eq Migration (NoOperatorGuard + 8 types)
  - DoD: 20 golden tests PASS; ≥70% perf vs Rust‑VM
- Gate E: Integration & Docs
  - DoD: E2E 10 cases PASS; docs complete; CI minimal green

Note: C Code Generator track is preserved as design reference but not primary; Pure Hakorune VM is the active plan.

## Minimal Instruction Set (Phase 20.5 scope)
- Values/control: const, ret
- Arithmetic: binop(Add/Sub/Mul/Div/Mod)
- Compare: Eq/Ne/Lt/Le/Gt/Ge (boolean 0/1)
- Control: jump, branch
- Phi: pre‑compute lowering (block‑local temps)

## Determinism & DoD
- Canonicalize JSON (sorted keys), stable whitespace/newlines
- No timestamps/PIDs/randomness/hash iteration variance
- DoD per gate as listed; aggregate: Parser→MIR→VM PoC end‑to‑end

## Test Plan
- Level 1 (Sanity): 4 tests → const/ret/binop/compare
- Level 2 (Coverage): 10 tests → if/loops/arrays/strings/minimal recursion
- Level 3 (PoC): 1 test → VM 5‑ops program runs (Pure Hakorune)
- Policy: preconditions unmet → SKIP（WARN）; only regressions are FAIL

## CI & Packaging (minimal)
- CI minimal: cargo build --release + quick‑selfhost (SKIP前提で緑) + make release(manifest)
- Windows builds: manual MSVC preferred; MinGW as reference
- Dist: Linux + Windows(MSVC/MinGW) with README_FROZEN_QUICKSTART and release notes

## Risks & Mitigations (short)
- v1 != v2 drift → deterministic JSON + canonical emit + golden tests
- Performance slow → accept PoC cost; profile hot paths; ≤30s budget for self‑compile later
- Box不足 → pre‑surveyed minimal set; add only under gates

## Next Steps (actionable)
1) Lock Gate A/B test lists; add golden fixtures（json sorted）
2) Implement VM PoC (5 ops) via HostBridge（apps/hakorune‑vm）
3) Migrate op_eq with NoOperatorGuard + 20 golden tests
4) Turn SKIP gates into PASS progressively（quick‑selfhost → plugins → integration）
5) Update MILESTONE/INDEX weekly; keep artifacts/manifest fresh

