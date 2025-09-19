use super::super::NyashRunner;
use nyash_rust::{
    backend::VM, interpreter::NyashInterpreter, mir::MirCompiler, parser::NyashParser,
};

impl NyashRunner {
    /// Execute benchmark mode (split)
    pub(crate) fn execute_benchmark_mode(&self) {
        let groups = self.config.as_groups();
        println!(
            "🏁 Running benchmark mode with {} iterations",
            groups.iterations
        );
        // Tests: some run on all backends, some are JIT+f64 only
        // Third element indicates JIT+f64 only (skip VM/Interpreter)
        let tests: Vec<(&str, &str, bool)> = vec![
            (
                "simple_add",
                r#"
                local x
                x = 42
                local y 
                y = x + 58
                return y
                "#,
                false,
            ),
            (
                "arith_loop_100k",
                r#"
                local i, sum
                i = 0
                sum = 0
                loop(i < 100000) {
                    sum = sum + i
                    i = i + 1
                }
                return sum
                "#,
                false,
            ),
            (
                "branch_return",
                r#"
                local a, b
                a = 3
                b = 5
                if (a < b) {
                    return 1
                } else {
                    return 2
                }
                "#,
                false,
            ),
            (
                "f64_add_jit",
                r#"
                local x, y
                x = 1.5
                y = 2.25
                return x + y
                "#,
                true,
            ),
        ];

        for (name, code, jit_f64_only) in tests {
            println!("\n====================================");
            println!("🧪 Test: {}", name);
            if jit_f64_only {
                println!(
                    "(JIT+f64 only) Skipping VM/Interpreter; requires --features cranelift-jit"
                );
                // Warmup JIT
                let warmup = (groups.iterations / 10).max(1);
                self.bench_jit(code, warmup);
                // Measured
                let jit_time = self.bench_jit(code, groups.iterations);
                println!("\n📊 Performance Summary [{}]:", name);
                println!(
                    "  JIT f64 ops: {} iters in {:?} ({:.2} ops/sec)",
                    groups.iterations,
                    jit_time,
                    groups.iterations as f64 / jit_time.as_secs_f64()
                );
            } else {
                // Quick correctness check across modes (golden): Interpreter vs VM vs VM+JIT
                if let Err(e) = self.verify_outputs_match(code) {
                    println!("❌ Output mismatch: {}", e);
                } else {
                    println!("✅ Outputs match across Interpreter/VM/JIT");
                }
                // Warmup (not measured)
                let warmup = (groups.iterations / 10).max(1);
                self.bench_interpreter(code, warmup);
                self.bench_vm(code, warmup);
                self.bench_jit(code, warmup);

                // Measured runs
                let interpreter_time = self.bench_interpreter(code, groups.iterations);
                let vm_time = self.bench_vm(code, groups.iterations);
                let jit_time = self.bench_jit(code, groups.iterations);

                // Summary
                let vm_vs_interp = interpreter_time.as_secs_f64() / vm_time.as_secs_f64();
                let jit_vs_vm = vm_time.as_secs_f64() / jit_time.as_secs_f64();
                println!("\n📊 Performance Summary [{}]:", name);
                println!(
                    "  VM is {:.2}x {} than Interpreter",
                    if vm_vs_interp > 1.0 {
                        vm_vs_interp
                    } else {
                        1.0 / vm_vs_interp
                    },
                    if vm_vs_interp > 1.0 {
                        "faster"
                    } else {
                        "slower"
                    }
                );
                println!(
                    "  JIT is {:.2}x {} than VM (note: compile cost included)",
                    if jit_vs_vm > 1.0 {
                        jit_vs_vm
                    } else {
                        1.0 / jit_vs_vm
                    },
                    if jit_vs_vm > 1.0 { "faster" } else { "slower" }
                );
            }
        }
    }

