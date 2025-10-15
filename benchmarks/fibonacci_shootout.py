#!/usr/bin/env python3
# Fibonacci(12) Benchmark - Python version (5 seconds fixed time)
# Matches Hakorune 02_fibonacci_bench_v2.hako logic

import time

def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

def now_ms():
    return int(time.time() * 1000)

def main():
    print("Fibonacci(12) Benchmark - Python 3 - 5 seconds")
    print()

    start_time = now_ms()
    end_time = start_time + 5000
    iterations = 0
    result = 0

    # Benchmark loop
    while now_ms() < end_time:
        result = fib(12)
        iterations += 1

    # Calculate stats
    elapsed_ms = now_ms() - start_time
    ops_per_sec = iterations * 1000 // elapsed_ms
    us_per_op = elapsed_ms * 1000 // iterations

    # Print results
    print(f"Iterations: {iterations}")
    print(f"Elapsed: {elapsed_ms} ms")
    print(f"Ops/sec: {ops_per_sec}")
    print(f"µs/op: {us_per_op}")
    print(f"Last result: {result}")

if __name__ == "__main__":
    main()
