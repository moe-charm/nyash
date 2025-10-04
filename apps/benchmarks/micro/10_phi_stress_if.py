#!/usr/bin/env python3
# PHI Stress Test: If-Else Chain (8-way branch)
# Python版 - Hakoruneと同じロジック

def phi_stress_if():
    x = 0
    i = 0

    while i < 10000:
        # 8-way branch
        if i % 8 == 0:
            x = x + 1
        elif i % 8 == 1:
            x = x + 2
        elif i % 8 == 2:
            x = x + 3
        elif i % 8 == 3:
            x = x + 4
        elif i % 8 == 4:
            x = x + 5
        elif i % 8 == 5:
            x = x + 6
        elif i % 8 == 6:
            x = x + 7
        else:
            x = x + 8

        i = i + 1

    return x

if __name__ == "__main__":
    result = phi_stress_if()
    print(f"Result: {result}")
