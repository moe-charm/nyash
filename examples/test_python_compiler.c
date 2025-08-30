// PythonCompilerBox 簡易テスト
// 環境変数 NYASH_PY_IR からJSON IRを読み取ってNyashコードを生成

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// 超簡易的なコンパイラ（JSONパースなし）
const char* compile_simple(const char* ir) {
    // JSONをちゃんとパースせずに、簡単なパターンマッチング
    if (strstr(ir, "\"name\":\"main\"") && strstr(ir, "\"return_value\":0")) {
        return "// Generated from Python\n"
               "static box Main {\n"
               "    main() {\n"
               "        return 0\n"
               "    }\n"
               "}\n";
    }
    return "// Unsupported IR\n";
}

int main() {
    const char* ir = getenv("NYASH_PY_IR");
    if (!ir) {
        ir = "{\"module\":{\"functions\":[{\"name\":\"main\",\"return_value\":0}]}}";
        printf("Using default IR: %s\n\n", ir);
    } else {
        printf("Compiling IR from NYASH_PY_IR: %s\n\n", ir);
    }
    
    printf("=== Generated Nyash Code ===\n");
    printf("%s", compile_simple(ir));
    
    return 0;
}