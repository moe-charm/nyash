// ベンチマーク1: シンプルカウンター (C言語版)
// 期待値: 10

#include <stdio.h>

int main() {
    int i = 0;
    while (i < 10) {
        i = i + 1;
    }
    printf("Result: %d\n", i);
    return 0;
}
