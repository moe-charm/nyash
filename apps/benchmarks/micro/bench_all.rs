// Rust版ベンチマーク - すべてまとめて測定
// 同一プロセスで実行して正確な測定
// 固定時間測定方式（5秒間実行）

use std::time::{Duration, Instant};

// Fibonacci(12) - 再帰版
fn fib(n: i64) -> i64 {
    if n <= 1 {
        return n;
    }
    fib(n - 1) + fib(n - 2)
}

// PHI Stress If - 8方向分岐
fn phi_stress_if() -> i64 {
    let mut x = 0i64;
    let mut i = 0i64;

    while i < 10000 {
        if i % 8 == 0 {
            x = x + 1;
        } else if i % 8 == 1 {
            x = x + 2;
        } else if i % 8 == 2 {
            x = x + 3;
        } else if i % 8 == 3 {
            x = x + 4;
        } else if i % 8 == 4 {
            x = x + 5;
        } else if i % 8 == 5 {
            x = x + 6;
        } else if i % 8 == 6 {
            x = x + 7;
        } else {
            x = x + 8;
        }

        i = i + 1;
    }

    x
}

// PHI Stress Loop - ネスト3段階
fn phi_stress_loop() -> i64 {
    let mut sum = 0i64;
    let mut i = 0i64;

    while i < 50 {
        let mut j = 0i64;
        while j < 50 {
            let mut k = 0i64;
            while k < 50 {
                sum = sum + 1;
                k = k + 1;
            }
            j = j + 1;
        }
        i = i + 1;
    }

    sum
}

fn bench<F>(name: &str, duration_secs: u64, mut f: F)
where
    F: FnMut() -> i64,
{
    // Warmup (1 second)
    let warmup_end = Instant::now() + Duration::from_secs(1);
    while Instant::now() < warmup_end {
        std::hint::black_box(f());
    }

    // Measurement (fixed duration)
    let measure_duration = Duration::from_secs(duration_secs);
    let start = Instant::now();
    let mut iterations = 0u64;
    let mut result = 0i64;

    let end_time = start + measure_duration;
    while Instant::now() < end_time {
        result = std::hint::black_box(f());
        iterations += 1;
    }

    let elapsed = start.elapsed();

    let avg_us = elapsed.as_micros() as f64 / iterations as f64;
    let ops_per_sec = iterations as f64 / elapsed.as_secs_f64();

    println!("{:<25} {} iterations in {:.3}s: {:.3}µs/op, {:.0} ops/sec (Result: {})",
             name, iterations, elapsed.as_secs_f64(), avg_us, ops_per_sec, result);

    // Prevent optimization from removing the result
    std::hint::black_box(result);
}

fn main() {
    println!("================================================================================");
    println!("Rust Benchmark Suite (Fixed-Time Measurement: 5 seconds each)");
    println!("================================================================================");
    println!();

    bench("Fibonacci(12)", 5, || fib(12));
    bench("PHI Stress If", 5, || phi_stress_if());
    bench("PHI Stress Loop", 5, || phi_stress_loop());

    println!();
    println!("================================================================================");
    println!("Complete!");
    println!("================================================================================");
}
