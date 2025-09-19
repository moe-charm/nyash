# User Macros (MacroBoxSpec) — Phase 2

Status: PoC complete; PyVM sandbox route wired. This guide explains how to author and run user macros in Nyash.

## Quickstart

- Register user macros (recommended minimal env):
  - `NYASH_MACRO_ENABLE=1`
  - `NYASH_MACRO_PATHS=apps/macros/examples/echo_macro.nyash`
- Run your program as usual (macro expansion happens once before MIR):
  - `./target/release/nyash --backend vm apps/tests/ternary_basic.nyash`

Environment overview (recommended minimal set)
- `NYASH_MACRO_ENABLE=1`（既定ON）
- `NYASH_MACRO_PATHS=...`（カンマ区切りのNyashマクロファイル）
- `NYASH_MACRO_STRICT=1`（既定: 厳格）
- `NYASH_MACRO_TRACE=0|1`（開発用トレース）

Backward compat (deprecated)
- `NYASH_MACRO_BOX_NY=1` + `NYASH_MACRO_BOX_NY_PATHS=...` → 今後は `NYASH_MACRO_PATHS` を使ってね

## Philosophy
- Hybrid approach: built-in (Rust) for minimal/core derives; user macros in Nyash (PyVM) for flexibility.
- Deterministic, sandboxed execution: default denies IO/NET/ENV.
- Unified interface: AST JSON v0 + MacroCtx. Expansion occurs pre-MIR.

MacroCtx (MVP)
- Rust側に最小の `MacroCtx` と `MacroCaps` を用意（将来のAPI統合のため）。
- フィールド/メソッド（MVP）:
  - `MacroCtx::from_env()` → 環境からcapabilitiesを組み立て
  - `ctx.gensym(prefix)` → 衛生識別子生成
  - `ctx.report(level, message)` → 開発用レポート（標準エラー）
  - `ctx.get_env(key)` → 環境取得（`NYASH_MACRO_CAP_ENV=1` のときのみ）
- 実行契約（PoC）：ランナーは `expand(json, ctx)` を優先し、失敗した場合は `expand(json)` にフォールバックします（後方互換）。

## Authoring a Macro

Create a Nyash file that defines `MacroBoxSpec` with a static `expand(json[, ctx])` returning a JSON string (AST JSON v0):

```nyash
static box MacroBoxSpec {
  static function expand(json, ctx) {
    // json: string (AST JSON v0)
    // ctx:  string (JSON; {caps:{io,net,env}} MVP). Optional for backward compatibility.
    // return: string (AST JSON v0)
    return json  // identity for MVP
  }
}
```

Example (repo): `apps/macros/examples/echo_macro.nyash`.

Editing template (string literal uppercasing)
- Example: `apps/macros/examples/upper_string_macro.nyash`
- Behavior: if a string literal value starts with `UPPER:`, the suffix is uppercased.
  - Input: `print("UPPER:hello")` → Output: `print("HELLO")`

## Running your Macro

Register and run via env (simple):

```bash
export NYASH_MACRO_ENABLE=1
export NYASH_MACRO_PATHS=apps/macros/examples/echo_macro.nyash

# Run your program (macro expansion happens before MIR)
./target/release/nyash --backend vm apps/tests/ternary_basic.nyash
```

Self‑host path（NYASH_USE_NY_COMPILER=1）での前展開（開発用）

```bash
NYASH_USE_NY_COMPILER=1 \
NYASH_MACRO_SELFHOST_PRE_EXPAND=1 \
NYASH_VM_USE_PY=1 \
./target/release/nyash --backend vm apps/tests/ternary_basic.nyash
```

Notes: 現状は PyVM ルートのみ対応。`NYASH_VM_USE_PY=1` が必須。

CLI プロファイル（推奨）
- `--profile dev`（既定相当: マクロON/厳格ON）
- `--profile lite`（マクロOFFの軽量モード）
- `--profile ci|strict`（マクロON/厳格ON）
  - 例: `./target/release/nyash --profile dev --backend vm apps/tests/ternary_basic.nyash`

