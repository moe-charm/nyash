# 解決策：埋め込みVMによるスクリプトプラグイン実現

## 💡 発想の転換

**制約は「リンク時にC ABIが必要」だけ。つまり、C ABI関数の中でVMを動かせばいい！**

## 🎯 アーキテクチャ

```c
// C ABI関数（静的リンク可能）
extern "C" int32_t nyplug_custom_math_invoke(
    uint32_t method_id,
    const uint8_t* args,
    size_t args_len,
    uint8_t* result,
    size_t* result_len
) {
    // 埋め込みVM起動
    static NyashVM* embedded_vm = NULL;
    if (!embedded_vm) {
        embedded_vm = nyash_vm_create_minimal();
        nyash_vm_load_script(embedded_vm, EMBEDDED_SCRIPT);
    }
    
    // スクリプト実行
    return nyash_vm_invoke(embedded_vm, method_id, args, args_len, result, result_len);
}

// スクリプトは文字列リテラルとして埋め込み
static const char* EMBEDDED_SCRIPT = R"(
export box CustomMath {
    cached_sin(x) {
        // Nyashコード
    }
}
)";
```

## 🔄 実現方法

### 1. Nyash→C トランスパイラー

```bash
# Nyashスクリプト → C関数
nyash-to-c custom_math.ny > custom_math_plugin.c

# 生成されるC
// custom_math_plugin.c
#include "nyash_embedded_vm.h"

static const char* SCRIPT = "..."; // Nyashコード埋め込み

extern "C" int32_t nyplug_custom_math_invoke(...) {
    return nyash_embedded_invoke(SCRIPT, method_id, ...);
}
```

### 2. 最小VM実装

```rust
// crates/nyash-embedded-vm
pub struct EmbeddedVM {
    // 最小限の実行環境
    values: Vec<VMValue>,
    // スクリプトはプリコンパイル済みMIR
    mir: MirModule,
}

#[no_mangle]
pub extern "C" fn nyash_embedded_invoke(
    script: *const c_char,
    method_id: u32,
    // ... TLV args/result
) -> i32 {
    // MIR実行（インタープリター）
}
```

## 📊 利点と制約

### ✅ 可能になること
- **スクリプトプラグインがEXEに埋め込み可能**
- **JIT/AOTから呼び出し可能**（C ABI経由）
- **既存のプラグインシステムと完全互換**

### ⚠️ 制約
- **パフォーマンス**: 埋め込みVMのオーバーヘッド
- **サイズ**: 最小VMランタイムが必要（~500KB?）
- **機能制限**: フルVMの一部機能のみ

## 🚀 段階的実装

### Phase 1: 最小埋め込みVM
```rust
// 必要最小限の機能
- MIR実行（インタープリター）
- 基本型（Integer, String, Bool）
- メソッド呼び出し
- TLVエンコード/デコード
```

### Phase 2: Nyash→Cトランスパイラー
```nyash
// input: custom_math.ny
export box CustomMath {
    sin(x) { ... }
}

// output: custom_math_plugin.c
extern "C" int32_t nyplug_custom_math_invoke(...) {
    static const uint8_t MIR_BYTECODE[] = { ... };
    return nyash_embedded_execute(MIR_BYTECODE, ...);
}
```

### Phase 3: 最適化
- MIRプリコンパイル
- 頻出パスのネイティブ化
- 選択的JITコンパイル

## 💡 実装例

```c
// 生成されたプラグイン
#include <nyash_embedded.h>

// MIRバイトコード（事前コンパイル）
static const uint8_t CUSTOM_MATH_MIR[] = {
    0x01, 0x00, // version
    0x10, 0x00, // function count
    // ... MIR instructions
};

extern "C" int32_t nyplug_custom_math_abi_version() {
    return 1;
}

extern "C" int32_t nyplug_custom_math_invoke(
    uint32_t method_id,
    const uint8_t* args,
    size_t args_len,
    uint8_t* result,
    size_t* result_len
) {
    // 埋め込みVM実行
    return nyash_mir_execute(
        CUSTOM_MATH_MIR,
        sizeof(CUSTOM_MATH_MIR),
        method_id,
        args, args_len,
        result, result_len
    );
}
```

## 🎯 結論

**「リンク時にC ABI」という制約は、埋め込みVMで解決可能！**

- Nyashスクリプト → MIR → Cコード → ネイティブプラグイン
- 開発の容易さ（Nyash）と配布の利便性（C ABI）を両立
- 既存のプラグインエコシステムと完全互換

これで「Everything is Box」が真に実現する！