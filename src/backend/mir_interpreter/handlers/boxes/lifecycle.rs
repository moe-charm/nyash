//! lifecycle.rs — birth/fini observation & contracts helpers (extracted)

use super::super::*;

impl MirInterpreter {
    /// Observe NewBox lifecycle events in a single place (contracts + traces).
    pub(crate) fn lifecycle_observe_new(&mut self, dst: ValueId, box_type: &str, argc: usize) {
        // Contracts observation: record NewBox event (dev-only)
        if crate::config::env::check_contracts() {
            let key = self.object_key_for(dst);
            self.contracts_new.insert(key);
            self.contracts_new_argv.insert(key, argc);
            eprintln!(
                r#"{{"kind":"contracts_newbox","class":"{}","argc":{},"key":{}}}"#,
                box_type,
                argc,
                key
            );
        }

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
        if !crate::config::env::check_contracts() { return; }
        let key = self.object_key_for(recv_val);
        let seen_new = self.contracts_new.contains(&key);
        let duplicate = !self.contracts_born.insert(key);
        let argc_new = self.contracts_new_argv.get(&key).cloned().unwrap_or(0);
        eprintln!(
            r#"{{"kind":"contracts_birth","seen_new":{},"duplicate":{},"argc_new":{},"argc_birth":{},"argc_match":{},"key":{}}}"#,
            if seen_new { 1 } else { 0 },
            if duplicate { 1 } else { 0 },
            argc_new,
            argc_birth,
            if argc_new == argc_birth { 1 } else { 0 },
            key
        );
    }
}
