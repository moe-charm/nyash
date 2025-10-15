// ベンチマーク6: 配列操作 (C言語版)
// 期待値: 499500 (0+1+2+...+999)
// 注: 動的配列としてmallocで確保

#include <stdio.h>
#include <stdlib.h>

int main() {
    int* arr = (int*)malloc(1000 * sizeof(int));
    int i = 0;
    int sum = 0;

    // push equivalent
    while (i < 1000) {
        arr[i] = i;
        i = i + 1;
    }

    // get + sum
    sum = 0;
    i = 0;
    while (i < 1000) {
        sum = sum + arr[i];
        i = i + 1;
    }

    printf("Result: %d\n", sum);
    free(arr);
    return 0;
}
