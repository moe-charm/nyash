use super::super::NyashRunner;
use nyash_rust::{parser::NyashParser, interpreter::NyashInterpreter, box_factory::builtin::BuiltinGroups, mir::MirCompiler, backend::VM};

impl NyashRunner {
    /// Execute benchmark mode (split)
    pub(crate) fn execute_benchmark_mode(&self) {
        println!("🏁 Running benchmark mode with {} iterations", self.config.iterations);
        // Two tests: simple add, arithmetic loop
        let tests: Vec<(&str, &str)> = vec![
            (
                "simple_add",
                r#"
                local x
                x = 42
                local y 
                y = x + 58
                return y
                "#,
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
            ),
        ];

        for (name, code) in tests {
            println!("\n====================================");
            println!("🧪 Test: {}", name);
            // Warmup (not measured)
            let warmup = (self.config.iterations / 10).max(1);
            self.bench_interpreter(code, warmup);
            self.bench_vm(code, warmup);
            self.bench_jit(code, warmup);

            // Measured runs
            let interpreter_time = self.bench_interpreter(code, self.config.iterations);
            let vm_time = self.bench_vm(code, self.config.iterations);
            let jit_time = self.bench_jit(code, self.config.iterations);

            // Summary
            let vm_vs_interp = interpreter_time.as_secs_f64() / vm_time.as_secs_f64();
            let jit_vs_vm = vm_time.as_secs_f64() / jit_time.as_secs_f64();
            println!("\n📊 Performance Summary [{}]:", name);
            println!("  VM is {:.2}x {} than Interpreter", if vm_vs_interp > 1.0 { vm_vs_interp } else { 1.0 / vm_vs_interp }, if vm_vs_interp > 1.0 { "faster" } else { "slower" });
            println!("  JIT is {:.2}x {} than VM (note: compile cost included)", if jit_vs_vm > 1.0 { jit_vs_vm } else { 1.0 / jit_vs_vm }, if jit_vs_vm > 1.0 { "faster" } else { "slower" });
        }
    }

    fn bench_interpreter(&self, code: &str, iters: u32) -> std::time::Duration {
        let start = std::time::Instant::now();
        for _ in 0..iters {
            if let Ok(ast) = NyashParser::parse_from_string(code) {
                let mut interp = NyashInterpreter::new_with_groups(BuiltinGroups::native_full());
                let _ = interp.execute(ast);
            }
        }
        let elapsed = start.elapsed();
        println!("  ⚡ Interpreter: {} iters in {:?} ({:.2} ops/sec)", iters, elapsed, iters as f64 / elapsed.as_secs_f64());
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
        println!("  🚀 VM:           {} iters in {:?} ({:.2} ops/sec)", iters, elapsed, iters as f64 / elapsed.as_secs_f64());
        elapsed
    }

    fn bench_jit(&self, code: &str, iters: u32) -> std::time::Duration {
        // Force JIT mode for this run
        std::env::set_var("NYASH_JIT_EXEC", "1");
        std::env::set_var("NYASH_JIT_THRESHOLD", "1");
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
        println!("  🔥 JIT:         {} iters in {:?} ({:.2} ops/sec)", iters, elapsed, iters as f64 / elapsed.as_secs_f64());
        elapsed
    }
}
