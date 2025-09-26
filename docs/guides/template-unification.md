# Template → Macro Unification (Breaking Change)

Status: Active. Templates are now specified and executed via the Macro Engine. There is no separate “template pipeline”. This simplifies expansion order, diagnostics and tooling. The `--expand` flow shows both macro and template (pattern/unquote) transformations.

What changed
- Template constructs are expressed using the macro Pattern/Quote API.
- `$name` and `$...name` (variadic) placeholders are part of macro templates.
- Arrays/Maps are supported in both matching and unquote.
- There is no legacy, template-only expansion pass.

Authoring
- Programmatic API (Rust): `crate::macro::pattern::{TemplatePattern, OrPattern, AstBuilder}`.
- Textual quote is a convenience helper: `AstBuilder::quote("code here")` and unquote with variable bindings.
- Variadics:
  - `$...tail` can appear at any position in call/array arguments.
  - During match: the variable-length segment is bound to an `ASTNode::Program`.
  - During unquote: the `Program`’s statements are spliced into the argument list.

Tooling
- `nyash --expand --dump-ast file.nyash` shows pre/post expansion.
- Macro gate: `NYASH_MACRO_ENABLE=1`.

Compatibility
- This is a breaking change. Existing “template-only” extension points must adopt the macro engine.
- Guidance: treat templates as macro sugar; move any custom template processors to macro patterns.

