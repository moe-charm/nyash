# ENV Variables — Core (Plugins/Provider)

**Important / 重要**
- Primary names are `HAKO_*`. `NYASH_*` are compatibility aliases. Use `HAKO_*` in new scripts/docs; runners mirror common `NYASH_*` → `HAKO_*` non‑destructively.
- Prefer CLI flags / profiles first; keep ENV toggles short‑lived for experiments only.  
  CLI/Profiles を優先し、ENV は短命な開発補助（実験後は元に戻すこと）。

Key variables (current)
- `HAKO_USING` — unified using mode (full|basic|off). New primary.
  - `full`（既定）: using + AST merge + file using をすべて有効化（開発体験重視）。
  - `basic`: using のみ（AST merge と file using は無効）。
  - `off`/`0`: using 無効。
  - 互換: `NYASH_USING`/`NYASH_USING_AST`/`NYASH_ALLOW_USING_FILE` は引き続き動作しますが、`HAKO_USING` が指定された場合はそちらを優先します。
- `HAKO_PLUGIN_POLICY` (auto|off|force) — primary plugin load policy (preferred)
  - `auto`（既定）: Plugin優先。未提供/失敗時は互換の範囲でbuiltinへフォールバック可。
  - `off`: Plugin無効（すべてbuiltin/embedded）。
  - `force`（Strict）: Plugin提供がある型はフォールバック禁止。Plugin未実装/失敗は即 Fail‑Fast（InvalidInstruction）。
    - 代表エラー例: `plugin strict: builtin fallback disabled for MapBox.noSuchMethod(0 args)`
    - ルーター境界で強制され、builtin への委譲は行われません。
  - Smokes noise control: `SMOKES_STRICT_NOISE=1` で、動的プラグイン再ビルド失敗の一部メッセージを WARN にダウングレード（後続の成功時にノイズを抑制）。
- `HAKO_PLUGIN_CONFIG` — plugin config path (prefer `hako.toml`)  
  compat: `NYASH_PLUGIN_CONFIG`
- `NYASH_PLUGIN_MAP_ARRAY_HANDLE` — Stage‑2: 1 で keys/values HostHandle 経路を有効化。0 で Stage‑1(keysS/valuesS) シム（plugins プロファイルは既定ON）。
- `NYASH_MAP_FORCE_HOST` — Dev/Test: Map.size/has/get/set を HostHandleRouter の slot(200/202/203/204) へ強制。既定OFF（plugins プロファイルはON）。
- `NYASH_ARRAY_FORCE_HOST` — Dev/Test: Array.size/get/set を HostHandleRouter の slot(102/100/101) へ強制。既定OFF（plugins プロファイルはON）。
- `NYASH_ARRAY_SIZE_FORCE_HOST` — Dev/Test: Array.size を HostHandleRouter の slot(102) へ強制（互換）。
- `NYASH_STRING_SIZE_FORCE_HOST` — Dev/Test: String.size/len を HostHandleRouter の slot(300) へ強制。既定OFF（plugins プロファイルはON）。
- `HAKO_HOST_HANDLE_TRACE` / `NYASH_HOST_HANDLE_TRACE` — HostHandle slot呼び出しの観測ログ（短命/既定OFF）
- `HAKO_MIRIO_PROVIDER` (scan|yyjson) — MirIoBox の入力プロバイダー選択（既定=scan）。yyjson は JSON プラグイン配置が必要。

### Smoke runner (profiles/tests)
- `SMOKES_REQUIRED_PLUGINS` — 必須プラグインのキー集合を動的指定（カンマ/スペース区切り）。
  - 既定: `stringbox integerbox mathbox arraybox mapbox filebox setbox`
  - 例: `SMOKES_REQUIRED_PLUGINS="stringbox arraybox mapbox setbox" tools/smokes/v2/run.sh --profile plugins`
  - ランナーはキー→crate 名へマップし、`cargo build -p <crate>` を自動実行（不足は警告し SKIP 方針）。

### JSON Canonicalization (testing aid)

- `HAKO_JSON_CANON` (0|1) — Enable canonicalization (sorted object keys, arrays preserve order) for JSON golden/tests. Default OFF.
  - Scope: Parser AST JSON CLI already uses canonical emit by default; this flag is for future MirIoBox ingress when host bridge is wired.
  - Behavior: When OFF, behavior unchanged. When ON (and Extern bridge available), MIR JSON at ingress is normalized for stable comparisons.
  - 互換: `NYASH_JSON_PROVIDER`（legacy）。同時指定時は `HAKO_MIRIO_PROVIDER` を優先。
