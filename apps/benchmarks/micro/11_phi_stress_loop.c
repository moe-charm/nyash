// PHI Stress Test: Nested Loops (depth=3)
// C言語版 - Hakoruneと同じロジック

#include <stdio.h>

int phi_stress_loop(void) {
    int sum = 0;
    int i = 0;

    while (i < 50) {
        int j = 0;
        while (j < 50) {
            int k = 0;
            while (k < 50) {
                sum = sum + 1;
                k = k + 1;
            }
            j = j + 1;
        }
        i = i + 1;
    }

    return sum;
}

int main(void) {
    int result = phi_stress_loop();
    printf("Result: %d\n", result);
    return 0;
}
