#!/usr/bin/env python3
# ベンチマーク7: 文字列結合 (Python版)
# 期待値: 100

def main():
    s = ""
    for i in range(100):
        s = s + "x"
    print(f"Result: {len(s)}")
    return 0

if __name__ == "__main__":
    main()
