/*!
 * phi_core – Unified PHI management scaffold (Phase 1)
 *
 * Purpose:
 * - Provide a single, stable entry point for PHI-related helpers.
 * - Start with re-exports of existing, verified logic (zero behavior change).
 * - Prepare ground for gradual consolidation of loop PHI handling.
 */

pub mod common;
pub mod if_phi;
pub mod loop_phi;

// Public surface for callers that want a stable path:
// Phase 1: No re-exports to avoid touching private builder internals.
// Callers should continue using existing paths. Future phases may expose
// stable wrappers here once migrated.
