#!/usr/bin/env python3
"""sum_loop ベンチマーク（固定時間5秒方式）
言語対決用: Nyash vs Python vs C
"""

import time

def main():
    DURATION_SEC = 5  # 5秒間測定

    start_time = time.time()
    end_time = start_time + DURATION_SEC

    iterations = 0
    sum_val = 0
    current_time = time.time()

    # 固定時間方式: end_timeまで繰り返し実行
    while current_time < end_time:
        sum_val += iterations
        iterations += 1
        current_time = time.time()

    # 結果表示
    elapsed_sec = current_time - start_time
    ops_per_sec = iterations / elapsed_sec

    print(f"Iterations: {iterations}")
    print(f"Elapsed: {elapsed_sec:.3f} sec")
    print(f"Ops/sec: {ops_per_sec:.0f}")
    print(f"Sum: {sum_val}")

    return 0

if __name__ == "__main__":
    exit(main())
