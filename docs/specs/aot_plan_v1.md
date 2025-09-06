# AOT-Plan v1 Schema (Phase 15.1)

Status: draft-frozen for Phase 15.1 (extensions via `extensions` object only)

- version: string, must be "1"
- name: optional plan/module name
- functions: array of PlanFunction
- externs: optional array of extern identifiers (reserved; not required in 15.1)
- exports: optional array of export names (reserved)
- units: optional array of link units (reserved)
- extensions: optional object for forward-compatible keys

PlanFunction:
- name: string
- params: array of { name: string, type?: string } (informational in 15.1)
- return_type: optional string; one of: integer, float, bool, string, void (or omitted → Unknown)
- body: optional object with tagged `kind`
  - kind = "const_return": { value: any-json (int/bool/float/string supported) }
  - kind = "empty": returns default 0 with Unknown type (Phase 15.1 importer behavior)

Notes:
- 15.1 importer does not emit object code; it constructs MIR13 skeletons only.
- If `return_type` is omitted, importer uses Unknown to keep VM dynamic display.
- `extensions` is a free-form map; the importer ignores unknown keys.

Example:
```
{
  "version": "1",
  "name": "mini_project",
  "functions": [
    { "name": "main", "return_type": "integer", "body": { "kind": "const_return", "value": 42 }},
    { "name": "greet", "return_type": "string",  "body": { "kind": "const_return", "value": "hi" }}
  ]
}
```

