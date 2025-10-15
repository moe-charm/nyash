/*!
 * Builtin Box Implementations (Phase 15.5: Everything is Plugin Migration)
 *
 * 🎯 Purpose: Separated implementations for easy deletion during Phase 2
 * 🗓️ Timeline: These files will be deleted one by one in Phase 2.1-2.6
 *
 * Deletion Order (dependency-based):
 * 1. string_box.rs    - Phase 2.1 ✅ Plugin ready
 * 2. integer_box.rs   - Phase 2.2 ✅ Plugin ready
 * 3. bool_box.rs      - Phase 2.3 🔄 Plugin needed
 * 4. array_box.rs     - Phase 2.4 🔄 Plugin check needed
 * 5. map_box.rs       - Phase 2.5 🔄 Plugin check needed
 * 6. console_box.rs   - Phase 2.6 🔄 Plugin exists, remove LAST
 * 7. null_box.rs      - TBD: 🤔 Keep as language primitive?
 */

// Phase 2.1-2.6: Delete these modules one by one
// string_box removed (Phase 2.1) — use nyash-string-plugin
#[cfg(feature = "legacy-boxes")]
pub mod integer_box;   // DELETE: Phase 2.2 (plugin ready)
#[cfg(feature = "legacy-boxes")]
pub mod bool_box;      // DELETE: Phase 2.3 (plugin needed)
// array_box removed (Phase 2.4) — use nyash-array-plugin
// map_box removed (Phase 2.5) — use nyash-map-plugin
#[cfg(feature = "legacy-boxes")]
pub mod console_box;   // DELETE: Phase 2.6 (LAST - critical for logging)

// Special consideration
#[cfg(feature = "legacy-boxes")]
pub mod null_box;      // DISCUSS: Keep as primitive?
#[cfg(feature = "legacy-boxes")]
pub mod set_box;       // Thin wrapper over MapBox (legacy helper)
