#!/usr/bin/env python3
# ベンチマーク5: オブジェクト作成・破棄 (Python版)
# 期待値: 999

class Counter:
    def __init__(self):
        self.value = 0

def main():
    last_value = 0
    for i in range(1000):
        c = Counter()
        c.value = i
        last_value = c.value
    print(f"Result: {last_value}")
    return 0

if __name__ == "__main__":
    main()
