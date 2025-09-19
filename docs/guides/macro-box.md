# MacroBox — User Extensible Macro Units (Experimental)

Status: Prototype, behind env gates (default OFF).

Goals
- Allow users to implement procedural macro-like expansions in pure Nyash (future) and Rust (prototype) with a clean, deterministic interface.
- Preserve “Box independence” (loose coupling). Operate on public interfaces; avoid relying on private internals.

Enabling
- Enable MacroBox execution: `NYASH_MACRO_BOX=1`
- Enable built-in example: `NYASH_MACRO_BOX_EXAMPLE=1`
- Macro engine is ON by default; disable with `NYASH_MACRO_DISABLE=1`.

API (Rust, prototype)
- Trait: `trait MacroBox { fn name(&self) -> &'static str; fn expand(&self, ast: &ASTNode) -> ASTNode }`
- Register: `macro::macro_box::register(&MY_BOX)`
- Execution: Engine applies all registered boxes once per expansion pass (after built-in expansions). Order is registration order.

Constraints (philosophy)
- Deterministic; side-effect free; no IO.
- Respect Box independence: operate on public interfaces only where applicable.

Built-in example
- `UppercasePrintMacro` (guarded by `NYASH_MACRO_BOX_EXAMPLE=1`): transforms `print("UPPER:...")` to uppercase.

Future plan
- MacroBox in Nyash: `box MacroBox { static function expand(ast) -> ast }` with sandboxing and package registration via `nyash.toml`.
- Attribute-style macros: `@derive`-like hooks executed as MacroBoxes.

