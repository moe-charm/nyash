Unified Call — MirCall and Callee

Scope
- Define the single, unified call instruction for MIR and its typed target (`Callee`).
- Provide mapping from legacy instructions and invariants for builders and backends.

Canonical Source
- Code: src/mir/definitions/call_unified.rs:22, 86, 150
- Printer support: src/mir/printer_helpers.rs:79
- Inference helpers: src/mir/builder/infer/receiver.rs

MirCall
- Fields
  - dst: Option<ValueId>
  - callee: Callee
  - args: Vec<ValueId>
  - flags: CallFlags (tail_call/no_return/can_inline/is_constructor)
  - effects: EffectMask

Callee variants
- Global(String)   // builtins only
- Extern(String)   // host/import functions (e.g., env.console.log)
- ModuleFunction(String) // module/user functions (e.g., "Counter.inc/0")
- Method { box_name: String, method: String, receiver: Option<ValueId>, certainty: TypeCertainty }
- Constructor { box_type: String }
- Closure { params: Vec<String>, captures: Vec<(String, ValueId)>, me_capture: Option<ValueId> }
- Value(ValueId)

Invariants
- Receiver is not part of args for Method; it lives in Callee::Method.receiver.
- Constructor/Closure must be emitted with flags.is_constructor=true and effects containing Alloc.
- Effects:
  - External calls include Io by default.
  - Pure internal calls default to PURE unless known side effects are modeled.
- Materialization (LocalSSA):
  - At call sites, values (recv/args) should be in-block materialized before call for VM/LLVM parity.

Legacy → MirCall mapping
- Call (func=const "name") → MirCall::global(name)
- BoxCall (box_val, method[, id], args) → MirCall::method(receiver=box_val, method)
- ExternCall (iface, method) → MirCall::external("iface.method")
- NewBox (box_type, args) → MirCall::constructor(box_type)
- NewClosure (params, captures, me) → MirCall::closure(...)
- PluginInvoke → MirCall::method (policy/effects capture the plugin path)
 - Call (func=const "Class.method/N") → MirCall::module_function("Class.method/N")

Builder policy
- Prefer emitting MirCall directly.
- When lowering legacy forms for compatibility, immediately migrate to MirCall via `migration::*` helpers.

Backend policy
- VM/LLVM/PyVM dispatch on Callee only.
- Extern(String) maps to host ABI/import layer.
- ModuleFunction(String) resolves via function table (exact name; arity in suffix).

Diagnostics
- Use `NYASH_OPT_DIAG=1` to report legacy instructions.
- `NYASH_OPT_DIAG_FORBID_LEGACY=1` to fail builds/tests on legacy forms.

VM adapter (sketch)
- BoxCall → Callee::Method { box_name=infer(box_val), method, receiver=Some(box_val) }
- NewBox → Callee::Constructor { box_type }
- ExternCall → Callee::Extern("iface.method")
- NewClosure → Callee::Closure { .. }
- Call(func=NameConst("Class.method/N")) → Callee::ModuleFunction("Class.method/N")

Flags
- `NYASH_MIR_UNIFIED_CALL=1`
  - Enable unified call emission for known builtins/externs in builder and printer.
- `NYASH_MIR_CALL_MODULE_FN=1`
  - Builder emits `Callee::ModuleFunction` for module/user functions when resolvable.
- `NYASH_MIR_CALL_MODULE_FN_STRICT=1`
  - Ambiguous tail matches are fail-fast with a detailed candidate list. When 0, a heuristic may prefer current box.
- `NYASH_MIR_CALL_MODULE_FN_CANON=1`
  - Only accept canonical dotted-with-arity names as module functions (exact match), for phased rollout.
- `NYASH_VM_CALL_ADAPTER=1`
  - VM routes legacy BoxCall/ExternCall/NewBox through a thin adapter that constructs `Callee` then dispatches.
- `NYASH_WARN_LEGACY_CALL=1`
  - Print a dev-only warn/JSON line whenever a legacy call-like instruction is executed/emitted.
- `NYASH_JSON_SCHEMA_V1=1` / `NYASH_JSON_SCHEMA_V0=1`
  - Select MIR JSON schema. v1 emits unified `{"op":"mir_call", ...}` entries with Callee payloads; v0 uses legacy separate ops.
