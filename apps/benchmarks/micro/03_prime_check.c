// ベンチマーク3: 素数判定 (C言語版)
// 期待値: 1 (17は素数)

#include <stdio.h>

int main() {
    int n = 17;
    int i = 2;
    int is_prime = 1;
    int divisor_found = 0;

    while (i * i <= n) {
        if (n - (n / i) * i == 0) {
            divisor_found = 1;
        }
        i = i + 1;
    }

    if (divisor_found == 1) {
        is_prime = 0;
    }

    printf("Result: %d\n", is_prime);
    return 0;
}
