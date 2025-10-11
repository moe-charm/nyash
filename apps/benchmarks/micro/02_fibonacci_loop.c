// ベンチマーク2: フィボナッチ数列 (C言語版 - ループ版)
// 期待値: 89 (n=10)

#include <stdio.h>

int main() {
    int n = 10;
    int a = 0;
    int b = 1;
    int temp;
    int i = 0;

    while (i < n) {
        temp = a + b;
        a = b;
        b = temp;
        i = i + 1;
    }

    printf("Result: %d\n", b);
    return 0;
}
