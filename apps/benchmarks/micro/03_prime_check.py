#!/usr/bin/env python3
# ベンチマーク3: 素数判定 (Python版)
# 期待値: 1

def main():
    n = 17
    i = 2
    is_prime = 1
    divisor_found = 0

    while i * i <= n:
        if n - (n // i) * i == 0:
            divisor_found = 1
        i = i + 1

    if divisor_found == 1:
        is_prime = 0

    print(f"Result: {is_prime}")
    return 0

if __name__ == "__main__":
    main()
