//! vm_config.rs — VM環境変数設定の集約化（Box化）
//!
//! 従来42ファイルに散在していた環境変数チェックを1箇所に集約。
//! テスト容易性・保守性・パフォーマンスの向上を実現。

use std::sync::OnceLock;

/// VM設定の singleton インスタンス
static VM_CONFIG: OnceLock<VmConfig> = OnceLock::new();

/// VM環境変数設定を一元管理する構造体
#[derive(Debug, Clone)]
pub struct VmConfig {
    // --- Call系トレース ---
    pub call_trace: bool,
    pub call_arg_trace: bool,

    // --- PHI系 ---
    pub phi_trace: bool,
    pub phi_tolerate_undefined: bool,

    // --- Birth lifecycle ---
    pub birth_trace: bool,
    pub auto_birth_cpp: bool,
    pub auto_birth_dev: bool,

    // --- その他のトレース ---
    pub general_trace: bool,      // NYASH_VM_TRACE (汎用トレース)
    pub ret_trace: bool,
    pub branch_trace: bool,
    pub param_trace: bool,
    pub resolve_trace: bool,
    pub trace_exec: bool,

    // --- Behavior flags ---
    pub tolerate_void: bool,
    pub dev_fallback: bool,       // NYASH_DEV_FALLBACK (開発時フォールバック)
    pub recv_arg_fallback: bool,
    pub strlike_instance_coerce: bool,

    // --- Limits ---
    pub max_instructions: Option<usize>,
    pub max_block_exec: Option<usize>,
}

impl VmConfig {
    /// グローバル設定インスタンスを取得（初回のみ環境変数を読み込み）
    pub fn global() -> &'static VmConfig {
        VM_CONFIG.get_or_init(Self::from_env)
    }

    /// 環境変数から設定を読み込み（内部使用）
    fn from_env() -> Self {
        Self {
            // Call系
            call_trace: env_bool("NYASH_VM_CALL_TRACE"),
            call_arg_trace: env_bool("NYASH_VM_CALL_ARG_TRACE"),

            // PHI系
            phi_trace: env_bool("NYASH_VM_PHI_TRACE"),
            phi_tolerate_undefined: env_bool("NYASH_VM_PHI_TOLERATE_UNDEFINED"),

            // Birth
            birth_trace: env_bool("NYASH_VM_BIRTH_TRACE"),
            auto_birth_cpp: env_bool("NYASH_VM_AUTO_BIRTH_CPP"),
            auto_birth_dev: env_bool("NYASH_VM_AUTO_BIRTH_DEV"),

            // Other traces
            general_trace: env_bool("NYASH_VM_TRACE"),
            ret_trace: env_bool("NYASH_VM_RET_TRACE"),
            branch_trace: env_bool("NYASH_VM_BRANCH_TRACE"),
            param_trace: env_bool("NYASH_VM_PARAM_TRACE"),
            resolve_trace: env_bool("NYASH_VM_RESOLVE_TRACE"),
            trace_exec: env_bool("NYASH_VM_TRACE_EXEC"),

            // Behavior
            tolerate_void: env_bool("NYASH_VM_TOLERATE_VOID"),
            dev_fallback: env_bool("NYASH_DEV_FALLBACK"),
            recv_arg_fallback: env_bool("NYASH_VM_RECV_ARG_FALLBACK"),
            strlike_instance_coerce: env_bool("NYASH_VM_STRLIKE_INSTANCE_COERCE"),

            // Limits
            max_instructions: env_usize("HAKO_VM_MAX_INSTRUCTIONS")
                .or_else(|| env_usize("NYASH_VM_MAX_INSTRUCTIONS")),
            max_block_exec: env_usize("HAKO_VM_MAX_BLOCK_EXEC")
                .or_else(|| env_usize("NYASH_VM_MAX_BLOCK_EXEC")),
        }
    }
}

/// 環境変数を bool として読み込み（"1" なら true）
fn env_bool(key: &str) -> bool {
    std::env::var(key).ok().as_deref() == Some("1")
}

/// 環境変数を usize として読み込み
fn env_usize(key: &str) -> Option<usize> {
    std::env::var(key).ok().and_then(|s| s.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vm_config_singleton() {
        let cfg1 = VmConfig::global();
        let cfg2 = VmConfig::global();
        assert!(std::ptr::eq(cfg1, cfg2)); // 同一インスタンス
    }
}
