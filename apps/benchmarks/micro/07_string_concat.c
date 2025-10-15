// ベンチマーク7: 文字列結合 (C言語版)
// 期待値: 100
// 注: C言語でもO(n²)の実装（公平な比較のため）

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

int main() {
    char* s = (char*)malloc(1);
    s[0] = '\0';
    int i = 0;

    while (i < 100) {
        int old_len = strlen(s);
        char* new_s = (char*)malloc(old_len + 2);
        strcpy(new_s, s);
        new_s[old_len] = 'x';
        new_s[old_len + 1] = '\0';
        free(s);
        s = new_s;
        i = i + 1;
    }

    printf("Result: %d\n", (int)strlen(s));
    free(s);
    return 0;
}
