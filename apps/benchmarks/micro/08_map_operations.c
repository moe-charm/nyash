// ベンチマーク8: Map操作 (C言語版)
// 期待値: 499500
// 注: 簡易的なハッシュマップ（線形探索）

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#define MAP_SIZE 1000

typedef struct {
    char key[32];
    int value;
    int used;
} MapEntry;

int main() {
    MapEntry* map = (MapEntry*)calloc(MAP_SIZE, sizeof(MapEntry));
    int i = 0;
    int sum = 0;
    char key[32];

    // set
    while (i < 1000) {
        snprintf(key, sizeof(key), "key%d", i);
        map[i].used = 1;
        strcpy(map[i].key, key);
        map[i].value = i;
        i = i + 1;
    }

    // get + sum
    sum = 0;
    i = 0;
    while (i < 1000) {
        snprintf(key, sizeof(key), "key%d", i);
        // Linear search (simple implementation)
        for (int j = 0; j < MAP_SIZE; j++) {
            if (map[j].used && strcmp(map[j].key, key) == 0) {
                sum = sum + map[j].value;
                break;
            }
        }
        i = i + 1;
    }

    printf("Result: %d\n", sum);
    free(map);
    return 0;
}
