Quick/LLVM — lightweight harness checks

Purpose
- Provide a very small set of LLVM/llvmlite harness smokes for quick profile.
- Focus on IR dump / trace visibility and minimal parity checks.

Conventions
- Skips automatically when LLVM is not detected (handled by run.sh).
- Keep each test short (< 1s) and self-contained.

Examples
- modulefn_llvm_trace.sh — verify ModuleFunction appears in LLVM trace.
- parity_m2_*_vm_llvm.sh — minimal VM↔LLVM output parity.

