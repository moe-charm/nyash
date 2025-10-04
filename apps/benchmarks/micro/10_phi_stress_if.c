// PHI Stress Test: If-Else Chain (8-way branch)
// C言語版 - Hakoruneと同じロジック

#include <stdio.h>

int phi_stress_if(void) {
    int x = 0;
    int i = 0;

    while (i < 10000) {
        // 8-way branch
        if (i % 8 == 0) {
            x = x + 1;
        } else if (i % 8 == 1) {
            x = x + 2;
        } else if (i % 8 == 2) {
            x = x + 3;
        } else if (i % 8 == 3) {
            x = x + 4;
        } else if (i % 8 == 4) {
            x = x + 5;
        } else if (i % 8 == 5) {
            x = x + 6;
        } else if (i % 8 == 6) {
            x = x + 7;
        } else {
            x = x + 8;
        }

        i = i + 1;
    }

    return x;
}

int main(void) {
    int result = phi_stress_if();
    printf("Result: %d\n", result);
    return 0;
}
