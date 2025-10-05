use super::*;
// Removed unused time imports

impl MirInterpreter {
    pub(super) fn handle_extern_call(
        &mut self,
        dst: Option<ValueId>,
        iface: &str,
        method: &str,
        args: &[ValueId],
    ) -> Result<(), VMError> {
        // VM Adapter 経由の疎結合呼び出し（既定ON）。未知は従来のmatchにフォールバック。
        // 先に args を VMValue に読み出しておき、adapter に渡す。
        let mut loaded_args: Vec<crate::backend::vm_types::VMValue> = Vec::with_capacity(args.len());
        for a in args {
            loaded_args.push(self.reg_load(*a)?);
        }
        if let Some(res) = crate::backend::mir_interpreter::extern_adapter::try_call(iface, method, &loaded_args) {
            let v = res?;
            self.maybe_register_scope_value(&v);
            if let Some(d) = dst { self.regs.insert(d, v); }
            return Ok(());
        }
        match (iface, method) {
            ("env.console", "log") => {
                if let Some(a0) = args.get(0) {
                    let v = self.reg_load(*a0)?;
                    // Dev-only: mirror print-trace for extern console.log
                    if Self::print_trace_enabled() { self.print_trace_emit(&v); }
                    // Treat VM Void and BoxRef(VoidBox) as JSON null for dev ergonomics
                    match &v {
                        VMValue::Void => { println!("null"); if let Some(d) = dst { self.regs.insert(d, VMValue::Void); } return Ok(()); }
                        VMValue::BoxRef(bx) => {
                            if bx.as_any().downcast_ref::<crate::box_trait::VoidBox>().is_some() {
                                println!("null"); if let Some(d) = dst { self.regs.insert(d, VMValue::Void); } return Ok(());
                            }
                            if let Some(sb) = bx.as_any().downcast_ref::<crate::box_trait::StringBox>() {
                                println!("{}", sb.value); if let Some(d) = dst { self.regs.insert(d, VMValue::Void); } return Ok(());
                            }
                        }
                        VMValue::String(s) => { println!("{}", s); if let Some(d) = dst { self.regs.insert(d, VMValue::Void); } return Ok(()); }
                        _ => {}
                    }
                    // Operator Box (Stringify) – dev flag gated
                    if std::env::var("NYASH_OPERATOR_BOX_STRINGIFY").ok().as_deref() == Some("1") {
                        if let Some(op) = self.functions.get("StringifyOperator.apply/1").cloned() {
                            let out = self.exec_function_inner(&op, Some(&[v.clone()]))?;
                            println!("{}", out.to_string());
                        } else {
                            println!("{}", v.to_string());
                        }
                    } else {
                        println!("{}", v.to_string());
                    }
                }
                if let Some(d) = dst { self.regs.insert(d, VMValue::Void); }
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
                            let vmv = VMValue::from_nyash_box(v);
                            self.maybe_register_scope_value(&vmv);
                            if let Some(d) = dst {
                                self.regs.insert(d, vmv);
                            }
                        }
                        _ => {
                            return Err(VMError::TypeError("await expects Future".into()));
                        }
                    }
                }
                Ok(())
            }
            // Core externs are handled via VM ExternAdapter; unknown fall through
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
                    let vmv = VMValue::from_nyash_box(vb);
                    self.maybe_register_scope_value(&vmv);
                    if let Some(d) = dst {
                        self.regs.insert(d, vmv);
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
