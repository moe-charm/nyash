#include <stdio.h>
#include <stdlib.h>
#include <dlfcn.h>
#include <string.h>

// FFI関数型定義
typedef int (*plugin_init_fn)(void);
typedef char* (*parse_fn)(const char*);
typedef void (*free_fn)(char*);

int main() {
    // プラグインをロード
    void* handle = dlopen("./target/release/libnyash_python_parser_plugin.so", RTLD_LAZY);
    if (!handle) {
        fprintf(stderr, "Failed to load plugin: %s\n", dlerror());
        return 1;
    }

    // 関数を取得
    plugin_init_fn init_fn = (plugin_init_fn)dlsym(handle, "nyash_plugin_init");
    parse_fn parse_func = (parse_fn)dlsym(handle, "nyash_python_parse");
    free_fn free_func = (free_fn)dlsym(handle, "nyash_python_free_string");

    if (!init_fn || !parse_func || !free_func) {
        fprintf(stderr, "Failed to load functions\n");
        dlclose(handle);
        return 1;
    }

    // 初期化
    if (init_fn() != 0) {
        fprintf(stderr, "Plugin init failed\n");
        dlclose(handle);
        return 1;
    }

    // 環境変数からコードを取得
    const char* code = getenv("NYASH_PY_CODE");
    if (!code) {
        code = "def main():\n    return 0";
        printf("Using default code: %s\n", code);
    } else {
        printf("Parsing code from NYASH_PY_CODE: %s\n", code);
    }

    // パース実行
    char* result = parse_func(code);
    if (result) {
        printf("\n=== Parse Result ===\n%s\n", result);
        free_func(result);
    } else {
        printf("Parse failed\n");
    }

    dlclose(handle);
    return 0;
}