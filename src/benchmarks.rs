/*!
 * Nyash Performance Benchmarks
 * 
 * Compare execution performance across different backends:
 * - Interpreter (AST direct execution)
 * - VM (MIR -> VM execution)  
 * - WASM (MIR -> WASM execution)
 */

use std::time::Instant;
use std::fs;
use crate::parser::NyashParser;
use crate::interpreter::NyashInterpreter;
use crate::mir::MirCompiler;
use crate::backend::{VM, WasmBackend};

#[derive(Debug)]
pub struct BenchmarkResult {
    pub name: String,
    pub backend: String,
    pub duration_ms: f64,
    pub iterations: u32,
    pub avg_duration_ms: f64,
}

pub struct BenchmarkSuite {
    iterations: u32,
}

impl BenchmarkSuite {
    pub fn new(iterations: u32) -> Self {
        Self { iterations }
    }
    
    /// Run comprehensive benchmark across all backends
    pub fn run_all(&self) -> Vec<BenchmarkResult> {
        let mut results = Vec::new();
        
        let benchmarks = [
            ("bench_light", "benchmarks/bench_light.nyash"),
            ("bench_medium", "benchmarks/bench_medium.nyash"), 
            ("bench_heavy", "benchmarks/bench_heavy.nyash"),
        ];
        
        for (name, file_path) in &benchmarks {
            println!("🚀 Running benchmark: {}", name);
            
            // Test if file exists and is readable
            if let Ok(source) = fs::read_to_string(file_path) {
                // Run on all backends
                if let Ok(interpreter_result) = self.run_interpreter_benchmark(name, &source) {
                    results.push(interpreter_result);
                }
                
                if let Ok(vm_result) = self.run_vm_benchmark(name, &source) {
                    results.push(vm_result);
                }
                
                if let Ok(wasm_result) = self.run_wasm_benchmark(name, &source) {
                    results.push(wasm_result);
                }
            } else {
                println!("⚠️ Could not read benchmark file: {}", file_path);
            }
        }
        
        results
    }
    
    /// Run benchmark on interpreter backend
    fn run_interpreter_benchmark(&self, name: &str, source: &str) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let mut total_duration = 0.0;
        
        for i in 0..self.iterations {
            let start = Instant::now();
            
            // Parse and execute
            let ast = NyashParser::parse_from_string(source)?;
            let mut interpreter = NyashInterpreter::new();
            let _result = interpreter.execute(ast)?;
            
            let duration = start.elapsed();
            total_duration += duration.as_secs_f64() * 1000.0; // Convert to ms
            
            if i == 0 {
                println!("  📊 Interpreter: First run completed");
            }
        }
        