Notes
- Built-in child route (stdin JSON -> stdout JSON) remains available when `NYASH_MACRO_BOX_CHILD_RUNNER=0`.
- Strict mode: `NYASH_MACRO_STRICT=1` (default) fails build on macro child error/timeout; set `0` to fallback to identity.
- Timeout: `NYASH_NY_COMPILER_TIMEOUT_MS` (default `2000`).

Testing
- Smoke: `tools/test/smoke/macro/macro_child_runner_identity_smoke.sh`
- Golden (identity): `tools/test/golden/macro/identity_user_macro_golden.sh`
- Golden (upper string): `tools/test/golden/macro/upper_string_user_macro_golden.sh`
 - Golden (array prepend 0): `tools/test/golden/macro/array_prepend_zero_user_macro_golden.sh`
 - Golden (map insert tag): `tools/test/golden/macro/map_insert_tag_user_macro_golden.sh`
 - Negative (timeout strict fail): `tools/test/smoke/macro/macro_user_timeout_strict_fail.sh`
 - Negative (invalid JSON strict fail): `tools/test/smoke/macro/macro_user_invalid_json_strict_fail.sh`
 - Negative (invalid JSON non‑strict identity): `tools/test/smoke/macro/macro_user_invalid_json_nonstrict_identity.sh`

Array/Map editing examples
- Array prepend zero: `apps/macros/examples/array_prepend_zero_macro.nyash`
  - Transforms every `{"kind":"Array","elements":[...]}` into one with a leading `0` literal element.
  - Example input: `print([1, 2])` → Expanded: elements `[0, 1, 2]`.
- Map insert tag: `apps/macros/examples/map_insert_tag_macro.nyash`
  - Transforms every `{"kind":"Map","entries":[...]}` by inserting the first entry `{k:"__macro", v: "on"}`.
  - Example input: `print({"a": 1})` → Expanded entries: `[{"__macro":"on"}, {"a":1}]`.

## Inspect Expanded AST

```bash
./target/release/nyash --dump-expanded-ast-json apps/tests/ternary_basic.nyash
```

Outputs AST JSON v0 after expansion; use this for golden comparison.

## AST JSON v0 Schema

See `docs/reference/ir/ast-json-v0.md` for the minimal schema used in Phase 2.

## Troubleshooting

- Child timeout: increase `NYASH_NY_COMPILER_TIMEOUT_MS` or simplify macro code; strict mode fails fast.
- Invalid JSON from child: ensure `expand(json)` returns a valid AST JSON v0 string.
- No changes observed: confirm your macro is registered and the runner route is enabled.
- Capability denied: set caps explicitly（デフォルトは全OFF）
  - `NYASH_MACRO_CAP_IO=1` → IO系Box（File/Path/Dir）許可
  - `NYASH_MACRO_CAP_NET=1` → NET系Box（HTTP/Socket）許可
  - `NYASH_MACRO_CAP_ENV=1` → `MacroCtx.get_env` 許可（将来拡張）

## Roadmap
- MacroCtx capabilities (io/net/env) expressed via nyash.toml per-macro.
- Diagnostics: JSONL tracing (`NYASH_MACRO_TRACE_JSONL`) and span/source maps.
- Golden tests for expanded JSON; strict mode as default.

## Capabilities (io/net/env)

Purpose: restrict side‑effects and ensure deterministic macro expansion.

- Default: all OFF (io=false, net=false, env=false)
- Behavior:
  - io=false → no FileBox/FS access inside macro; AST JSON only
  - net=false → no Http/Socket inside macro
  - env=false → MacroCtx.getEnv disabled; child inherits scrubbed env
- Planned configuration (nyash.toml): see `docs/reference/macro/capabilities.md`
- PoC mapping: child is always `NYASH_VM_USE_PY=1`, `NYASH_DISABLE_PLUGINS=1`, timeout via `NYASH_NY_COMPILER_TIMEOUT_MS`

## Top-level static MacroBoxSpec (safety)
- 既定では無効（`NYASH_MACRO_TOPLEVEL_ALLOW=0`）。Box宣言なしで `static function MacroBoxSpec.expand` を受理したい場合は `--macro-top-level-allow` を指定してください。
