# AST JSON v0 (Macro Expansion)

Status: Draft. This document specifies a minimal JSON schema for representing Nyash AST to enable macro expansion by external processes (e.g., PyVM-based MacroBox).

Top-level
- Object with `kind` discriminator.
- Nested nodes referenced inline; no IDs.
- Span is omitted in v0 (unknown). Future versions may include `span` with file/line/col.

Kinds (subset for Phase 2+)
- Program: { kind: "Program", statements: [Node] }
- Loop: { kind: "Loop", condition: Node, body: [Node] }
- Print: { kind: "Print", expression: Node }
- Return: { kind: "Return", value: Node|null }
- Break: { kind: "Break" }
- Continue: { kind: "Continue" }
- Assignment: { kind: "Assignment", target: Node, value: Node }
- If: { kind: "If", condition: Node, then: [Node], else: [Node]|null }
- FunctionDeclaration: { kind: "FunctionDeclaration", name: string, params: [string], body: [Node], static: bool, override: bool }
- Variable: { kind: "Variable", name: string }
- Literal: { kind: "Literal", value: LiteralValue }
- BinaryOp: { kind: "BinaryOp", op: string, left: Node, right: Node }
- UnaryOp: { kind: "UnaryOp", op: string, operand: Node }
- MethodCall: { kind: "MethodCall", object: Node, method: string, arguments: [Node] }
- FunctionCall: { kind: "FunctionCall", name: string, arguments: [Node] }
- Array: { kind: "Array", elements: [Node] }
- Map: { kind: "Map", entries: [{k: string, v: Node}] }
- Local: { kind: "Local", variables: [string], inits: [Node|null] }

LiteralValue
- { type: "string", value: string }
- { type: "int", value: integer }
- { type: "float", value: number }
- { type: "bool", value: boolean }
- { type: "null" }
- { type: "void" }

Unary operators
- "-" for Minus, "not" for Not

Binary operators
- "+", "-", "*", "/", "%", "&", "|", "^", "<<", ">>", "==", "!=", "<", ">", "<=", ">=", "&&", "||"

Notes
- The schema is intentionally minimal; it covers nodes needed for Phase 2 samples.
- Future: add `span`, `attrs`, typed annotations as needed.
- Type checks (is/as) mapping
  - AST JSON v0 does not introduce a dedicated TypeOp node. Instead, write MethodCall with
    method "is" or "as" and a single string literal type argument:
    {"kind":"MethodCall","object":<expr>,"method":"is","arguments":[{"kind":"Literal","value":{"type":"string","value":"Integer"}}]}
  - Lowering maps this to MIR::TypeOp(Check/ Cast) with the target type resolved by name.

## Example

Source:
```
print("x")
```

Expanded AST JSON v0:
```
{"kind":"Program","statements":[{"kind":"Print","expression":{"kind":"Literal","value":{"type":"string","value":"x"}}}]}
```