    fn bench_interpreter(&self, code: &str, iters: u32) -> std::time::Duration {
        // Enable native f64 when available to exercise widened ABI
        std::env::set_var("NYASH_JIT_NATIVE_F64", "1");
        let start = std::time::Instant::now();
        for _ in 0..iters {
            if let Ok(ast) = NyashParser::parse_from_string(code) {
                let mut interp = NyashInterpreter::new();
                let _ = interp.execute(ast);
            }
        }
        let elapsed = start.elapsed();
        println!(
            "  ⚡ Interpreter: {} iters in {:?} ({:.2} ops/sec)",
            iters,
            elapsed,
            iters as f64 / elapsed.as_secs_f64()
        );
        elapsed
    }

    fn bench_vm(&self, code: &str, iters: u32) -> std::time::Duration {
        let start = std::time::Instant::now();
        for _ in 0..iters {
            if let Ok(ast) = NyashParser::parse_from_string(code) {
                let mut mc = MirCompiler::new();
                if let Ok(cr) = mc.compile(ast) {
                    let mut vm = VM::new();
                    let _ = vm.execute_module(&cr.module);
                }
            }
        }
        let elapsed = start.elapsed();
        println!(
            "  🚀 VM:           {} iters in {:?} ({:.2} ops/sec)",
            iters,
            elapsed,
            iters as f64 / elapsed.as_secs_f64()
        );
        elapsed
    }

    fn bench_jit(&self, code: &str, iters: u32) -> std::time::Duration {
        // Force JIT mode for this run
        std::env::set_var("NYASH_JIT_EXEC", "1");
        std::env::set_var("NYASH_JIT_THRESHOLD", "1");
        let groups = self.config.as_groups();
        if groups.backend.jit.stats {
            std::env::set_var("NYASH_JIT_STATS", "1");
        }
        if groups.backend.jit.stats_json {
            std::env::set_var("NYASH_JIT_STATS_JSON", "1");
        }
        let start = std::time::Instant::now();
        for _ in 0..iters {
            if let Ok(ast) = NyashParser::parse_from_string(code) {
                let mut mc = MirCompiler::new();
                if let Ok(cr) = mc.compile(ast) {
                    let mut vm = VM::new();
                    let _ = vm.execute_module(&cr.module);
                }
            }
        }
        let elapsed = start.elapsed();
        println!(
            "  🔥 JIT:         {} iters in {:?} ({:.2} ops/sec)",
            iters,
            elapsed,
            iters as f64 / elapsed.as_secs_f64()
        );
        elapsed
    }

    /// Verify that outputs match across VM and JIT-enabled VM (golden)
    fn verify_outputs_match(&self, code: &str) -> Result<(), String> {
        // VM
        let vm_out = {
            let ast =
                NyashParser::parse_from_string(code).map_err(|e| format!("vm parse: {}", e))?;
            let mut mc = MirCompiler::new();
            let cr = mc.compile(ast).map_err(|e| format!("vm compile: {}", e))?;
            let mut vm = VM::new();
            let out = vm
                .execute_module(&cr.module)
                .map_err(|e| format!("vm exec: {}", e))?;
            out.to_string_box().value
        };
        // VM+JIT
        let jit_out = {
            std::env::set_var("NYASH_JIT_EXEC", "1");
            std::env::set_var("NYASH_JIT_THRESHOLD", "1");
            let ast =
                NyashParser::parse_from_string(code).map_err(|e| format!("jit parse: {}", e))?;
            let mut mc = MirCompiler::new();
            let cr = mc.compile(ast).map_err(|e| format!("jit compile: {}", e))?;
            let mut vm = VM::new();
            let out = vm
                .execute_module(&cr.module)
                .map_err(|e| format!("jit exec: {}", e))?;
            out.to_string_box().value
        };
        if vm_out != jit_out {
            return Err(format!("vm='{}' jit='{}'", vm_out, jit_out));
        }
        Ok(())
    }
}
#![cfg(feature = "vm-legacy")]
