# MIR JSON v0 Schema (SSOT)

Goal: Single Source of Truth for MIR(JSON) exchanged between emitters (Rust/Hakorune compiler) and consumers (Rust VM, Hakorune‑VM, LLVM harness/ny‑llvmc).

Header (required)
- kind: "MIR"
- schema_version: "1.0"

Top level
- functions: array of function objects

Function
- name: string
- params: array (u32 ids or omitted)
- blocks: array of BasicBlock (preferred) or
- instructions: array of Instruction (when blocks omitted; treated as block 0)

BasicBlock
- id: u32
- instructions: Instruction[]

Instruction (selected)
- const: { op:"const", dst:u32, value:{ type:"i64|f64|bool|string|null|void|handle", value: T, [box_type: String] } }
- binop: { op:"binop", dst:u32, lhs:u32, rhs:u32, operation:"+|-|*|/|%|&|||^|<<|>>" [, dst_type:{kind:"handle", box_type:"StringBox"}] }
- compare: { op:"compare", dst:u32, lhs:u32, rhs:u32, cmp:"Eq|Ne|Lt|Le|Gt|Ge" | operation:"==|!=|<|<=|>|>=" }
- branch: { op:"branch", cond:u32, then:u32, else:u32 }
- jump:   { op:"jump", target:u32 }
- ret:    { op:"ret" | "return", value:u32|null }
- phi:    { op:"phi", dst:u32, values:[{block:u32, value:u32}], [dst_type:{kind:"handle", box_type:"StringBox"}] }
- copy:   { op:"copy", dst:u32, src:u32 }
- mir_call (v1 only): { op:"mir_call", dst:u32|null, mir_call:{ callee:{ type:"Extern|Global|ModuleFunction|Method|Constructor|Closure|Value", ... }, args:[u32], effects:["IO"|...], flags:{} } }

Notes
- v0 accepts functions[].instructions (implicit block 0) or blocks[].instructions.
- For booleans, emit as {type:"i64", value:0|1} or keep dedicated "bool"; readers normalize.
- Strings as handles: {type:{kind:"handle", box_type:"StringBox"}, value:"…"}.
- Readers must Fail‑Fast when kind != "MIR" or schema_version is present and != "1.0".
