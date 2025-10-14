Parser Layer (Guarded)

Responsibility
- Lexed tokens → AST-like structure.
- No name resolution; no code generation; no runtime assumptions.

Notes
- This folder carries guard metadata only (no code compiled yet).
- Concrete parser lives in `src/parser.rs` and will be migrated gradually.

Facade (Phase 2)
- Entry: `front::parser_layer::facade::parse_source_to_ast(&str) -> Result<ast::ASTNode, layers::FrontendError>`
- Behavior: delegates to existing `parser::NyashParser` (no behavior change)
- Dev flag to opt-in via runner: `HAKO_FRONT_USE_FACADE=1`

I/O Examples
- Input: `"static box Main { main() { print(\"OK\"); return 0 } }"`
- Output: `ast::ASTNode::Program { statements: [...], span: ... }`
- Type: `ASTNode` implements `layers::ParserOutput`/`ResolverInput`
