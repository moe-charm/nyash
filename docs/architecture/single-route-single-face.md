# Single Route, Single Face — Runtime Architecture (Phase 15.7)

Purpose
- Eliminate duplicated special-cases and hardcoded branches across builtin and plugins.
- Provide one clear path for init, routing, semantics, and diagnostics so code stays simple and predictable.

Core Principles
- One Boot: PluginBootBox
- One Router: MethodRouterBox (resolver → invoker)
- One Spec: TypeRegistry (type_id/method_id/arity)
- One Semantics: hako_core_* crates (String/Array/Map)
- One Bridge: HostAPIBox (reverse calls, grow buffers)
- One Env: EnvGateBox (aliases + truthy parsing)
- One Ingest: SpecIngestBox (nyash_box/hako_box)
- One Identity: HandleCacheBox ((type_id, instance_id) → Arc)
- One Log: DiagnosticsBox (JSON-structured traces)

## Boxes and Responsibilities

- runtime/plugin_boot_box (added)
  - Idempotent boot (OnceLock). Loads config (override → nyash.toml → hako.toml → hakorune.toml) and registers providers into v2 BoxFactoryRegistry.
  - Called from runner, unified_registry, provider (same function).

- MethodRouterBox (planned)
  - Single entry for method routing. Avoid hardcoded “if StringBox … else if PluginBox …”.
  - Flow: Router → Resolve (TypeRegistry) → Invoke (BuiltinInvoker | PluginInvoker) → hako_core_*.

- TypeRegistry
  - Single source of truth for builtin method ids and arities. Plugin ids ingested from specs.
  - Builder/VM/LLVM read the same ids (no divergence).

- hako_core_* (string/array/map)
  - Define the semantics (e.g., indexOf/substring/slice/keys/values). Both builtin and plugin use the same functions.
  - Builtin MapBox uses `hako_core_map::keys_sorted_from_map_str` and `values_for_keys_str` to keep ordering consistent with plugin path.

### Slots (SSOT)
- StringBox slots (TypeRegistry):
  - 300: len/length/size, 312: isEmpty, 301: substring(start,end), 314: charAt(idx)
  - 303: indexOf(needle[,from]), 313: lastIndexOf(needle[,from])
  - 302: concat(rhs), 304: replace(from,to), 305: trim(), 306: toUpper(), 307: toLower()
  - 308: toString(), 309: stringify() [alias]
- ArrayBox slots:
  - 102: len/length/size, 100: get, 101: set, 103: push, 104: pop, 105: clear
  - 106: contains, 107: indexOf, 108: join, 109: sort, 110: reverse, 111: slice, 112: toJSON
- MapBox slots:
  - 200: size, 201: len, 202: has, 203: get, 204: set, 205: delete/remove, 206: keys, 207: values, 208: clear, 209: toJSON


- HostAPIBox
  - Reverse-call C ABI wrappers (nyrt_host_call_slot/name) with grow-on-short-buffer.
  - Provides slot constants (e.g., ARRAY_SET=101). Stage-2 implementations use this box.

- EnvGateBox
  - Normalizes env reads (HAKO_/NYASH_ aliases, truthy parsing). Callers import from here.

- SpecIngestBox
  - Ingests nyash_box.toml/hako_box.toml paths discovered near libraries; keeps loader code thin.

- HandleCacheBox
  - Guarantees identity reuse for (type_id, instance_id) → Arc across reconstructions.

- DiagnosticsBox
  - JSON-structured logs for stable smokes. Scattershot eprintln! should migrate here.

## Policies

- Plugin-On (policy=auto/force): No builtin fallback for core boxes (Array/Map/String).
  - ProviderBox and NewBox handler enforce Fail-Fast if plugins are unavailable.

- Extern Lowering
  - String: nyrt.string.{length,indexOf,lastIndexOf,substring,charAt,replace}
  - Array: nyrt.array.size
  - Map: nyrt.map.{size,keys,values}


## Core Semantics Responsibilities

