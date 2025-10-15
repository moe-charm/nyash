// ベンチマーク5: オブジェクト作成・破棄 (C言語版)
// 期待値: 999
// 注: C言語ではstruct+mallocで疑似的にオブジェクトを表現

#include <stdio.h>
#include <stdlib.h>

typedef struct {
    int value;
} Counter;

int main() {
    int last_value = 0;
    int i = 0;

    while (i < 1000) {
        Counter* c = (Counter*)malloc(sizeof(Counter));
        c->value = 0;
        c->value = i;
        last_value = c->value;
        free(c);
        i = i + 1;
    }

    printf("Result: %d\n", last_value);
    return 0;
}
