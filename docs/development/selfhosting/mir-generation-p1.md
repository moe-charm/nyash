MIR Generation — Phase 15.7 (P1)

Scope
- Migrate minimal MIR(JSON v0) generation to Nyash boxes under selfhost/compiler/pipeline_v2/ using shared helpers in apps/selfhost/common/mir/.
- Keep Rust side as a JSON receiver; semantics unchanged.

Boxes
- Shared (common):
  - MirSchemaBox: instruction/blocks/module constructors (pure, minimal)
  - BlockBuilderBox: small CFG assemblers (const→ret, compare→branch→ret)
- Pipeline (compiler):
  - EmitMirFlow/EmitMirFlowMap: existing path (compatible)
  - Next: adapt to use MirSchemaBox/BlockBuilderBox gradually (no behavior change)

Acceptance
- quick: selfhost_emit_mir_min_rc_vm (rc-only) passes
- integration-core: parity set remains green (no behavior change)

Next Steps (P2+)
- Binop/unary via MirSchemaBox extensions
- Call/Method/Extern via SSOT slots and extern registry
- NewBox minimal (Array/String/Map)

