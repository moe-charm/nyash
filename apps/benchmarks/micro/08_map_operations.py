#!/usr/bin/env python3
# ベンチマーク8: Map操作 (Python版)
# 期待値: 499500

def main():
    map = {}
    for i in range(1000):
        key = f"key{i}"
        map[key] = i

    sum = 0
    for i in range(1000):
        key = f"key{i}"
        sum += map[key]

    print(f"Result: {sum}")
    return 0

if __name__ == "__main__":
    main()
