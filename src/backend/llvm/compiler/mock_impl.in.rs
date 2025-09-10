impl LLVMCompiler {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            values: HashMap::new(),
        })
    }

    pub fn compile_module(&self, mir_module: &MirModule, output_path: &str) -> Result<(), String> {
        // Mock implementation - in a real scenario this would:
        // 1. Create LLVM context and module
        // 2. Convert MIR instructions to LLVM IR
        // 3. Generate object file

        println!("🔧 Mock LLVM Compilation:");
        println!("   Module: {}", mir_module.name);
        println!("   Functions: {}", mir_module.functions.len());
        println!("   Output: {}", output_path);

        // Find entry function (prefer is_entry_point, then Main.main, then main, else first)
        let main_func = if let Some((_n, f)) = mir_module
            .functions
            .iter()
            .find(|(_n, f)| f.metadata.is_entry_point)
        {
            f
        } else if let Some(f) = mir_module.functions.get("Main.main") {
            f
        } else if let Some(f) = mir_module.functions.get("main") {
            f
        } else if let Some((_n, f)) = mir_module.functions.iter().next() {
            f
        } else {
            return Err("Main.main function not found");
        };

        println!(
            "   Main function found with {} blocks",
            main_func.blocks.len()
        );

        // Simulate object file generation
        std::fs::write(output_path, b"Mock object file")?;
        println!("   ✅ Mock object file created");

        Ok(())
    }

    pub fn compile_and_execute(
        &mut self,
        mir_module: &MirModule,
        temp_path: &str,
    ) -> Result<Box<dyn NyashBox>, String> {
        // Mock implementation - interprets MIR instructions to simulate execution

        eprintln!("⚠️⚠️⚠️ WARNING: Using MOCK LLVM Implementation! ⚠️⚠️⚠️");
        eprintln!("⚠️ This is NOT real LLVM execution!");
        eprintln!("⚠️ Build with --features llvm for real compilation!");
        println!("🚀 Mock LLVM Compile & Execute (MIR Interpreter Mode):");

        // 1. Mock object file generation
        let obj_path = format!("{}.o", temp_path);
        self.compile_module(mir_module, &obj_path)?;

        // 2. Find and execute main function
        let main_func = mir_module
            .functions
            .get("Main.main")
            .ok_or("Main.main function not found")?;

        println!("   ⚡ Interpreting MIR instructions...");

        // 3. Execute MIR instructions
        let result = self.interpret_function(main_func)?;

        // 4. Cleanup mock files
        let _ = std::fs::remove_file(&obj_path);

        Ok(result)
    }

    /// Interpret a MIR function by executing its instructions
    fn interpret_function(
        &mut self,
        func: &crate::mir::function::MirFunction,
    ) -> Result<Box<dyn NyashBox>, String> {
        // Clear value storage
        self.values.clear();

        // For now, just execute the entry block
        if let Some(entry_block) = func.blocks.get(&0) {
            for inst in &entry_block.instructions {
                match inst {
                    MirInstruction::Const { dst, value } => {
                        let nyash_value = match value {
                            ConstValue::Integer(i) => {
                                Box::new(IntegerBox::new(*i)) as Box<dyn NyashBox>
                            }
                            ConstValue::Float(f) => {
                                Box::new(FloatBox::new(*f)) as Box<dyn NyashBox>
                            }
                            ConstValue::String(s) => {
                                Box::new(StringBox::new(s.clone())) as Box<dyn NyashBox>
                            }
                            ConstValue::Bool(b) => Box::new(BoolBox::new(*b)) as Box<dyn NyashBox>,
                            ConstValue::Null => Box::new(NullBox::new()) as Box<dyn NyashBox>,
                        };
                        self.values.insert(*dst, nyash_value);
                        println!("   📝 %{} = const {:?}", dst.0, value);
                    }

                    MirInstruction::BinOp { dst, op, lhs, rhs } => {
                        // Get operands
                        let left = self
                            .values
                            .get(lhs)
                            .ok_or_else(|| format!("Value %{} not found", lhs.0))?;
                        let right = self
                            .values
                            .get(rhs)
                            .ok_or_else(|| format!("Value %{} not found", rhs.0))?;

                        // Simple integer arithmetic for now
                        if let (Some(l), Some(r)) = (
                            left.as_any().downcast_ref::<IntegerBox>(),
                            right.as_any().downcast_ref::<IntegerBox>(),
                        ) {
                            let result = match op {
                                BinaryOp::Add => l.value + r.value,
                                BinaryOp::Sub => l.value - r.value,
                                BinaryOp::Mul => l.value * r.value,
                                BinaryOp::Div => {
                                    if r.value == 0 {
                                        return Err("Division by zero".to_string());
                                    }
                                    l.value / r.value
                                }
                                BinaryOp::Mod => l.value % r.value,
                                _ => {
                                    return Err(
                                        "Binary operation not supported in mock".to_string()
                                    );
                                }
                            };
                            self.values.insert(*dst, Box::new(IntegerBox::new(result)));
                            println!(
                                "   📊 %{} = %{} {:?} %{} = {}",
                                dst.0, lhs.0, op, rhs.0, result
                            );
                        } else {
                            return Err(
                                "Binary operation on non-integer values not supported in mock"
                                    .to_string(),
                            );
                        }
                    }

                    MirInstruction::Return { value } => {
                        if let Some(val_id) = value {
                            let result = self
                                .values
                                .get(val_id)
                                .ok_or_else(|| format!("Return value %{} not found", val_id.0))?
                                .clone_box();
                            println!("   ✅ Returning value from %{}", val_id.0);
                            return Ok(result);
                        } else {
                            println!("   ✅ Void return");
                            return Ok(Box::new(IntegerBox::new(0)));
                        }
                    }

                    MirInstruction::FunctionNew {
                        dst,
                        params,
                        body,
                        captures,
                        me,
                    } => {
                        // Minimal: build FunctionBox with empty captures unless provided
                        let mut env = crate::boxes::function_box::ClosureEnv::new();
                        // Materialize captures (by value) if any
                        for (name, vid) in captures.iter() {
                            let v = self.values.get(vid).ok_or_else(|| {
                                format!("Value %{} not found for capture {}", vid.0, name)
                            })?;
                            env.captures.insert(name.clone(), v.clone_box());
                        }
                        // me capture (weak) if provided and is a box
                        if let Some(m) = me {
                            if let Some(b) = self.values.get(m) {
                                if let Some(arc) = std::sync::Arc::downcast::<dyn NyashBox>({
                                    let bx: std::sync::Arc<dyn NyashBox> =
                                        std::sync::Arc::from(b.clone_box());
                                    bx
                                })
                                .ok()
                                {
                                    env.me_value = Some(std::sync::Arc::downgrade(&arc));
                                }
                            }
                        }
                        let fun = FunctionBox::with_env(params.clone(), body.clone(), env);
                        self.values.insert(*dst, Box::new(fun));
                        println!("   🧰 %{} = function_new (params={})", dst.0, params.len());
                    }

                    MirInstruction::Call {
                        dst, func, args, ..
                    } => {
                        // Resolve callee
                        let cal = self
                            .values
                            .get(func)
                            .ok_or_else(|| format!("Call target %{} not found", func.0))?;
                        if let Some(fb) = cal.as_any().downcast_ref::<FunctionBox>() {
                            // Collect args as NyashBox
                            let mut argv: Vec<Box<dyn NyashBox>> = Vec::new();
                            for a in args {
                                let av = self
                                    .values
                                    .get(a)
                                    .ok_or_else(|| format!("Arg %{} not found", a.0))?;
                                argv.push(av.clone_box());
                            }
                            let out = crate::interpreter::run_function_box(fb, argv)
                                .map_err(|e| format!("FunctionBox call failed: {:?}", e))?;
                            if let Some(d) = dst {
                                self.values.insert(*d, out);
                            }
                            println!(
                                "   📞 call %{} -> {}",
                                func.0,
                                dst.map(|v| v.0).unwrap_or(u32::MAX)
                            );
                        } else {
                            println!("   ⚠️  Skipping call: callee not FunctionBox");
                        }
                    }

                    _ => {
                        // Other instructions not yet implemented
                        println!("   ⚠️  Skipping instruction: {:?}", inst);
                    }
                }
            }
        }

        // Default return
        Ok(Box::new(IntegerBox::new(0)))
    }
}
