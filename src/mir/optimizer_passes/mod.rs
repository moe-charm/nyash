pub mod boxfield;
pub mod diagnostics;
pub mod intrinsics;
pub mod normalize;
pub mod reorder;
// normalize_core13_pure removed (deprecated)
// normalize_legacy_all removed (dead wrapper, delegated to normalize::normalize_legacy_instructions)
pub mod hints_config;
pub mod detectors;
pub mod hints;
pub mod reporter;
