# Block‑Postfix Catch Language Design (Draft)

Goal
- Make exception boundaries explicit by attaching `catch`/`cleanup` to standalone blocks.

Design
- Syntax: `{ body } catch (e) { handler } [cleanup { … }]` and `{ body } cleanup { … }`.
- Policy: single catch (branch inside the catch); scope limited to the same block; no implicit propagation.
- Static check (MVP): direct `throw` in a standalone block requires an immediate postfix `catch`.

Implementation Notes
- Parser (gated): normalize postfix to `ASTNode::TryCatch`.
- Bridge(Result‑mode): ThrowCtx routes nested `throw` to the single catch; merge with PHI‑off.
- Friendly errors: disallow top‑level leading `catch`/`cleanup`, and attaching to structural if/loop blocks.

References
- Parser: `src/parser/statements.rs`
- Smokes: `src/tests/parser_block_postfix_{catch,errors}.rs`

Open Questions
- Multiple catch with type hierarchy; effects typing for static checks; formatter support.
