## ResultBox Migration TODO (Phase 9.78h follow-up)

Goal: fully migrate from legacy `box_trait::ResultBox` to `boxes::result::NyashResultBox` (aka `boxes::ResultBox`).

### Current usages (grep snapshot)
- src/backend/vm.rs
  - Handles both new `NyashResultBox` and legacy `box_trait::ResultBox` for `.is_ok/.get_value/.get_error` (deprecation suppressed)。

### Proposed steps (small, incremental)
- Step 1: Keep dual handling but gate legacy path with feature flag or cfg for deprecation-only builds（任意）。
- Step 2: Audit call sites that construct legacy ResultBox; replace with `boxes::result::NyashResultBox` constructors。
- Step 3: Remove legacy path from VM once no legacy constructors remain。
- Step 4: Delete/Archive legacy `box_trait::ResultBox`（テスト緑後）。

### Notes
- New API already aliased: `pub type ResultBox = NyashResultBox;` so external references may transparently resolve after migration。
- Keep migration scoped: do not mix with unrelated refactors。

Last updated: 2025-08-25
