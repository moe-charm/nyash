UsingResolverBox — Pre‑Resolve Using Declarations

Scope
- Manage alias→path and alias→namespace name maps for compiler pipeline (P2‑A).
- Consume lightweight JSON emitted by parser (UsingCollectorBox or ParserBox.add_using).
- Provide stable getters for later NamespaceBox (P2‑B) and Emit boxes.

Responsibilities
- Input forms:
  - Path using: [{"name":"Alias","path":"apps/..../file.hako"}, ...]
  - Namespace using: [{"name":"selfhost.vm.mir_min"}, ...] or [{"name":"Alias"}] when ParserBox.add_using(ns, alias) is used.
- Output/context:
  - alias_paths: Alias → file path
  - alias_names: Alias → namespace name (best effort; Alias itself when only ns is known)
  - modules_map: NsName → file path (provided by caller; no file IO here)

Non‑Responsibilities
- Reading hako.toml or filesystem.
- Runtime using resolution. This is compiler‑only pre‑resolution.

API (box methods)
- load_usings_json(usings_json)
- load_modules_json(modules_json)
- add_ns(alias, ns_name) / add_module(ns_name, path) / add_path(alias, path)
- resolve_path_alias(alias) -> path | null
- resolve_namespace_alias(alias) -> ns_name | null
- resolve_module_path_from_alias(alias) -> path | null
- to_context_json() -> {alias_paths,namespaces,modules}

Notes
- When UsingCollectorBox is used, namespace entries contain the full name in "name" and no alias. In that case alias_names[alias] will use the same string; callers may still override via add_ns(alias, ns).
- Keep this box pure and side‑effect free for easy testing.

