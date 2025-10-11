Shared MIR helpers (schema/builders)

Purpose
- Provide a single, reusable set of helpers to construct MIR(JSON v0) structures from Nyash code.
- Keep the JSON shape consistent between the selfhost compiler and the selfhost VM tools.

Boxes
- MirSchemaBox: constants and instruction/object constructors (pure)
- BlockBuilderBox: tiny helpers to assemble blocks and simple CFGs (P1 scope)

Design
- Pure functions only; no global state. Return Nyash map/array values that match the runtime JSON v0.
- Instruction constructors accept primitive values or small maps; the schema is validated by the consumer.

JSON v0 (minimum used here)
- Instruction: { op: "const"|"ret"|"binop"|"compare"|"branch"|"jump", ... }
- Block: { id: Int, instructions: Array }
- Module: { version: 0, kind: "MIR", functions: [ { name: "main", blocks: [...] } ] }

