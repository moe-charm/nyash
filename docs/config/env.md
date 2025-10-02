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
  - force: plugin-only execution (compat: NYASH_PLUGIN_ONLY=1)

Syntax sugar (default ON)
- NYASH_SYNTAX_SUGAR_LEVEL={basic|full}（unset=ON）。
  - basic: pipeline/raw/numeric separators/trailing comma など基本糖衣
  - full: 将来拡張用（現時点は basic と同等か superset）
- Deprecated（互換のみ; 将来削除予定）
  - NYASH_ENABLE_ARRAY_LITERAL / NYASH_ENABLE_MAP_LITERAL / NYASH_ENABLE_MAP_IDENT_KEY
  - これらは SYNTAX_SUGAR_LEVEL に統合。verbose時に非推奨ログを出力。

Flow（stateless namespace）
- Default ON（disable with NYASH_ENABLE_FLOW=0|false|off）。
- Lowering: `Name.method(a,b)` → `Name.method/2`（グローバル関数）。BoxCallなし。

Plugin ABI (Final; experimental, default OFF)
- NYASH_PLUGIN_ABI_FINAL=1: prefer NyValue/NyResult Final ABI when available (fallback to v2)
- NYASH_PLUGIN_META=1: log presence of Final ABI/meta (quiet when absent)
- NYASH_PLUGIN_CAPS_ENFORCE=1: enforce required_capabilities at load (dev/ci recommended)
- NYASH_TRACE_EFFECTS=1: emit JSON lines for method effects (dev only)
- NYASH_CHECK_CONTRACTS=1: trace pre/post contracts (log only)

VM plugin routing (Phase C — default OFF)
- NYASH_VM_BOXCALL_PLUGIN_FIRST=1: route BoxCall to PluginInvoke when receiver is plugin-backed
- NYASH_VM_PLUGIN_PREFER_STRING=1: prefer plugin provider for StringBox
- NYASH_VM_PLUGIN_PREFER_ARRAY=1: prefer plugin provider for ArrayBox
- NYASH_VM_PLUGIN_PREFER_MAP=1: prefer plugin provider for MapBox
  Notes: runner merges these into NYASH_PLUGIN_OVERRIDE_TYPES and rebuilds the unified registry.

Plugin ABI (Final Vision; dev/experimental)
- NYASH_PLUGIN_ABI_FINAL=1: prefer NyResult-based invoke and enable Final ABI probes (fallback to v2 when unavailable)
- NYASH_PLUGIN_META=1: query and log plugin meta (get_method_meta/get_all_methods/get_type_info) when present
- NYASH_PLUGIN_CAPS_ENFORCE=1: enforce required_capabilities at load time (dev/ci only recommended)
- NYASH_TRACE_EFFECTS=1: emit JSON lines for declared method effects at call time
- NYASH_CHECK_CONTRACTS=1: trace pre/post contracts (log-only; no hard enforcement yet)

Alias/Using (runner)
- NYASH_ALIAS_INTERNAL_REWRITE=1 (default 1): after renaming prelude tops to `Alias_<Top>`, also rewrite internal references inside the prelude to the new names. Set to 0/false to disable (emergency toggle).

Tracing (dev only; default OFF)
- NYASH_VM_TRACE, NYASH_LOCAL_SSA_TRACE, NYASH_ROUTER_TRACE, NYASH_VARMAP_TRACE, NYASH_VM_BRANCH_TRACE

Timeout (dev scripts)
- DEV_TIMEOUT_SEC: default 60; set 0 for no timeout.

Compiler Track (dev only; default OFF)
- NYASH_COMPILER_TRACK: 1 to enable new Selfhost Compiler pipeline pieces under apps/selfhost-compiler/.
- NYASH_JSON_ONLY: 1 to print only JSON payloads (quiet mode) for acceptance checks.
- NYASH_QUIET: 1 to suppress non-essential logs even when verbose switches would normally print.
  - Runner and subsystems honor quiet mode to avoid polluting child JSON output.

Selfhost Compiler (parent→child; official, default OFF)
- NYASH_USE_NY_COMPILER=1: enable selfhost pipeline in runner (parent executes child Ny compiler program).
- NYASH_NY_COMPILER_MIN_JSON=1: child receives `-- --min-json` (emit minimal AST JSON).
- NYASH_SELFHOST_READ_TMP=1: child receives `-- --read-tmp` (read `tmp/ny_parser_input.ny`).
- NYASH_NY_COMPILER_STAGE3=1: child receives `-- --stage3` (enable Stage‑3 surface).
- NYASH_NY_COMPILER_CHILD_ARGS="...": extra args passed verbatim to child after `--` (e.g., `--emit-mir --compiler-track`).
- NYASH_NY_COMPILER_TIMEOUT_MS: child timeout in milliseconds (default 2000).
- NYASH_NY_COMPILER_EMIT_ONLY: default 1 (emit‑only); when 1, runner prints child JSON and returns handled.
- NYASH_USE_NY_COMPILER_EXE=1: prefer external compiler EXE (optional; respects `NYASH_NY_COMPILER_EXE_PATH`).
- NYASH_NY_COMPILER_SKIP_PY=1: skip Python MVP harness fallback（deprecated; PyVM is withdrawn by default）.
