/*!
 * MIR Utilities - Phase 15 段階的根治戦略の共通ユーティリティ
 * 
 * フェーズS: 即効止血
 * フェーズM: PHI一本化  
 * フェーズL: 根本解決
 * 
 * 全フェーズで使用する汎用関数を提供
 */

pub mod control_flow;

// 外部公開API
pub use control_flow::{
    is_current_block_terminated,
    capture_actual_predecessor_and_jump,
    collect_phi_incoming_if_reachable,
    execute_statement_with_termination_check,
};