#!/usr/bin/env python3
# PHI Stress Test: Nested Loops (depth=3)
# Python版 - Hakoruneと同じロジック

def phi_stress_loop():
    sum_val = 0
    i = 0

    while i < 50:
        j = 0
        while j < 50:
            k = 0
            while k < 50:
                sum_val = sum_val + 1
                k = k + 1
            j = j + 1
        i = i + 1

    return sum_val

if __name__ == "__main__":
    result = phi_stress_loop()
    print(f"Result: {result}")
