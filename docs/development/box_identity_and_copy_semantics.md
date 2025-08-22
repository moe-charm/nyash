Box Identity and Copy Semantics (Nyash)

Context
- In Nyash, everything is a Box. Some boxes carry identity (external handles, stateful resources), while others are pure values.
- Picking clone_box vs share_box is critical: getting it wrong can silently fork state or break handle identity (e.g., plugin/socket/file handles).

Key Terms
- Identity Box: Represents an external or stateful handle where the instance_id (or similar) must remain stable across uses. Examples: PluginBoxV2 (all plugin boxes), Socket boxes, FileBox, DB handles.
- Value Box: Pure data containers where cloning yields an equivalent value. Examples: StringBox, IntegerBox, JSONBox, ArrayBox/MapBox (when used as data).

Rules of Thumb
- When preserving identity matters (handle/connection/descriptor): use share_box.
- When passing or returning pure values: use clone_box (or deep copy APIs) so callers can freely mutate their copy.
- Do not call clone_box on identity boxes during common control flows like method dispatch, result unwrapping, or variable assignment unless you’re explicitly birthing a new instance.

Where mistakes commonly happen
1) Result unwrap (ResultBox.get_value)
   - Wrong: always clone_box() — duplicates plugin handles and changes instance_id.
   - Right: if value is PluginBoxV2 (or any identity box), return share_box(); else clone_box().

2) Method dispatch (receiver preparation)
   - Wrong: always clone the receiver box before method call.
   - Right: if the VM stores BoxRef(Arc<dyn NyashBox>), use Arc::share_box() for the receiver; only clone for value boxes.

3) Environment/variable storage
   - Wrong: eagerly clone_box() on set/assign.
   - Right: keep as BoxRef (Arc) where semantics require identity; only copy for value semantics.

4) Returning plugin results
   - Ensure loader returns PluginBoxV2 with the exact instance_id provided by the plugin, wrapped as BoxRef. Avoid any implicit clone.

Concrete patterns
- In ResultBox.get_value():
  - if val.is::<PluginBoxV2>() -> return val.share_box()
  - else -> val.clone_box()

- In VMValue conversions:
  - VMValue::from_nyash_box: store as BoxRef(Arc<dyn NyashBox>) for identity-bearing boxes (default safe choice).
  - VMValue::to_nyash_box: for BoxRef -> share_box().

- In plugin method dispatch (PluginBoxV2):
  - Identify receiver as PluginBoxV2 and pass its instance_id to FFI directly (avoid cloning receiver).

Anti-patterns to avoid
- Using clone_box blindly in:
  - Result unwrap paths
  - Method receiver preparation
  - Field access and variable bindings for identity boxes

Design guidance
- Consider tagging boxes with an identity flag:
  - Add BoxCore::is_identity() -> bool (default false; override to true for PluginBoxV2, Socket/File/DB boxes).
  - Provide a helper: clone_or_share_by_semantics(box: &dyn NyashBox) that calls share for identity and clone for values.
- Add debug assertions/logs in dev builds when a PluginBoxV2’s instance_id unexpectedly changes across a single logical flow (indicative of unintended clone).

Testing checklist
- Log instance_id for plugin boxes at:
  - client.get/post result (resp_id)
  - ResultBox.get_value return (should match)
  - Method dispatch receiver (should match)
  - readBody()/write logs (should match)
- For e2e tests, assert that write/mirror/read use the same response id.

Migrations for existing code
- Update ResultBox.get_value to share PluginBoxV2.
- Audit call sites that do clone_box() before method dispatch or storage; prefer BoxRef + share for identity boxes.
- Ensure plugin loader returns PluginBoxV2 as BoxRef, not cloned.

Future improvements
- Static lint: forbid clone_box on types that implement BoxCore::is_identity() == true in critical paths (unwrap, dispatch, assign).
- Provide a macro/helper to explicitly mark intent: share!(x), clone_value!(x) to make code review easier.

