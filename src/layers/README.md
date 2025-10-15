Layers — Interfaces and Contracts

Purpose
- Define thin, explicit interfaces between front-end layers without entangling implementation.
- Keep responsibilities crisp and testable, enabling phased migration.

Guidelines
- Parser: turns tokens into an AST-like structure (no name resolution, no codegen).
- Resolver: turns parser output into a resolved form (names/types/imports), no codegen.
- MIR: lowering only (no execution).
- Runtime: execution only (no parsing).

Where to implement
- Traits live in `interfaces.rs` in this folder.
- Concrete implementations remain in existing modules until migration is complete.

