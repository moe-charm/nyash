Resolver Layer (Guarded)

Responsibility
- ParserOutput → Resolved form (names/imports/types minimal).
- No code generation; no runtime execution.

Notes
- Guard-only for now; concrete logic stays where it is until migration.

Facade (Phase 2)
- Entry: `front::resolver_layer::facade::resolve_passthrough(ast::ASTNode) -> Result<ast::ASTNode, layers::FrontendError>`
- Behavior: passthrough (no-op), keeps boundary explicit
- ParserOutput/ResolverInput unify on `ASTNode` for now (migration-friendly)

I/O Examples
- Input: `ASTNode::Program { ... }` (from parser facade)
- Output: same `ASTNode` (no changes yet)
