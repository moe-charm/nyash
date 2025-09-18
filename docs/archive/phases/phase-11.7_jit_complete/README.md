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

## JIT Single-Exit Policy and TRACE

- Single-Exit: JIT は関数終端で単一の ret ブロックに合流する方針。分岐合流は BlockParam（最小PHI）で表現し、`end_function` で最終 seal を行う。
- Branch Fast-Path: then/else がともに i64 定数を即時 return する場合、`select(cond, K_then, K_else)` → `return` に縮約（常時有効）。
- TRACE 環境変数（必要時のみON）:
  - `NYASH_JIT_DUMP=1` …… Lower の要約/CFGライトダンプを表示
  - `NYASH_JIT_TRACE_BLOCKS=1` … ブロック入場ログ（`[JIT-BLOCK] enter=<idx>`）
  - `NYASH_JIT_TRACE_BR=1` …… br_if の cond 有無ログ
  - `NYASH_JIT_TRACE_SEL=1` … select の cond/then/else 値（tag=100/101/102）
  - `NYASH_JIT_TRACE_RET=1` … return 値ログ（tag=201=直前, 200=合流）

Notes:
- 旧フラグ `NYASH_JIT_FASTPATH_SELECT` は不要になりました（存在しても無視）。
