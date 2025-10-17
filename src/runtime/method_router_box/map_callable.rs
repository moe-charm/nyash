//! Map callable sugar routing.
//!
//! Encapsulates the VM-side desugaring for Map.call / Map.callAsync so the
//! primary router stays small and the behaviour is easier to extend.

use crate::backend::mir_interpreter::MirInterpreter;
use crate::backend::vm_types::{VMError, VMValue};
use crate::runtime::meta::callable::callable_box::CallableBox;
use crate::runtime::plugin_loader_v2::PluginBoxV2;

pub struct MapCallableBox;

impl MapCallableBox {
    /// Handles `MapBox.call` and `MapBox.callAsync` whenever the receiver is a
    /// plugin-backed MapBox. Returns `None` for unrelated invocations.
    #[allow(unused_variables)]
    pub fn try_route(
        interp: &mut MirInterpreter,
        receiver: &VMValue,
        method: &str,
        args: &[VMValue],
    ) -> Option<Result<VMValue, VMError>> {
        if !(method == "call" || method == "callAsync") {
            return None;
        }

        let plugin_box = match receiver {
            VMValue::BoxRef(bx) => bx
                .as_any()
                .downcast_ref::<PluginBoxV2>()
                .filter(|p| p.box_type.as_str() == "MapBox"),
            _ => None,
        }?;

        Some(Self::invoke(interp, plugin_box, method, args))
    }

    #[allow(unused_variables)]
    fn invoke(
        interp: &mut MirInterpreter,
        plugin_box: &PluginBoxV2,
        method: &str,
        args: &[VMValue],
    ) -> Result<VMValue, VMError> {
        if args.len() != 2 {
            return Err(VMError::InvalidInstruction(crate::common::diagnostics::msg::no_method_arity(
                "MapBox", method, args.len(), &[2]
            )));
        }

        let key = args[0].to_nyash_box();
        let value = crate::runtime::plugin_host_box::invoke_instance_method(
            &plugin_box.box_type,
            "get",
            plugin_box.inner.instance_id,
            &vec![key],
        )
        .map_err(|err| VMError::InvalidInstruction(format!("Map.{} get failed: {:?}", method, err)))?;

        let val = match value {
            Some(ret) => VMValue::from_nyash_box(ret),
            None => VMValue::Void,
        };
        if let VMValue::Void = val {
            // P1 spec: missing or nullish value → null
            return Ok(VMValue::Void);
        }

        let callable_arc = match val {
            VMValue::BoxRef(bx) => bx,
            _ => {
                return Err(VMError::InvalidInstruction(
                    "Map.call: value is not CallableBox".into(),
                ))
            }
        };

        let callable = callable_arc
            .as_ref()
            .as_any()
            .downcast_ref::<CallableBox>()
            .ok_or_else(|| VMError::InvalidInstruction("Map.call: value is not CallableBox".into()))?;

        {
            if callable.receiver.is_none() {
                return Err(VMError::InvalidInstruction(
                    "CallableBox without receiver".into(),
                ));
            }

            let call_args = vec![args[1].clone()];
            if method == "callAsync" {
                super::route(interp, &VMValue::BoxRef(callable_arc), "callAsync", &call_args)
            } else {
                super::route(interp, &VMValue::BoxRef(callable_arc), "call", &call_args)
            }
        }
    }
}
