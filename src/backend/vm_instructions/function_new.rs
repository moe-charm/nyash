use crate::mir::ValueId;
use std::sync::Arc;
use crate::backend::vm::ControlFlow;
use crate::backend::{VM, VMError, VMValue};

impl VM {
    /// Execute FunctionNew instruction (construct a FunctionBox value)
    pub(crate) fn execute_function_new(
        &mut self,
        dst: ValueId,
        params: &[String],
        body: &[crate::ast::ASTNode],
        captures: &[(String, ValueId)],
        me: &Option<ValueId>,
    ) -> Result<ControlFlow, VMError> {
        // Build ClosureEnv
        let mut env = crate::boxes::function_box::ClosureEnv::new();
        // Add captures by value
        for (name, vid) in captures.iter() {
            let v = self.get_value(*vid)?;
            env.captures.insert(name.clone(), v.to_nyash_box());
        }
        // Capture 'me' weakly if provided and is a BoxRef
        if let Some(m) = me {
            match self.get_value(*m)? {
                VMValue::BoxRef(b) => { env.me_value = Some(Arc::downgrade(&b)); }
                _ => {}
            }
        }
        let fun = crate::boxes::function_box::FunctionBox::with_env(params.to_vec(), body.to_vec(), env);
        self.set_value(dst, VMValue::BoxRef(Arc::new(fun)));
        Ok(ControlFlow::Continue)
    }
}
