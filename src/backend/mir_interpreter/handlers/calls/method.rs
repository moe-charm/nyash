//! method.rs — Method dispatch (Router single entry)
//!
//! Route all method calls through MethodRouterBox. Legacy branches were removed.

use super::super::*;

impl MirInterpreter {
    pub(crate) fn execute_method_call(
        &mut self,
        receiver: &VMValue,
        method: &str,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        // DEBUG: Trace method_call entry
        if std::env::var("HAKO_DEBUG_BOXCALL_ARGV").is_ok() {
            eprintln!("[METHOD-CALL] method={} recv={:?} args_len={}",
                      method,
                      match receiver {
                          VMValue::String(s) => format!("String(\"{}\")", if s.len() > 20 { &s[..20] } else { s }),
                          VMValue::Integer(i) => format!("Integer({})", i),
                          VMValue::BoxRef(b) => format!("BoxRef({})", b.type_name()),
                          VMValue::Void => "Void".into(),
                          _ => "Other".into(),
                      },
                      args.len());
        }

        // Single-entry routing: convert args to VMValue and delegate to Router.
        let mut argv_vals: Vec<VMValue> = Vec::with_capacity(args.len());
        for a in args { argv_vals.push(self.reg_load(*a)?); }
        crate::runtime::method_router_box::route(self, receiver, method, &argv_vals)
    }
}
