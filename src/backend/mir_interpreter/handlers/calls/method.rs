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
        // Single-entry routing: convert args to VMValue and delegate to Router.
        let mut argv_vals: Vec<VMValue> = Vec::with_capacity(args.len());
        for a in args { argv_vals.push(self.reg_load(*a)?); }
        crate::runtime::method_router_box::route(self, receiver, method, &argv_vals)
    }
}
