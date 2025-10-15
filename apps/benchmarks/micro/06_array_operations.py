#!/usr/bin/env python3
# ベンチマーク6: 配列操作 (Python版)
# 期待値: 499500

def main():
    arr = []
    for i in range(1000):
        arr.append(i)

    sum = 0
    for i in range(1000):
        sum += arr[i]

    print(f"Result: {sum}")
    return 0

if __name__ == "__main__":
    main()
