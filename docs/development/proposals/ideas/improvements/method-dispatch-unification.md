# Method Dispatch Unification — Router/Invoker/TypeRegistry (Selfhost‑Ready)

Status: Accepted (Phase 15.7 → 16, incremental)

Purpose
- Eliminate double implementation (builtin vs plugin) while keeping plugin‑on/off both green.
- Make MIR the single source of truth so VM and LLVM share the same semantics.
- Reduce Selfhost VM complexity via a single Router entry (`route_method_call`).

Current Reality (Option B: double implementation)
- Surface contract is unified (method names/arity), but implementations are split:
  - Builtin: VM `execute_method_call()` matches on strings (indexOf/substring …)
  - Plugin: v2 TypeBox FFI + spec/config driven `resolve_method_id()` → TLV invoke
- TypeRegistry: used for verification/shape only; not used at execution time.
- Result: duplicate logic (e.g., String.indexOf) lives in VM and in string plugin.

Design Goals
1) Shared logic in one crate: hako_core_string (and future hako_core_array/map)
2) Method resolution unified: TypeRegistry provides (type_id, method_id) for builtin and plugin
3) Two invokers under a single Router: BuiltinInvoker and PluginInvoker
4) MIR‑first: method_id injection (or Method callee) is produced in one place; VM/LLVM lower from the same MIR

Revised 4‑Step Plan (adopted)
Step 1: Core crate (1–2 days)
- New crate `crates/hako_core_string/` exporting minimal, deterministic functions:
  - `index_of(s:&str, needle:&str, from:i64) -> i64`
  - `last_index_of(s:&str, needle:&str, from:i64) -> i64`
  - `substring_bytes(s:&str, start:i64, end:i64) -> String`
  - `char_at(s:&str, idx:i64) -> String`
- VM builtin と string plugin の内部からこの関数群を呼ぶ（意味論を1箇所に集約）。

Step 2: Method resolution unification (1 day)
- Resolve `(type_id, method_id)` via TypeRegistry（MethodRegistryBox は TypeRegistry に統合）。
- Builtin も Plugin も同じ ID 空間で扱う（現行 canonical: String=13, Array=12, Map=11）。
- 入口 API（擬似）:
  ```rust
  fn resolve_method_handle(type_name:&str, method:&str, arity:usize) -> Result<(u32,u32), Error>
  ```

Step 3: Two invokers (2–3 days)
- BuiltinInvoker: `(type_id, method_id, receiver, args) -> VMValue`
  - 文字列系は `hako_core_string` を呼ぶ。
- PluginInvoker: 既存の v2 invoke を踏襲（TLV encode/decode）。

Step 4: Router 統合（1 day）
- `execute_method_call`/Selfhost の `route_method_call` は 1 行で経路を選択:
  ```nyash
  me.route_method_call(recv, method, args)
  ```
- 実体は `(type_id, method_id)` を解決 → PluginBoxV2 なら PluginInvoker、それ以外は BuiltinInvoker。

MIR と LLVM の接続
- MIR 正規化: builder が method_id を注入（BoxCall または Callee::Method）。
- VM: Router→Invoker で実行。今回の “橋（plugins OFFのみ builtin 実装へ）” は Router 導入後に置き換え。
- LLVM: lower_boxcall/lower_call で同じ `(type_id, method_id)` を用い、
  - Plugin: 既存の `nyash_plugin_invoke_v2_shim` に降ろす（VMと同じTLV semantics）
  - Builtin: 当面は `nyrt.string.*` / `nyrt.array.*` / `nyrt.map.*` の extern に降ろす（VM 側の extern_adapter に実装済み）。必要があれば後から inlining 最適化。

Adoption Notes（提案の修正点の反映）
- MethodRegistryBox は実体を TypeRegistry に統合（実装は段階移行）。ドキュメント上も TypeRegistry を第一参照にする。
- Plugin 側も `hako_core_string` を参照し、二重実装を排除（将来の差分も1箇所）。
- 4 ステップは上記に簡略化し、重複説明を削る。

Acceptance / Success Criteria
- quick（plugins OFF）と quick‑selfhost（plugin‑on）の String smokes が常緑（現状達成）。
- 新規 `hako_core_string` 導入後も同スモーク緑。
- Method resolution 経路が VM/LLVM で一致（同じ method_id を使用）。
- Selfhost VM では `route_method_call` 1本化で分岐ロジックを95%削減。

Risks / Notes
- Unicode/byte ポリシーは byte 既定（NYASH_STR_CP は実験/開発向け）。仕様を先に文書化する。
- `TypeRegistry` 実戦投入までは「過度に厳密にし過ぎない」。config/spec→TypeRegistry の順で導入、Fail‑Fast は段階的に強化。
- Invoker/Router 導入は小差分で段階適用（先に String、次に Array/Map）。

ENV/Profiles
- plugin‑on profile は `HAKO_PLUGIN_POLICY=auto` とし、builtin 実装は常用しない（parity 検証のみ）。
- デバッグ時は `NYASH_METHOD_REG_TRACE=1` で解決ログを Method 経路に一本化（VM/LLVM 共通）。
