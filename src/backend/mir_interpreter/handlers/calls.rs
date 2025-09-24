use super::*;

impl MirInterpreter {
    pub(super) fn handle_call(
        &mut self,
        dst: Option<ValueId>,
        func: ValueId,
        callee: Option<&Callee>,
        args: &[ValueId],
    ) -> Result<(), VMError> {
        let call_result = if let Some(callee_type) = callee {
            self.execute_callee_call(callee_type, args)?
        } else {
            self.execute_legacy_call(func, args)?
        };
        if let Some(d) = dst {
            self.regs.insert(d, call_result);
        }
        Ok(())
    }

    pub(super) fn execute_callee_call(
        &mut self,
        callee: &Callee,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        match callee {
            Callee::Global(func_name) => self.execute_global_function(func_name, args),
            Callee::Method {
                box_name: _,
                method,
                receiver,
            } => {
                if let Some(recv_id) = receiver {
                    let recv_val = self.reg_load(*recv_id)?;
                    self.execute_method_call(&recv_val, method, args)
                } else {
                    Err(VMError::InvalidInstruction(format!(
                        "Method call missing receiver for {}",
                        method
                    )))
                }
            }
            Callee::Constructor { box_type } => Err(VMError::InvalidInstruction(format!(
                "Constructor calls not yet implemented for {}",
                box_type
            ))),
            Callee::Closure { .. } => Err(VMError::InvalidInstruction(
                "Closure creation not yet implemented in VM".into(),
            )),
            Callee::Value(func_val_id) => {
                let _func_val = self.reg_load(*func_val_id)?;
                Err(VMError::InvalidInstruction(
                    "First-class function calls not yet implemented in VM".into(),
                ))
            }
            Callee::Extern(extern_name) => self.execute_extern_function(extern_name, args),
        }
    }

    pub(super) fn execute_legacy_call(
        &mut self,
        func_id: ValueId,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        let name_val = self.reg_load(func_id)?;
        let raw = match name_val {
            VMValue::String(ref s) => s.clone(),
            other => other.to_string(),
        };

        let mut pick: Option<String> = None;
        if self.functions.contains_key(&raw) {
            pick = Some(raw.clone());
        } else {
            let arity = args.len();
            let mut cands: Vec<String> = Vec::new();
            let suf = format!(".{}{}", raw, format!("/{}", arity));
            for k in self.functions.keys() {
                if k.ends_with(&suf) {
                    cands.push(k.clone());
                }
            }
            if cands.is_empty() && raw.contains('/') && self.functions.contains_key(&raw) {
                cands.push(raw.clone());
            }
            if cands.len() > 1 {
                if let Some(cur) = &self.cur_fn {
                    let cur_box = cur.split('.').next().unwrap_or("");
                    let scoped: Vec<String> = cands
                        .iter()
                        .filter(|k| k.starts_with(&format!("{}.", cur_box)))
                        .cloned()
                        .collect();
                    if scoped.len() == 1 {
                        cands = scoped;
                    }
                }
            }
            if cands.len() == 1 {
                pick = Some(cands.remove(0));
            } else if cands.len() > 1 {
                cands.sort();
                pick = Some(cands[0].clone());
            }
        }

        let fname = pick.ok_or_else(|| {
            VMError::InvalidInstruction(format!(
                "call unresolved: '{}' (arity={})",
                raw,
                args.len()
            ))
        })?;

        if std::env::var("NYASH_VM_CALL_TRACE").ok().as_deref() == Some("1") {
            eprintln!("[vm] legacy-call resolved '{}' -> '{}'", raw, fname);
        }

        let callee =
            self.functions.get(&fname).cloned().ok_or_else(|| {
                VMError::InvalidInstruction(format!("function not found: {}", fname))
            })?;

        let mut argv: Vec<VMValue> = Vec::new();
        for a in args {
            argv.push(self.reg_load(*a)?);
        }
        self.exec_function_inner(&callee, Some(&argv))
    }

    fn execute_global_function(
        &mut self,
        func_name: &str,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        match func_name {
            "nyash.builtin.print" | "print" | "nyash.console.log" => {
                if let Some(arg_id) = args.get(0) {
                    let val = self.reg_load(*arg_id)?;
                    println!("{}", val.to_string());
                }
                Ok(VMValue::Void)
            }
            "nyash.builtin.error" => {
                if let Some(arg_id) = args.get(0) {
                    let val = self.reg_load(*arg_id)?;
                    eprintln!("Error: {}", val.to_string());
                }
                Ok(VMValue::Void)
            }
            _ => Err(VMError::InvalidInstruction(format!(
                "Unknown global function: {}",
                func_name
            ))),
        }
    }

    fn execute_method_call(
        &mut self,
        receiver: &VMValue,
        method: &str,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        match receiver {
            VMValue::String(s) => match method {
                "length" => Ok(VMValue::Integer(s.len() as i64)),
                "concat" => {
                    if let Some(arg_id) = args.get(0) {
                        let arg_val = self.reg_load(*arg_id)?;
                        let new_str = format!("{}{}", s, arg_val.to_string());
                        Ok(VMValue::String(new_str))
                    } else {
                        Err(VMError::InvalidInstruction(
                            "concat requires 1 argument".into(),
                        ))
                    }
                }
                _ => Err(VMError::InvalidInstruction(format!(
                    "Unknown String method: {}",
                    method
                ))),
            },
            VMValue::BoxRef(box_ref) => {
                if let Some(p) = box_ref
                    .as_any()
                    .downcast_ref::<crate::runtime::plugin_loader_v2::PluginBoxV2>()
                {
                    let host = crate::runtime::plugin_loader_unified::get_global_plugin_host();
                    let host = host.read().unwrap();
                    let mut argv: Vec<Box<dyn crate::box_trait::NyashBox>> =
                        Vec::with_capacity(args.len());
                    for a in args {
                        argv.push(self.reg_load(*a)?.to_nyash_box());
                    }
                    match host.invoke_instance_method(
                        &p.box_type,
                        method,
                        p.inner.instance_id,
                        &argv,
                    ) {
                        Ok(Some(ret)) => Ok(VMValue::from_nyash_box(ret)),
                        Ok(None) => Ok(VMValue::Void),
                        Err(e) => Err(VMError::InvalidInstruction(format!(
                            "Plugin method {}.{} failed: {:?}",
                            p.box_type, method, e
                        ))),
                    }
                } else {
                    Err(VMError::InvalidInstruction(format!(
                        "Method {} not supported on BoxRef({})",
                        method,
                        box_ref.type_name()
                    )))
                }
            }
            _ => Err(VMError::InvalidInstruction(format!(
                "Method {} not supported on {:?}",
                method, receiver
            ))),
        }
    }

    fn execute_extern_function(
        &mut self,
        extern_name: &str,
        args: &[ValueId],
    ) -> Result<VMValue, VMError> {
        match extern_name {
            "exit" => {
                let code = if let Some(arg_id) = args.get(0) {
                    self.reg_load(*arg_id)?.as_integer().unwrap_or(0)
                } else {
                    0
                };
                std::process::exit(code as i32);
            }
            "panic" => {
                let msg = if let Some(arg_id) = args.get(0) {
                    self.reg_load(*arg_id)?.to_string()
                } else {
                    "VM panic".to_string()
                };
                panic!("{}", msg);
            }
            _ => Err(VMError::InvalidInstruction(format!(
                "Unknown extern function: {}",
                extern_name
            ))),
        }
    }
}