- `HAKO_ALLOW_USING_FILE` — using でファイルパス参照を許可（開発/スモーク用）。
- `NYASH_USING_AST` — using prelude の AST マージを有効化（開発/スモーク用）。

### Gate C (NyVM 直実行の薄配線)
- `NYASH_GATE_C_DIRECT=1` — `--nyvm-json-file/--nyvm-pipe` を直接 Interpreter に接続し、数値1行のみ出力（静音/Fail‑Fast）。
  - Canonicalizer: `{type:"i64", value:N}` を整数へ自動アンラップ（dst/ret.value は既定対応、他オペランドは段階拡張）。
  - プラグインは既定OFF（`NYASH_DISABLE_PLUGINS=1`）。

## FFI / extern_c (Phase 15.76)

- `HAKO_FFI_ALLOW_LIST` — 追加許可するシンボルをカンマ区切りで指定
  - 例: `HAKO_FFI_ALLOW_LIST=llvm_compile_mir_to_object`
- `HAKO_FFI_ALLOW_ALL` — 1 ですべて許可（開発専用。CI/配布では禁止）
- `HAKO_FFI_LIB_PATHS` — バックエンドlib探索パス（`:`区切り）
  - 既定探索: `./target/release`, `$NYASH_ROOT/target/release`, `.`
  - 例: `HAKO_FFI_LIB_PATHS=$(pwd)/target/release`

TOML（プロジェクト設定）
```
[ffi.dynamic]
allow = ["strlen", "getpid", "system", "llvm_compile_mir_to_object"]
```
優先順位（強い→弱い）: CLI(将来) → ENV → TOML → 既定（最小）

Deprecated (compat) — avoid in new scripts
- `NYASH_USE_PLUGIN_BUILTINS` — superseded by `HAKO_PLUGIN_POLICY`
- `NYASH_PLUGIN_OVERRIDE_TYPES` — superseded by `HAKO_PLUGIN_POLICY`
- `NYASH_BUILTIN_DISABLE_{STRING|ARRAY|MAP}` — superseded by `HAKO_PLUGIN_POLICY`

Profiles
- plugin‑on: sets `HAKO_PLUGIN_POLICY=auto`, `HAKO_PLUGIN_CONFIG=hako.toml` (compat: `NYASH_PLUGIN_CONFIG`)
- plugins: Stage‑2 HostHandle 既定ON（`NYASH_PLUGIN_MAP_ARRAY_HANDLE=1`）＋ HostHandleRouter 経路を優先（Map/Array/String の強制ENVをON）
- quick: 段階導入（最小）— `NYASH_ARRAY_SIZE_FORCE_HOST=1` のみ既定ON。他は必要時に opt‑in。

Birth Adoption
- VM will call `birth()` when a plugin box is created with `instance_id=0`, and adopt the returned handle.
- No‑op when `birth` does not exist.

Notes
- Prefer CLI/Profiles over ENV when possible; ENV should be minimal and scoped.
- Primary names are `HAKO_*`; `NYASH_*` are compatibility aliases. 新規は `HAKO_*` を優先。短命のデバッグ用 ENV のみプロファイル内で使用。
- HostHandleRouter フェーズイン中の強制ENV（`NYASH_MAP_FORCE_HOST` / `NYASH_ARRAY_FORCE_HOST` / `NYASH_ARRAY_SIZE_FORCE_HOST` / `NYASH_STRING_SIZE_FORCE_HOST`）は開発・スモーク専用。長期運用は想定しない（将来的に削除）。

### NYASH_MODULES（開発限定・段階撤退）

- 目的: 一時的に「モジュール名 → ファイルパス」を手元で差し込むための開発補助（短命）。
- 現状: 既定のプロファイルでは最小限のみ（selfhost.vm.mir_min）。builder/schema の自動注入は撤退済みです。
- 推奨: 新規テスト・アプリでは `hako.toml [modules]` / workspace の `hako_module.toml [exports]` を使用してください。ENV は最後の手段に限定します。
- 互換: 過去のスクリプトで必要な場合のみ、テストスクリプト内で明示的に設定してください（推奨しません）。

Direct load (dev‑only)
- `NYASH_PLUGIN_DIRECT_LIB` / `NYASH_PLUGIN_DIRECT_PATH` / `NYASH_PLUGIN_DIRECT_BOXES`
  - テスト/スモークで特定の .so を強制ロードしたい場合に使用（短命）。
  - 例: JSON プロバイダー: `LIB=libnyash_json_plugin.so`, `PATH=plugins/nyash-json-plugin/libnyash_json_plugin.so`, `BOXES=JsonDocBox,JsonNodeBox`。

