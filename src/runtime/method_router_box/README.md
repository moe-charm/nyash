Method Router Box — Split Adapters (Phase 15.75)

責務
- `mod.rs`: エントリ関数 `route(..)` のみ。最小の前処理（MethodRef/HostHandle解決、プリミティブString）を行い、
  以降は `plugin::try_route_plugin_box` → `builtin::try_route_builtin_box` に早期委譲する。
- `plugin.rs`: Plugin(TypeBox v2) 経路。HostHandleRouter の強制スロット（Map.size/has/get/set 等）の早期適用もここで扱う。
- `builtin.rs`: 既存の Rust 実装（legacy-boxes）。File/Callable/Array/Map などの腕を順次こちらへ集約する。

入出力（共通）
- 入力: `receiver: &VMValue`, `method: &str`, `args: &[VMValue]`
- 出力: `Result<VMValue, VMError>`（Void は `VMValue::Void`）

ガード/方針
- Arity: TypeRegistry の known arities に基づくガード（`maybe_arity_guard`）を境界で適用。
- HostHandle: `HostHandleBox` は実体へ置換して再入（再帰）。
- String: プリミティブはここで直接処理（BoxRef ではないため builtin へ委譲不可）。
- Plugin → Builtin の順に 1 本ずつ委譲し、未処理は `method_not_supported` を返す。

撤退計画
- 旧経路（mod.rs 内の巨大分岐）は委譲実装への移設完了後に物理削除（済み）。
- 以後の機能追加は `plugin.rs` / `builtin.rs` へ集約し、`mod.rs` は薄い入口を維持する。

HostHandle slots / ENV（開発用）
- Array: `100:get/1`, `101:set/2`, `102:len/0`
- Map: `200:size/0`, `202:has/1`, `203:get/1`, `204:set/2`
- String: `300:size/0`
- 強制フラグ（例）
  - `NYASH_ARRAY_SIZE_FORCE_HOST=1`
  - `NYASH_MAP_SIZE_FORCE_HOST=1`, `NYASH_MAP_HAS_FORCE_HOST=1`
  - `NYASH_MAP_GET_FORCE_HOST=1`, `NYASH_MAP_SET_FORCE_HOST=1`
  - `NYASH_STRING_SIZE_FORCE_HOST=1`
（注）ENV は観測用の開発フラグ。既定OFF。統一道後に撤去予定。
