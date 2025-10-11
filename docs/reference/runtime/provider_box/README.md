# ProviderBox — NewBox Entry (Phase 15.7)

Purpose
- Single boundary to resolve `new <Box>` in the VM: PluginHost → Unified Registry → Embedded fallback.
- Enforces fail‑fast rules and avoids scattered ad‑hoc fallbacks.

Responsibilities
- Ensure plugin host is initialized (config/specs loaded; static specs applied)
- Try to construct via PluginHost
- If plugin returns a placeholder with `instance_id=0`, proactively call `birth()` and adopt the returned handle
- Register the created box into scope for lifecycle (fini on scope exit)

Non‑Responsibilities
- No special‑case knowledge of collections or user boxes
- No silent string/JSON parsing; errors must bubble up

Guards/Policies
- Priority: User (Nyash) > Plugin > Embedded
- Birth adoption is idempotent: if no `birth` exists, do nothing
- Fail‑fast on unknown box type when plugin‑only policy is active

References
- Implementation: `src/backend/mir_interpreter/handlers/boxes/newbox.rs`
- Loader/specs: `src/runtime/plugin_loader_v2/enabled/*`, `src/runtime/static_plugins/mod.rs`
