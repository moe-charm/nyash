use super::*;

impl MirInterpreter {
    pub(super) fn handle_extern_call(
        &mut self,
        dst: Option<ValueId>,
        iface: &str,
        method: &str,
        args: &[ValueId],
    ) -> Result<(), VMError> {
        match (iface, method) {
            ("env.console", "log") => {
                if let Some(a0) = args.get(0) {
                    let v = self.reg_load(*a0)?;
                    println!("{}", v.to_string());
                }
                if let Some(d) = dst {
                    self.regs.insert(d, VMValue::Void);
                }
                Ok(())
            }
            ("env.future", "new") => {
                let fut = crate::boxes::future::NyashFutureBox::new();
                if let Some(a0) = args.get(0) {
                    let v = self.reg_load(*a0)?;
                    fut.set_result(v.to_nyash_box());
                }
                if let Some(d) = dst {
                    self.regs.insert(d, VMValue::Future(fut));
                }
                Ok(())
            }
            ("env.future", "set") => {
                if args.len() >= 2 {
                    let f = self.reg_load(args[0])?;
                    let v = self.reg_load(args[1])?;
                    if let VMValue::Future(fut) = f {
                        fut.set_result(v.to_nyash_box());
                    } else {
                        return Err(VMError::TypeError("env.future.set expects Future".into()));
                    }
                }
                if let Some(d) = dst {
                    self.regs.insert(d, VMValue::Void);
                }
                Ok(())
            }
            ("env.future", "await") => {
                if let Some(a0) = args.get(0) {
                    let f = self.reg_load(*a0)?;
                    match f {
                        VMValue::Future(fut) => {
                            let v = fut.get();
                            if let Some(d) = dst {
                                self.regs.insert(d, VMValue::from_nyash_box(v));
                            }
                        }
                        _ => {
                            return Err(VMError::TypeError("await expects Future".into()));
                        }
                    }
                }
                Ok(())
            }
            ("env.runtime", "checkpoint") => {
                crate::runtime::global_hooks::safepoint_and_poll();
                if let Some(d) = dst {
                    self.regs.insert(d, VMValue::Void);
                }
                Ok(())
            }
            ("env.modules", "set") => {
                if args.len() >= 2 {
                    let k = self.reg_load(args[0])?.to_string();
                    let v = self.reg_load(args[1])?.to_nyash_box();
                    crate::runtime::modules_registry::set(k, v);
                }
                if let Some(d) = dst {
                    self.regs.insert(d, VMValue::Void);
                }
                Ok(())
            }
            ("env.modules", "get") => {
                if let Some(a0) = args.get(0) {
                    let k = self.reg_load(*a0)?.to_string();
                    let vb = crate::runtime::modules_registry::get(&k)
                        .unwrap_or_else(|| Box::new(crate::box_trait::VoidBox::new()));
                    if let Some(d) = dst {
                        self.regs.insert(d, VMValue::from_nyash_box(vb));
                    }
                }
                Ok(())
            }
            _ => Err(VMError::InvalidInstruction(format!(
                "ExternCall {}.{} not supported",
                iface, method
            ))),
        }
    }
}
