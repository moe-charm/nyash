#!/usr/bin/env python3
# Python Fixed-Time Benchmark Suite (5 seconds each)
# Unified measurement for fair comparison

import time

# Benchmark 1: Fibonacci(12)
def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def bench_fibonacci():
    return fib(12)

# Benchmark 2: PHI Stress If
def bench_phi_if():
    x = 0
    i = 0
    while i < 10000:
        if i % 8 == 0:
            x = x + 1
        elif i % 8 == 1:
            x = x + 2
        elif i % 8 == 2:
            x = x + 3
        elif i % 8 == 3:
            x = x + 4
        elif i % 8 == 4:
            x = x + 5
        elif i % 8 == 5:
            x = x + 6
        elif i % 8 == 6:
            x = x + 7
        else:
            x = x + 8
        i = i + 1
    return x

# Benchmark 3: PHI Stress Loop
def bench_phi_loop():
    sum_val = 0
    i = 0
    while i < 50:
        j = 0
        while j < 50:
            k = 0
            while k < 50:
                sum_val = sum_val + 1
                k = k + 1
            j = j + 1
        i = i + 1
    return sum_val

def run_bench(name, bench_func, duration_sec=5):
    # Warmup 1 second
    warmup_end = time.time() + 1.0
    while time.time() < warmup_end:
        bench_func()

    # Measurement
    iterations = 0
    start = time.time()
    end_time = start + duration_sec

    while time.time() < end_time:
        result = bench_func()
        iterations += 1

    elapsed = time.time() - start

    # Calculate stats
    us_per_op = (elapsed * 1_000_000) / iterations
    ops_per_sec = iterations / elapsed

    print(f"{name}")
    print(f"  Iterations: {iterations} in {elapsed:.3f}s")
    print(f"  µs/op: {us_per_op:.3f}")
    print(f"  ops/sec: {ops_per_sec:.0f}")
    print()

if __name__ == "__main__":
    print("=" * 80)
    print("Python Benchmark Suite (Fixed-Time: 5 seconds each)")
    print("=" * 80)
    print()

    run_bench("Fibonacci(12)", bench_fibonacci)
    run_bench("PHI Stress If", bench_phi_if)
    run_bench("PHI Stress Loop", bench_phi_loop)

    print("=" * 80)
    print("Complete!")
    print("=" * 80)
