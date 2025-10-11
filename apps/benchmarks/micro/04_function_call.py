#!/usr/bin/env python3
# ベンチマーク4: 関数呼び出しオーバーヘッド (Python版)
# 期待値: 1000

def add(a, b):
    return a + b

def main():
    sum = 0
    for i in range(1000):
        sum = add(sum, 1)
    print(f"Result: {sum}")
    return 0

if __name__ == "__main__":
    main()
