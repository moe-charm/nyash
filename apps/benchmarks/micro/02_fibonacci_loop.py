#!/usr/bin/env python3
# ベンチマーク2: フィボナッチ数列 (Python版 - ループ版)
# 期待値: 89

def main():
    n = 10
    a = 0
    b = 1
    i = 0

    while i < n:
        temp = a + b
        a = b
        b = temp
        i = i + 1

    print(f"Result: {b}")
    return 0

if __name__ == "__main__":
    main()
