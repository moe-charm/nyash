// extern_future_legacy.rs — env.future.* handlers (legacy-boxes gated)
use std::collections::HashMap;

use crate::backend::vm_types::{VMError, VMValue};

pub fn register(map: &mut HashMap<(String, String), super::HandlerFn>) {
    #[cfg(feature = "legacy-boxes")]
    {
        fn value_as_future(value: &VMValue) -> Option<crate::boxes::future::FutureBox> {
            match value {
                VMValue::BoxRef(b) => b.as_any().downcast_ref::<crate::boxes::future::FutureBox>().cloned(),
                _ => None,
            }
        }
        // env.future.new() -> Future
        map.insert(("env.future".into(), "new".into()), |_args: &[VMValue]| {
            let fut = crate::boxes::future::FutureBox::new();
            crate::runtime::global_hooks::register_future_to_current_group(&fut);
            Ok(VMValue::from_nyash_box(Box::new(fut)))
        });
        // env.future.set(future, value) -> Void
        map.insert(("env.future".into(), "set".into()), |args: &[VMValue]| {
            if args.len() < 2 { return Err(VMError::InvalidInstruction("env.future.set requires 2 args".into())); }
            if let Some(fut) = value_as_future(&args[0]) {
                fut.set_result(args[1].to_nyash_box());
                Ok(VMValue::Void)
            } else {
                Err(VMError::TypeError("env.future.set expects Future".into()))
            }
        });
        // env.future.await(future) -> any
        map.insert(("env.future".into(), "await".into()), |args: &[VMValue]| {
            let first = args.get(0).ok_or_else(|| VMError::InvalidInstruction("env.future.await requires 1 arg".into()))?;
            if let Some(fut) = value_as_future(first) {
                // block until ready and return value
                match fut.wait_and_get() {
                    Ok(v) => Ok(VMValue::from_nyash_box(v)),
                    Err(e) => Err(VMError::InvalidInstruction(format!("future.await error: {}", e))),
                }
            } else {
                Err(VMError::TypeError("env.future.await expects Future".into()))
            }
        });
        // env.future.spawn_instance(callable, argvArray) -> Future
        map.insert(("env.future".into(), "spawn_instance".into()), |args: &[VMValue]| {
            let fut = crate::boxes::future::FutureBox::new();
            crate::runtime::global_hooks::register_future_to_current_group(&fut);
            let cb = args.get(0).cloned().unwrap_or(VMValue::Void);
            let argv = args.get(1).cloned().unwrap_or(VMValue::Void);
            let fut_clone = fut.clone();
            let name = "env.future.spawn_instance".to_string();
            let _scheduled = crate::runtime::global_hooks::spawn_task(&name, Box::new(move || {
                // Interpret callable in a fresh interpreter to avoid sharing state
                let mut vm = crate::backend::mir_interpreter::MirInterpreter::new();
                let out = match cb {
                    VMValue::BoxRef(bx) => crate::runtime::method_router_box::route(&mut vm, &VMValue::BoxRef(bx), "call", &match argv {
                        VMValue::BoxRef(_) => vec![argv.clone()],
                        _ => vec![],
                    }),
                    _ => Err(VMError::InvalidInstruction("spawn_instance expects callable".into())),
                };
                match out { Ok(v) => fut_clone.set_result(v.to_nyash_box()), Err(_) => fut_clone.set_result(Box::new(crate::box_trait::StringBox::new("invoke_failed"))) }
            }));
            Ok(VMValue::from_nyash_box(Box::new(fut)))
        });
    }
    #[cfg(not(feature = "legacy-boxes"))]
    {
        // Plugin-only builds: provide stable diagnostics instead of panicking
        let err = |_: &[VMValue]| Err(VMError::InvalidInstruction("Extern future disabled (legacy-only)".into()));
        map.insert(("env.future".into(), "new".into()), err);
        map.insert(("env.future".into(), "set".into()), err);
        map.insert(("env.future".into(), "await".into()), err);
        map.insert(("env.future".into(), "spawn_instance".into()), err);
    }
}
