// Removed unused time imports
//
// Phase 1 Cleanup (2025-10-11):
// - ✅ handle_extern_call() removed (142 lines) - ExternCall instruction retired
// - ✅ handle_op_eq() removed (34 lines) - unused after ExternCall retirement
//
// Note: ExternCall instruction was retired in favor of Call+Callee::Extern unified path.
// All extern functionality now routed through extern_adapter.rs.