- Collections: unification across String/Array/Map
  - size/isEmpty: all collections implement `size()` and `isEmpty()`; `length()` remains as alias where present.
  - get semantics:
    - Array/String: `get(index)` out-of-bounds returns `null`.
    - Map: `get(missing-key)` returns `null` (legacy error/suggestion strings removed).
  - mutators return value:
    - Array: `set(index, val)` and `push(val)` return `Void`.
    - Map: `set(key, val)`, `clear()`, `delete(key)` return `Void`.
  - ordering:
    - Map.keys(): returns Array of keys in lexicographic (dictionary) order (ascending, UTF‑8 byte order).
    - Map.values(): returns values aligned to the same key order as Map.keys() (dictionary order).
  - slice policy (Array):
    - `slice(start, end)` clamps to bounds. Special case: `end < 0` means “to end” (clamped to `len`).
      The canonical implementation is `hako_core_array::slice_bounds`.

- Router normalization rules (Phase‑1 facade)
  - Router normalizes mutator return values to `Void` (Array.set/push, Map.set/clear/delete) irrespective of builtin vs plugin backend.
  - Router delegates semantics to `hako_core_*` crates; Builtins call directly, plugins may adapt via HostHandle/FFI.

- Semantics (selected)
  - Array.slice(start, end): end < 0 — current quick-selfhost profile expects clamp-to-len.
    - i0 = clamp(start, 0..len)
    - i1 = if end < 0 then len else clamp(end, 0..len)
    - Core policy is defined in hako_core_array::slice_bounds.
  - Map.keys/values: Stage-1 shim (string) and Stage-2 HostHandle(ArrayBox) under host-export builds. Tests prefer Stage-1 unless configured.

Plugin-On Strict (Fallback Gate)
- Default: builtin fallback is allowed for core boxes when plugin-on, to keep bring-up stable.
- To enforce strict plugin-on (fail-fast, no builtin fallback), set:
  - `NYASH_PLUGIN_ON_STRICT=1` (alias: `HAKO_PLUGIN_ON_STRICT=1`).
- Scope: enforced at ProviderBox boundary (NewBox). This does not affect plugin-off path.

TypeBox Re‑Probe Hardening
- When near-spec ingest is missing, the loader now records per-Box `invoke_id` via TypeBox symbol probing.
- On deduced library paths, the loader also records `type_id` into specs to make `type_id → invoke` resolution robust.

## Migration Notes

- Avoid hardcoded forks (e.g., special-case String/Array/Map in routers). Move logic to Router+Core.
- Prefer Fail-Fast to fallback when policy demands deterministic behavior (plugin-on core boxes).
- Keep smokes green with structured logs and stable EnvGateBox flags.


### Verification (Smokes)
- plugin-on profile:
  - Map: `get(miss)==null`, `set/clear/delete` do not yield a value (Void semantics), `keys/values` follow dictionary order.
  - Array: `slice` with negative `end` clamps to `len`, `get(oob)==null`, `set/push` are `Void`.
- plugin-on-strict profile:
  - Same as above, plus no builtin fallback permitted (fail-fast when plugins unavailable).

## Router Built‑in Branches（Phase‑1の位置づけ）

- 対象: StringBox / ArrayBox / MapBox の最小メソッド群（length/size/indexOf/substring/charAt、Array.get/set/push/slice、Map.size/get/has/keys/values など）。
- 目的: 「単一路」を先に完成させるための足場。VM 内の分岐を Router に集約し、意味論は hako_core_* に寄せる。
- 種別: 最適化（fast‑path）ではなく、設計上のファサード（Invoker の片側）。
  - 実装は TypeRegistry による arity 検証を通し、表駆動（slot/id）へ段階移行する前提。
  - プラグイン経路は PluginInvoker（FFI）で統一。Router は builtin/plugin の区別を意識しない。
- 撤退計画: TypeRegistry のテーブル化（Builder/VM/LLVM の ID 統一）が完了したら、箱ごとの if 分岐は表走査へ漸進置換する。
  - その時点でも意味論は hako_core_* が唯一の真実。Router は薄いルーティングのまま。
- 注意: これは「ハードコーディングで高速化」ではなく、「分岐散在の撤去と一経路化」のための一時的な配置換え。挙動は docs の仕様（core/Extern 降ろし）に一致する。


### CallableBox (One Call Surface)
- CallableBox encapsulates (receiver, method, arity).
- Router dispatches `call/callAsync` via the same MirCall path, keeping sync/async unified.
- Created via `ArrayBox.methodRef(name, arity)` or `env.callable.make(recv, name, arity)`.