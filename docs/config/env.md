Nyash — Environment Variables (Dev vs Prod Alignment)

Truthiness & Calls
- NYASH_REWRITE_KNOWN_DEFAULT: default 1 (ON). Instance→Function rewrite enabled.
- NYASH_OPERATOR_BOX_COMPARE_ADOPT: default 1 (ON). Compare adopts OperatorBox.
- NYASH_OPERATOR_BOX_ADD_ADOPT: default 0 (OFF). String concat uses VM fast‑path; enable only when needed.

VM Runtime (dev‑only toggles; default OFF)
- NYASH_VM_PARSERBOX_BOOL: treat ParserBox BoxRef as bool via gpos. Default 0. Use only for bring‑up.
- NYASH_VM_STRLIKE_INSTANCE_COERCE: coerce InstanceBox receiver to string for substring/indexOf/lastIndexOf in ParserBox.*. Default 0.
- NYASH_VM_TOLERATE_VOID: substitute Void for undefined registers (dev safety). Default 0.
- NYASH_VM_USER_INSTANCE_BOXCALL: allow user Instance BoxCall at runtime (dev/ci only). Default 1 in dev scripts, 0 in prod.

Using/AST (dev only)
- NYASH_ENABLE_USING, NYASH_ALLOW_USING_FILE, NYASH_USING_AST: default 0. Dev scripts may set to 1.

Tracing (dev only; default OFF)
- NYASH_VM_TRACE, NYASH_LOCAL_SSA_TRACE, NYASH_ROUTER_TRACE, NYASH_VARMAP_TRACE, NYASH_VM_BRANCH_TRACE

Timeout (dev scripts)
- DEV_TIMEOUT_SEC: default 60; set 0 for no timeout.
