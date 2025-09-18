# 埋め込みVM実装ロードマップ

## 🎯 目標：スクリプトプラグインのC ABI化

**Nyashスクリプト → C ABIプラグイン変換の完全自動化**

## 📊 技術スタック

```
[Nyashスクリプト]
    ↓ パース・型チェック
[MIR (中間表現)]
    ↓ 最適化・定数畳み込み
[MIRバイトコード]
    ↓ 埋め込み
[Cソースコード] ← nyash-to-c ツール
    ↓ コンパイル (cc/clang/gcc)
[.so/.dll/.a] ← 通常のプラグイン！
```

## 🚀 実装フェーズ

### Phase 12.1: 最小埋め込みVM（2-3週間）

#### 1. MIRバイトコード設計
```rust
// mir_bytecode.rs
pub enum CompactInstruction {
    // 1バイト命令（頻出）
    LoadLocal(u8),      // 0x00-0x7F
    StoreLocal(u8),     // 0x80-0xFF
    
    // 2バイト命令
    LoadConst(u8),      // 0x01 XX
    Call(u8),           // 0x02 XX
    
    // 可変長
    LoadString,         // 0x03 [len:u16] [data]
    Jump,               // 0x04 [offset:i16]
}
```

#### 2. 埋め込みVMコア
```c
// nyash_embedded_vm.h
typedef struct {
    const uint8_t* bytecode;
    size_t bytecode_len;
    
    // 実行時状態（最小）
    void* stack[256];
    int sp;
    void* locals[16];
} NyashEmbeddedVM;

int32_t nyash_embedded_execute(
    const uint8_t* bytecode,
    size_t bytecode_len,
    uint32_t method_id,
    const uint8_t* args,
    size_t args_len,
    uint8_t* result,
    size_t* result_len
);
```

### Phase 12.2: Nyash→Cトランスパイラー（3-4週間）

#### 1. 基本変換
```bash
$ nyash-to-c math_plugin.ny -o math_plugin.c
Generating C plugin from Nyash script...
- Parsing... OK
- Type checking... OK  
- MIR generation... OK
- Bytecode emission... OK
- C code generation... OK
Output: math_plugin.c (2.3KB)
```

#### 2. 生成コード例
```c
// Generated from: math_plugin.ny
#include <nyash_embedded.h>

// MIRバイトコード（最適化済み）
static const uint8_t BYTECODE[] = {
    0x01, 0x00,  // Version 1.0
    0x01, 0x00,  // 1 function
    
    // Function: cached_sin
    0x00, 0x08,  // Function header
    0x80, 0x00,  // StoreLocal 0 (x)
    0x02, 0x10,  // Call sin
    0x90,        // Return
};

// プラグインエントリポイント
extern "C" int32_t nyplug_math_plugin_invoke(
    uint32_t type_id,
    uint32_t method_id,
    uint32_t instance_id,
    const uint8_t* args,
    size_t args_len,
    uint8_t* result,
    size_t* result_len
) {
    return nyash_embedded_execute(
        BYTECODE, sizeof(BYTECODE),
        method_id,
        args, args_len,
        result, result_len
    );
}
```

### Phase 12.3: 最適化とツールチェーン（4-6週間）

#### 1. ビルドシステム統合
```toml
# nyash.toml
[[plugins]]
name = "math_plugin"
source = "plugins/math_plugin.ny"  # Nyashソース
type = "script"                     # 自動的にC変換

[[plugins]]
name = "file_plugin"  
source = "plugins/file_plugin/Cargo.toml"
type = "native"                     # 従来のRustプラグイン
```

#### 2. 自動ビルドパイプライン
```bash
$ nyash build --plugins
Building plugins...
[1/2] math_plugin (script)
  - Transpiling to C... OK
  - Compiling... OK
  - Output: target/plugins/libmath_plugin.so
[2/2] file_plugin (native)
  - Building with cargo... OK
  - Output: target/plugins/libfile_plugin.so
Done!
```

## 📈 パフォーマンス目標

| 操作 | ネイティブ | 埋め込みVM | 目標比率 |
|------|-----------|------------|----------|
| 単純計算 | 10ns | 50ns | 5x |
| メソッド呼び出し | 20ns | 100ns | 5x |
| 文字列操作 | 100ns | 200ns | 2x |
| I/O操作 | 10μs | 10.1μs | 1.01x |

## 🔧 開発ツール

### 1. デバッガ
```bash
$ nyash-debug math_plugin.ny --method cached_sin --args "[3.14]"
Executing cached_sin(3.14)...
[PC:0000] LoadLocal 0     ; x = 3.14
[PC:0002] Call sin         ; sin(3.14)
[PC:0004] Return           ; 0.0015926...
Result: 0.0015926
```

### 2. プロファイラ
```bash
$ nyash-profile math_plugin.so
Method statistics:
- cached_sin: 1000 calls, avg 120ns
- cached_cos: 500 calls, avg 115ns
Bottlenecks: None detected
```

## 🎉 最終形

```bash
# 開発者の体験
$ cat my_plugin.ny
export box MyPlugin {
    init { cache = new MapBox() }
    process(x) { return x * 2 }
}

$ nyash build my_plugin.ny
✓ Generated: my_plugin.so

$ nyash run --plugin my_plugin.so test.ny
✓ Plugin loaded (C ABI)
✓ Result: 42
```

**Nyashで書いて、どこでも動く！**