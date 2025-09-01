# Phase 11.7 – JIT Complete (MIR-15 + Semantics + Sync/Async GC)

Scope: Implement a production‑ready JIT backend for Nyash that fully supports the MIR Core‑15 instruction set, synchronous/async GC cooperation, and delivers a simple, reproducible developer experience across platforms (WSL/Linux/Windows).

Why now:
- LLVM AOT は一度実装を進めたが、Windows 依存が重くサイズも大きい。Cranelift に回帰して、純Rustで“Just Works”のDXを取り戻し、高速な反復開発を実現する（AOT/LLVMは資料として保持）。

Outcomes (Definition of Done):
- All MIR‑15 ops compile and execute via JIT with behavioral parity to VM.
- BoxCall/ExternCall are handled through NyRT shims (handle‑first ABI) safely.
- Sync GC barriers in place (read/write), async safepoints wired at call/loop edges.
- Smokes: echo/array/map/vinvoke/extern pass; parity checks vs VM/JIT (logs included).
- 1‑command setup and run on WSL + Windows Dev PowerShell; no external LLVM needed.

Backends Strategy:
- LLVM AOT はアーカイブ（参照は可）。主線は Cranelift（JIT/軽量AOT）。
- JIT 既定は Cranelift（feature: `cranelift-jit`）。AOT は必要に応じ `cranelift-object` を併用。

This folder contains the living plan (PLAN.md) and the rolling snapshot of the current task focus (CURRENT_TASK.md). Semantics 層の導入により、Nyash スクリプト／VM／JIT（exe）の動作を一致させる。
