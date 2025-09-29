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

Using / AST merge
- NYASH_USING=0|1 (default 1): enable/disable using system (alias: NYASH_ENABLE_USING)
- NYASH_USING_STRATEGY={resolver|prelude} (alias: NYASH_USING_IMPL; fallback: NYASH_USING_AST)
  - resolver: name resolution only (no AST merge)
  - prelude: AST prelude merge (dev/ci default ON; prod default OFF)
- NYASH_ALLOW_USING_FILE=0|1 (default 0): allow file‑based using in dev convenience scenarios

Plugins
- NYASH_PLUGIN_POLICY={auto|off|force} (default auto)
  - off: disable external plugins (compat: NYASH_DISABLE_PLUGINS=1)
  - force: plugin‑only execution (compat: NYASH_PLUGIN_ONLY=1)

Alias/Using (runner)
- NYASH_ALIAS_INTERNAL_REWRITE=1 (default 1): after renaming prelude tops to `Alias_<Top>`, also rewrite internal references inside the prelude to the new names. Set to 0/false to disable (emergency toggle).

Tracing (dev only; default OFF)
- NYASH_VM_TRACE, NYASH_LOCAL_SSA_TRACE, NYASH_ROUTER_TRACE, NYASH_VARMAP_TRACE, NYASH_VM_BRANCH_TRACE

Timeout (dev scripts)
- DEV_TIMEOUT_SEC: default 60; set 0 for no timeout.

Compiler Track (dev only; default OFF)
- NYASH_COMPILER_TRACK: 1 to enable new Selfhost Compiler pipeline pieces under apps/selfhost-compiler/.
- NYASH_JSON_ONLY: 1 to print only JSON payloads (quiet mode) for acceptance checks.

Selfhost Compiler (parent→child; official, default OFF)
- NYASH_USE_NY_COMPILER=1: enable selfhost pipeline in runner (parent executes child Ny compiler program).
- NYASH_NY_COMPILER_MIN_JSON=1: child receives `-- --min-json` (emit minimal AST JSON).
- NYASH_SELFHOST_READ_TMP=1: child receives `-- --read-tmp` (read `tmp/ny_parser_input.ny`).
- NYASH_NY_COMPILER_STAGE3=1: child receives `-- --stage3` (enable Stage‑3 surface).
- NYASH_NY_COMPILER_CHILD_ARGS="...": extra args passed verbatim to child after `--` (e.g., `--emit-mir --compiler-track`).
- NYASH_NY_COMPILER_TIMEOUT_MS: child timeout in milliseconds (default 2000).
- NYASH_NY_COMPILER_EMIT_ONLY: default 1 (emit‑only); when 1, runner prints child JSON and returns handled.
- NYASH_USE_NY_COMPILER_EXE=1: prefer external compiler EXE (optional; respects `NYASH_NY_COMPILER_EXE_PATH`).
- NYASH_NY_COMPILER_SKIP_PY=1: skip Python MVP harness fallback.