        Ok(BenchmarkResult {
            name: name.to_string(),
            backend: "Interpreter".to_string(),
            duration_ms: total_duration,
            iterations: self.iterations,
            avg_duration_ms: total_duration / (self.iterations as f64),
        })
    }
    
    /// Run benchmark on VM backend
    fn run_vm_benchmark(&self, name: &str, source: &str) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let mut total_duration = 0.0;
        
        for i in 0..self.iterations {
            let start = Instant::now();
            
            // Parse -> MIR -> VM
            let ast = NyashParser::parse_from_string(source)?;
            let mut compiler = MirCompiler::new();
            let compile_result = compiler.compile(ast)?;
            let mut vm = VM::new();
            let _result = vm.execute_module(&compile_result.module)?;
            
            let duration = start.elapsed();
            total_duration += duration.as_secs_f64() * 1000.0; // Convert to ms
            
            if i == 0 {
                println!("  🏎️ VM: First run completed");
            }
        }
        
        Ok(BenchmarkResult {
            name: name.to_string(),
            backend: "VM".to_string(),
            duration_ms: total_duration,
            iterations: self.iterations,
            avg_duration_ms: total_duration / (self.iterations as f64),
        })
    }
    
    /// Run benchmark on WASM backend
    fn run_wasm_benchmark(&self, name: &str, source: &str) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let mut total_duration = 0.0;
        
        for i in 0..self.iterations {
            let start = Instant::now();
            
            // Parse -> MIR -> WASM
            let ast = NyashParser::parse_from_string(source)?;
            let mut compiler = MirCompiler::new();
            let compile_result = compiler.compile(ast)?;
            
            // Generate and execute WASM
            let mut wasm_backend = WasmBackend::new();
            let _wat_output = wasm_backend.compile_module(compile_result.module)?;
            // Note: For now we only measure compilation time
            // Full WASM execution would require wasmtime integration
            
            let duration = start.elapsed();
            total_duration += duration.as_secs_f64() * 1000.0; // Convert to ms
            
            if i == 0 {
                println!("  🌐 WASM: First run completed (compilation only)");
            }
        }
        
        Ok(BenchmarkResult {
            name: name.to_string(),
            backend: "WASM".to_string(),
            duration_ms: total_duration,
            iterations: self.iterations,
            avg_duration_ms: total_duration / (self.iterations as f64),
        })
    }
    
    /// Print benchmark results in a nice format
    pub fn print_results(&self, results: &[BenchmarkResult]) {
        println!("\n📊 Nyash Performance Benchmark Results");
        println!("=====================================");
        println!("Iterations per test: {}", self.iterations);
        println!();
        
        // Group by benchmark name
        let mut benchmarks: std::collections::HashMap<String, Vec<&BenchmarkResult>> = std::collections::HashMap::new();
        for result in results {
            benchmarks.entry(result.name.clone()).or_insert_with(Vec::new).push(result);
        }
        
        for (bench_name, bench_results) in benchmarks {
            println!("🎯 {}", bench_name);
            println!("  Backend       | Avg Time (ms) | Total Time (ms) | Speed Ratio");
            println!("  --------------|---------------|-----------------|------------");
            
            // Find fastest for ratio calculation
            let fastest = bench_results.iter().min_by(|a, b| a.avg_duration_ms.partial_cmp(&b.avg_duration_ms).unwrap()).unwrap();
            
            for result in &bench_results {
                let ratio = result.avg_duration_ms / fastest.avg_duration_ms;
                println!("  {:12} | {:11.3} | {:13.1} | {:8.2}x", 
                    result.backend, 
                    result.avg_duration_ms,
                    result.duration_ms,
                    ratio
                );
            }
            println!();
        }
        
        // Summary
        println!("💡 Performance Summary:");
        let interpreter_avg: f64 = results.iter()
            .filter(|r| r.backend == "Interpreter")
            .map(|r| r.avg_duration_ms)
            .sum::<f64>() / results.iter().filter(|r| r.backend == "Interpreter").count() as f64;
            
        let vm_avg: f64 = results.iter()
            .filter(|r| r.backend == "VM")
            .map(|r| r.avg_duration_ms)
            .sum::<f64>() / results.iter().filter(|r| r.backend == "VM").count() as f64;
            
        let wasm_avg: f64 = results.iter()
            .filter(|r| r.backend == "WASM")
            .map(|r| r.avg_duration_ms)
            .sum::<f64>() / results.iter().filter(|r| r.backend == "WASM").count() as f64;
        
        println!("  📈 Average across all benchmarks:");
        println!("     Interpreter: {:.2} ms", interpreter_avg);
        println!("     VM:          {:.2} ms ({:.1}x faster than interpreter)", vm_avg, interpreter_avg / vm_avg);
        println!("     WASM:        {:.2} ms ({:.1}x faster than interpreter)", wasm_avg, interpreter_avg / wasm_avg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_light() {
        let suite = BenchmarkSuite::new(3); // Only 3 iterations for testing
        let results = suite.run_all();
        
        // Should have results for all backends
        assert!(results.len() >= 3); // At least one benchmark with 3 backends
        
        suite.print_results(&results);
    }
}