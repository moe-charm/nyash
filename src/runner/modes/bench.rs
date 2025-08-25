use super::super::NyashRunner;
use nyash_rust::{parser::NyashParser, interpreter::NyashInterpreter, box_factory::builtin::BuiltinGroups, mir::MirCompiler, backend::VM};

impl NyashRunner {
    /// Execute benchmark mode (split)
    pub(crate) fn execute_benchmark_mode(&self) {
        println!("🏁 Running benchmark mode with {} iterations", self.config.iterations);
        let test_code = r#"
        local x
        x = 42
        local y 
        y = x + 58
        return y
        "#;

        println!("\n🧪 Test code:\n{}", test_code);

        // Interpreter
        println!("\n⚡ Interpreter Backend:");
        let start = std::time::Instant::now();
        for _ in 0..self.config.iterations {
            if let Ok(ast) = NyashParser::parse_from_string(test_code) {
                let mut interp = NyashInterpreter::new_with_groups(BuiltinGroups::native_full());
                let _ = interp.execute(ast);
            }
        }
        let interpreter_time = start.elapsed();
        println!("  {} iterations in {:?} ({:.2} ops/sec)", self.config.iterations, interpreter_time, self.config.iterations as f64 / interpreter_time.as_secs_f64());

        // VM
        println!("\n🚀 VM Backend:");
        let start = std::time::Instant::now();
        for _ in 0..self.config.iterations {
            if let Ok(ast) = NyashParser::parse_from_string(test_code) {
                let mut mc = MirCompiler::new();
                if let Ok(cr) = mc.compile(ast) {
                    let mut vm = VM::new();
                    let _ = vm.execute_module(&cr.module);
                }
            }
        }
        let vm_time = start.elapsed();
        println!("  {} iterations in {:?} ({:.2} ops/sec)", self.config.iterations, vm_time, self.config.iterations as f64 / vm_time.as_secs_f64());

        // Summary
        let speedup = interpreter_time.as_secs_f64() / vm_time.as_secs_f64();
        println!("\n📊 Performance Summary:\n  VM is {:.2}x {} than Interpreter", if speedup > 1.0 { speedup } else { 1.0 / speedup }, if speedup > 1.0 { "faster" } else { "slower" });
    }
}

