Selfhost — Hakorune Self‑Hosting (Compiler/VM/Shared)

Purpose
- Provide a top‑level home for the self‑hosting toolchain.
- Separate responsibilities from apps/ (samples/demos) for clarity and growth.

Layout
- shared/  — Cross‑cutting boxes used by both compiler and VM (e.g., MIR schema/builders, JSON cursors).
- compiler/ — Selfhost compiler pipeline (parsing → IR emit → normalization/SSA → MIR emit).
- vm/       — Mini‑VM boxes for executing MIR used in development/testing.

Boundaries (Box‑First)
- shared: no I/O side effects; provide pure helpers and schemas.
- compiler: no runtime execution; emits MIR only (Fail‑Fast at boundaries).
- vm: execution only; no parsing/emit responsibilities.

Migration
- Step 1: introduce top‑level structure (this commit) and start routing new work here.
- Step 2: gradually migrate apps/selfhost-* callers and tests to selfhost/* (module aliases or file‑path using).
- Step 3: retire legacy paths under apps/ when references are gone (smokes green).
