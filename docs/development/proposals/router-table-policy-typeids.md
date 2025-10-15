# Router Table, Plugin Policy, and Type IDs — Simplification (Phase 15.75 mini)

Status: landed (phase‑in continues)

Goals
- Replace growing string matches with a small, declarative table per Box.
- Centralize HostHandle slot IDs and boundary return codes.
- Read plugin policy from a single helper (env_gate_box) for consistent Fail‑Fast.
- Prepare a single source of truth for core Type IDs (follow‑up).

Scope (small, reversible)
- Introduced `src/runtime/host_handle_router/consts.rs` with slot IDs and error codes.
- Added `src/runtime/method_router_box/tables.rs` to hold declarative HostHandle routes (String/Array/Map).
- Refactored `method_router_box/plugin.rs` and `builtin.rs` to consume the table helpers and `env_gate_box`.
- Introduced `method_router_box/host_slot.rs` as the single HostHandle execution helper (plugin/builtin/primitive String share TLV + ERR_BUF_SMALL handling).
- Added `try_invoke_arc()` helper to consume a `HostSlotRoutes` table uniformly from plugin/builtin routers. This keeps the ENV gate and slot selection centralized and minimizes divergence between routers.
- builtin router now consumes the same Array/Map tables; primitive String also reuses `STRING_HOST_ROUTES` before falling back to TypeRegistry slots.
- Split VM extern adapter into per‑iface modules (`extern_{string,array,map,set,env}.rs`) with `nyrt.time.now_ms` remaining inline.
- Behavior remains unchanged (env flags preserved), no semantic shifts.

Design
- HostHandle consts (in `host_handle_router::consts`)
  - Array: `GET=100`, `SET=101`, `SIZE=102`
  - Map: `SIZE=200`, `HAS=202`, `GET=203`, `SET=204`
  - String: `LEN=300`
  - RC: `ERR_UNKNOWN=-1`, `ERR_UNSUPPORTED=-11`, `ERR_BAD_ARGS=-13`, `ERR_BAD_RET=-14`, `ERR_BUF_SMALL=-3`
- Router tables (in `method_router_box::tables`)
  - `STRING_HOST_ROUTES`: `{size|len|length}` guarded by `HAKO/NYASH_STRING_SIZE_FORCE_HOST`
  - `ARRAY_HOST_ROUTES`: `{size|len|length}` guarded by `HAKO/NYASH_ARRAY_SIZE_FORCE_HOST`
  - `MAP_HOST_ROUTES`: `{ size, has, get, set }` with per-method gates plus `HAKO/NYASH_MAP_FORCE_HOST`
  - Helpers (`EnvToggle`, `HostSlotRoute`, `HostSlotRoutes`) encapsulate env logic.
- Plugin policy
  - `env_gate_box` remains the single entry (helpers already exist). Callers use `plugin_policy_on()` and `bool_any`.
- VM extern adapter
  - Each iface registers via its split module; reduces duplication and keeps slot constants in one place.

Migration plan
1) Add consts and table helpers (done).
2) Switch plugin/builtin routers to use the tables (done).
3) Split extern adapter modules and remove the monolithic core (done).
4) (In progress) Introduce `src/types/ids.rs` and migrate remaining runtime/type prints.
5) (Next) Sweep direct `std::env::var` reads in runtime to `env_gate_box`.
6) (Next) Expand tables to cover additional HostHandle routes (Array get/set) once env policy is set.

Testing
- Targeted smokes: `host_handle_router_*`, plugins parity (map/array size/has/get/set), and quick profile.
- No profile default changes; only internal refactor.

Notes (follow‑up)
- String.size/len は Builder が Extern(`nyrt.string.length`) へ正規化するため、Router 表は（primitive String のみ）開発/検証用途で利用します。Extern 経路の受領者素材化は Builder 側（finalize/repair）で担保します。

Backout
- Delete `consts.rs` and restore in‑file literals; revert single file change in plugin router. No data migration.
