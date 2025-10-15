//! lifecycle.rs — birth/fini observation & contracts helpers (extracted)

use super::super::*;
use crate::backend::mir_interpreter::contracts::policy as contracts_policy;

impl MirInterpreter {
    /// Observe NewBox lifecycle events in a single place (contracts + traces).
    pub(crate) fn lifecycle_observe_new(&mut self, dst: ValueId, box_type: &str, argc: usize) {
        // Contracts observation: record NewBox event (always record; log gated)
        let key = self.object_key_for(dst);
        self.contracts_new.insert(key);
        self.contracts_new_argv.insert(key, argc);
        contracts_policy::emit_new(box_type, argc, key.try_into().unwrap_or(i64::MAX));

        // Handle-trace (dev-only, no behavior change)
        if std::env::var("NYASH_HANDLE_TRACE").ok().as_deref() == Some("1") {
            let key = self.object_key_for(dst);
            eprintln!(
                r#"{{"kind":"handle_new","class":"{}","key":{}}}"#,
                box_type, key
            );
        }

        // Trace: new box event (dev-only)
        if Self::box_trace_enabled() {
            self.box_trace_emit_new(box_type, argc);
        }
    }

    /// Observe birth/fini handle-trace for a receiver value id and method name.
    pub(crate) fn lifecycle_observe_method(&mut self, recv_val: ValueId, method: &str) {
        if std::env::var("NYASH_HANDLE_TRACE").ok().as_deref() != Some("1") {
            return;
        }
        if method != "birth" && method != "fini" { return; }
        let recv_cls = match self.reg_load(recv_val).unwrap_or(VMValue::Void) {
            VMValue::BoxRef(b) => b.type_name().to_string(),
            _ => "<unknown>".to_string(),
        };
        let key = self.object_key_for(recv_val);
        eprintln!(
            r#"{{"kind":"handle_{}","class":"{}","key":{}}}"#,
            method, recv_cls, key
        );
    }

    /// Record and emit birth contracts info.
    pub(crate) fn lifecycle_contracts_birth(&mut self, recv_val: ValueId, argc_birth: usize) {
        let key = self.object_key_for(recv_val);
        let seen_new = self.contracts_new.contains(&key);
        let duplicate = crate::common::lifecycle_contracts::record_birth(&mut self.contracts_born, key);
        let argc_new = self.contracts_new_argv.get(&key).cloned().unwrap_or(0);
        contracts_policy::emit_birth(seen_new, duplicate, argc_new, argc_birth, key.try_into().unwrap_or(i64::MAX));
    }
}
