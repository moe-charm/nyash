Method Arity — Fail‑Fast Policy (Built‑ins)

Overview
- Goal: eliminate “silent failures” by validating method arity for BoxCall/Method calls.
- Scope (phase‑1): built‑in boxes (StringBox, ArrayBox, MapBox) at runtime (VM). Plugins and user boxes will follow via compile‑time verifier.

Runtime Validation (VM)
- When calling a method on built‑in boxes, VM consults the static type registry:
  - resolve_typebox_by_name(type)
  - resolve_slot_by_name(type, method, arity)
  - known_arities_for(type, method) for diagnostics
- On mismatch, VM throws an InvalidInstruction with a concise message:
  - No matching method: StringBox.indexOf(2 args). Available arities: [1]
  - No matching method: ArrayBox.push(2 args). Available arities: [1]

Compile‑time Verification (wired)
- SignatureVerifierBox (Pipeline V2) validates (method, arity) right after Method extraction and before emit.
- MethodRegistryBox exposes built‑in signatures for the verifier (apps/hakorune/vm/boxes/method_registry.hako).
- Files:
  - apps/selfhost-compiler/pipeline_v2/signature_verifier_box.hako
  - apps/selfhost-compiler/pipeline_v2/pipeline.hako: Method paths call `SignatureVerifierBox.verify_from_args(...)`.

Design Notes
- Fail‑Fast: no silent fallbacks. Unknown method or wrong arity should stop with a clear error.
- Built‑ins first: arity is enforced uniformly at runtime; compile‑time verifier will broaden coverage and catch errors earlier.
- Plugins: prefer method_id + (optional) arity metadata in hako.toml. If present, verifier enforces. If absent, dev warns; prod tolerates until metadata lands.

Smokes
- arity_error_array_push_2args_vm.sh — ensures Array.push(2) errors with a clear message.

Next Steps
- Add SignatureVerifierBox (Pipeline V2) and wire it.
- Tighten using/JSON strict paths (missing module/key → error).
- Expand Registry coverage incrementally (built‑ins first, then plugins with metadata).


Updates (2025‑10‑06)
- Registry coverage extended:
  - StringBox: toString(0), stringify(0), startsWith(1), endsWith(1)
  - ArrayBox/MapBox: toString(0), stringify(0)
- Call‑side verifier (SignatureVerifierBox.verify_call_name_arity):
  - Splits at the last '.' to get method; the penultimate token is treated as the box name candidate.
  - Strict arity check applies only to known built‑ins (String|StringBox, Array|ArrayBox, Map|MapBox). Other namespaces are allowed (no false positives).
