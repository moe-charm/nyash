// C Fixed-Time Benchmark Suite (5 seconds each)
// Unified measurement for fair comparison

#include <stdio.h>
#include <time.h>
#include <stdint.h>

// Benchmark 1: Fibonacci(12)
int64_t fib(int64_t n) {
    if (n <= 1) return n;
    return fib(n - 1) + fib(n - 2);
}

int64_t bench_fibonacci() {
    return fib(12);
}

// Benchmark 2: PHI Stress If
int64_t bench_phi_if() {
    int64_t x = 0;
    int64_t i = 0;

    while (i < 10000) {
        if (i % 8 == 0) {
            x = x + 1;
        } else if (i % 8 == 1) {
            x = x + 2;
        } else if (i % 8 == 2) {
            x = x + 3;
        } else if (i % 8 == 3) {
            x = x + 4;
        } else if (i % 8 == 4) {
            x = x + 5;
        } else if (i % 8 == 5) {
            x = x + 6;
        } else if (i % 8 == 6) {
            x = x + 7;
        } else {
            x = x + 8;
        }
        i = i + 1;
    }

    return x;
}

// Benchmark 3: PHI Stress Loop
int64_t bench_phi_loop() {
    int64_t sum = 0;
    int64_t i = 0;

    while (i < 50) {
        int64_t j = 0;
        while (j < 50) {
            int64_t k = 0;
            while (k < 50) {
                sum = sum + 1;
                k = k + 1;
            }
            j = j + 1;
        }
        i = i + 1;
    }

    return sum;
}

// Get current time in seconds (double precision)
double get_time() {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return ts.tv_sec + ts.tv_nsec / 1000000000.0;
}

void run_bench(const char* name, int64_t (*bench_func)(), double duration_sec) {
    // Warmup 1 second
    double warmup_end = get_time() + 1.0;
    while (get_time() < warmup_end) {
        bench_func();
    }

    // Measurement
    uint64_t iterations = 0;
    double start = get_time();
    double end_time = start + duration_sec;
    int64_t result = 0;

    while (get_time() < end_time) {
        result = bench_func();
        iterations++;
    }

    double elapsed = get_time() - start;

    // Calculate stats
    double us_per_op = (elapsed * 1000000.0) / iterations;
    double ops_per_sec = iterations / elapsed;

    printf("%s\n", name);
    printf("  Iterations: %lu in %.3fs\n", iterations, elapsed);
    printf("  µs/op: %.3f\n", us_per_op);
    printf("  ops/sec: %.0f\n", ops_per_sec);
    printf("\n");
}

int main() {
    printf("================================================================================\n");
    printf("C Benchmark Suite (Fixed-Time: 5 seconds each)\n");
    printf("================================================================================\n");
    printf("\n");

    run_bench("Fibonacci(12)", bench_fibonacci, 5.0);
    run_bench("PHI Stress If", bench_phi_if, 5.0);
    run_bench("PHI Stress Loop", bench_phi_loop, 5.0);

    printf("================================================================================\n");
    printf("Complete!\n");
    printf("================================================================================\n");

    return 0;
}