TTL/cleanup
- 実験・観測用 ENV は短命。機能が安定したら削除または CLI/プロファイルへ昇格。


Adapter/Fallbacks
- Stage‑1 keys/values fallback is implemented in `runtime/adapters/map_keys_values_stage1.rs` and is active when `NYASH_PLUGIN_MAP_ARRAY_HANDLE` is not `1`.
- Stage‑2 (HostHandle arrays) requires `NYASH_PLUGIN_MAP_ARRAY_HANDLE=1` and returns real arrays; identity/parity tests are part of plugin‑on smokes.

### Test Hooks (HostHandleRouter)

- `HAKO_HOSTHANDLE_TEST_RET_MISMATCH=1` (alias `NYASH_HOSTHANDLE_TEST_RET_MISMATCH`)
  - Forces String.len HostHandle route to return `ERR_BAD_RETURN (-14)` for boundary smoke tests.
- `HAKO_TRACE_HOST_HANDLE=1` (alias `NYASH_TRACE_HOST_HANDLE`)
  - Enables HostHandle alloc/get trace logging.

### Scheduler & Task scopes

- `HAKO_SCHED_TRACE=1` / alias `NYASH_SCHED_TRACE`
  - Trace each poll cycle (moved/ran/budget).
- `HAKO_SCHED_POLL_BUDGET=<N>` / alias `NYASH_SCHED_POLL_BUDGET`
  - Limit tasks per poll (default 1, values ≤0 are ignored).
- `HAKO_TASK_SCOPE_JOIN_MS=<ms>` / alias `NYASH_TASK_SCOPE_JOIN_MS`
  - Join timeout for scope teardown (default 1000ms).

### Plugin Map keys()/values() Stage gate

- `HAKO_PLUGIN_MAP_ARRAY_HANDLE=1` / alias `NYASH_PLUGIN_MAP_ARRAY_HANDLE`
  - Enables Stage‑2 HostHandle arrays for Map.keys()/values(). When unset, Stage‑1 shim (`keysS/valuesS`) remains active.

### Dev stubs

- `HAKO_ENABLE_NYKERNEL_STUB=1` / alias `NYASH_ENABLE_NYKERNEL_STUB`
  - Activates the nykernel dev heap (malloc/load/store stub).

All runtime consumers read these via `env_gate_box` helpers to keep alias handling consistent.

### Runtime HostHandle & Scheduler flags

- `HAKO_ARRAY_FORCE_HOST` / `NYASH_ARRAY_FORCE_HOST` — map Array.size/get/set into HostHandle slots; pair with `HAKO_ARRAY_SIZE_FORCE_HOST` when only len/size is desired.
- `HAKO_MAP_FORCE_HOST` / `NYASH_MAP_FORCE_HOST` — enable Map.size/has/get/set HostHandle slots; per-method toggles (`HAKO_MAP_SIZE_FORCE_HOST`, etc.) refine coverage.
- `HAKO_STRING_SIZE_FORCE_HOST` / `NYASH_STRING_SIZE_FORCE_HOST` — route String.len/size through HostHandle slot 300.
- `HAKO_HOSTHANDLE_TEST_RET_MISMATCH` / `NYASH_HOSTHANDLE_TEST_RET_MISMATCH` — boundary testing hook returning `ERR_BAD_RETURN (-14)` for String.len.
- `HAKO_TRACE_HOST_HANDLE` / `NYASH_TRACE_HOST_HANDLE` — opt-in trace logging for HostHandle alloc/get.
- `HAKO_SCHED_TRACE` / `NYASH_SCHED_TRACE` — trace `poll()` scheduling cycles.
- `HAKO_SCHED_POLL_BUDGET` / `NYASH_SCHED_POLL_BUDGET` — limit queue drain per poll (default 1, <=0 ignored).
- `HAKO_TASK_SCOPE_JOIN_MS` / `NYASH_TASK_SCOPE_JOIN_MS` — timeout (ms) for task scope teardown joins (default 1000).
- `HAKO_PLUGIN_MAP_ARRAY_HANDLE` / `NYASH_PLUGIN_MAP_ARRAY_HANDLE` — Stage-2 HostHandle arrays for Map.keys()/values(); unset keeps Stage-1 shim (`keysS`/`valuesS`).
- `HAKO_ENABLE_NYKERNEL_STUB` / `NYASH_ENABLE_NYKERNEL_STUB` — enable nykernel dev heap stub (malloc/load/store).

All values are normalized via `env_gate_box` helpers so aliases stay in sync. Prefer `HAKO_*` when introducing new scripts or docs.
