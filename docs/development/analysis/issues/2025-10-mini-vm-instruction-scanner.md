# Mini‑VM instruction scanner bug (2025‑10)

Status: open (root cause isolated — fix via box modularization)

Summary
- Symptom: Mini‑VM (selfhost/vm/boxes/mir_vm_min.hako) executes the first `const` in a block but fails to advance to the second object (const/compare/ret). The loop ends with `moved==0`, returning 0 even for Eq true cases.
- Repro: `SMOKES_DEV_LOG=1 tools/smokes/v2/profiles/quick/selfhost/selfhost_mir_m2_eq_true_vm.sh`
- Logs: `[mirvm] inst_seg.head= {"op":"const"} , {"op":"const"} , {"op":"compare"} , {"op":"ret"}` then early end.

Root cause (analysis)
- The block object scanning logic mixes responsibilities and advances the pointer inconsistently (`p/pp`), sometimes turning invalid between iterations. Delimiter and brace depth handling are coupled to extraction logic, making off‑by‑one easy.

Plan (Box‑First)
- InstructionScannerBox (new):
  - API: `normalize_delimiters(seg)`, `next(seg,pos) -> {start,end,op}|null`
  - Invariants: pos strictly advances; brace‑depth aware; delimiter‑tolerant; dev trace on failure.
- OpHandlers (new):
  - API: `handle_const/compare/ret(seg, regs, …)` with tolerant key readers (escaped/unescaped, whitespace).
- mir_vm_min.hako: reduce to a minimal loop delegating to scanner/handlers.

Dev toggles
- `SMOKES_DEV_LOG=1` — dump smoke raw output (keeps [mirvm] logs).

Acceptance
- quick/selfhost: Eq true/false return 1/0 respectively.
- (optional) LLVM harness MIR direct smoke parity (SKIP when unavailable).

