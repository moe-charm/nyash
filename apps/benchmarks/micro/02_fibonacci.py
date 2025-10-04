#!/usr/bin/env python3
# Fibonacci(12) - Recursive version
# Python版 - Hakoruneと同じロジック

def fib(n):
    if n <= 1:
        return n
    return fib(n - 1) + fib(n - 2)

if __name__ == "__main__":
    result = fib(12)
    print(f"Result: {result}")
