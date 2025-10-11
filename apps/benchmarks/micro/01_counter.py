#!/usr/bin/env python3
# ベンチマーク1: シンプルカウンター (Python版)
# 期待値: 10

def main():
    i = 0
    while i < 10:
        i = i + 1
    print(f"Result: {i}")
    return 0

if __name__ == "__main__":
    main()
