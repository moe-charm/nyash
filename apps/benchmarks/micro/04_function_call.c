// ベンチマーク4: 関数呼び出しオーバーヘッド (C言語版)
// 期待値: 1000

#include <stdio.h>

int add(int a, int b) {
    return a + b;
}

int main() {
    int sum = 0;
    int i = 0;

    while (i < 1000) {
        sum = add(sum, 1);
        i = i + 1;
    }

    printf("Result: %d\n", sum);
    return 0;
}
