// Fibonacci(12) Benchmark - C version (5 seconds fixed time)
// Matches Hakorune 02_fibonacci_bench_v2.hako logic

#include <stdio.h>
#include <sys/time.h>

int fib(int n) {
    if (n <= 1) {
        return n;
    }
    return fib(n - 1) + fib(n - 2);
}

long long now_ms() {
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return (long long)tv.tv_sec * 1000 + tv.tv_usec / 1000;
}

int main(void) {
    printf("Fibonacci(12) Benchmark - C (gcc -O3) - 5 seconds\n\n");

    long long start_time = now_ms();
    long long end_time = start_time + 5000;
    long long iterations = 0;
    int result = 0;

    // Benchmark loop
    while (now_ms() < end_time) {
        result = fib(12);
        iterations++;
    }

    // Calculate stats
    long long elapsed_ms = now_ms() - start_time;
    long long ops_per_sec = iterations * 1000 / elapsed_ms;
    long long us_per_op = elapsed_ms * 1000 / iterations;

    // Print results
    printf("Iterations: %lld\n", iterations);
    printf("Elapsed: %lld ms\n", elapsed_ms);
    printf("Ops/sec: %lld\n", ops_per_sec);
    printf("µs/op: %lld\n", us_per_op);
    printf("Last result: %d\n", result);

    return 0;
}
