/* sum_loop ベンチマーク（固定時間5秒方式）
 * 言語対決用: Nyash vs Python vs C
 */

#include <stdio.h>
#include <time.h>

// ミリ秒単位の現在時刻を取得
long long get_time_ms() {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (long long)ts.tv_sec * 1000 + ts.tv_nsec / 1000000;
}

int main() {
    const int DURATION_SEC = 5;  // 5秒間測定

    long long start_time = get_time_ms();
    long long end_time = start_time + DURATION_SEC * 1000;

    long long iterations = 0;
    long long sum = 0;
    long long current_time = get_time_ms();

    // 固定時間方式: end_timeまで繰り返し実行
    while (current_time < end_time) {
        sum += iterations;
        iterations++;
        current_time = get_time_ms();
    }

    // 結果表示
    long long elapsed_ms = current_time - start_time;
    long long ops_per_sec = (iterations * 1000) / elapsed_ms;

    printf("Iterations: %lld\n", iterations);
    printf("Elapsed: %lld ms\n", elapsed_ms);
    printf("Ops/sec: %lld\n", ops_per_sec);
    printf("Sum: %lld\n", sum);

    return 0;
}
