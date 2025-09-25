# Nyash vs Other Languages — Feature Comparison

Perspective
- Nyash emphasizes a unified Box model, hygienic AST macros with sandboxing, and multi‑backend execution (PyVM/LLVM/Cranelift/WASM).

Axes
- Control Flow & SSA
  - Nyash: explicit observability (hints), PHI hygiene invariants.
  - Rust/Swift/Kotlin: SSA is internal; limited direct observability.
- Exceptions
  - Nyash: postfix `catch/cleanup` (scope‑first), zero‑cost lowering.
  - Rust: Result/? idiom (no exceptions). Swift/Kotlin: try/catch/finally.
- Macros
  - Nyash: AST JSON v0, hygienic by construction, isolated child with capabilities.
  - C: text macro. Rust: macro_rules!/proc‑macros. Lisp/Julia: homoiconic AST.
- Scope
  - Nyash: ScopeBox (compile‑time metadata) and ScopeEnter/Leave hints; disappears at runtime.
  - Go/Rust/Swift: lexical scopes (no explicit observability layer).
- Backends
  - Nyash: PyVM (reference semantics), LLVM (AOT), Cranelift (JIT), WASM.
  - Others: single backend or VM.

Takeaways
- Nyash’s differentiator is “observability without cost” and macro safety by default.
- Where trade‑offs exist (e.g., temporary hygiene valves), they’re gated and documented.

