# NyRT C ABI v0 (JIT/AOT/Plugin 共通)

- 命名規則:
  - コア: `nyrt_*`
  - プラグイン: `nyplug_{name}_*`
- 呼出規約: x86_64 SysV / aarch64 AAPCS64 / Win64（Cranelift `call_conv` と一致）
- 型: `i32/i64/u64/double/void*` に限定。`bool` は `u8`。構造体は不透明ハンドル。
- 互換性: `nyrt_abi_version()` / `nyplug_{name}_abi_version()`（v0=1）で fail-fast。

## 最小ヘッダ例（コア: nyrt.h）
```c
#pragma once
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

typedef struct NyBox { void* data; uint64_t typeid; uint32_t flags; uint32_t gen; } NyBox;

int32_t  nyrt_abi_version(void);

// Box 基本
NyBox    nyrt_box_new(uint64_t typeid, uint64_t size);
void     nyrt_box_free(NyBox b);
int32_t  nyrt_adopt(NyBox parent, NyBox child);
int32_t  nyrt_release(NyBox parent, NyBox child, NyBox* out_weak);
NyBox    nyrt_weak_load(NyBox weak);

// GC/Safepoint
void     nyrt_epoch_collect(void);

// Sync（最小）
void*    nyrt_mutex_lock(NyBox sync);
void     nyrt_mutex_unlock(void* guard);

// Bus
int32_t  nyrt_bus_send(NyBox port, NyBox msg);

#ifdef __cplusplus
}
#endif
```

## プラグインヘッダ例（nyplug_array.h）
```c
#pragma once
#include "nyrt.h"
#ifdef __cplusplus
extern "C" {
#endif

int32_t nyplug_array_abi_version(void);
NyBox   nyplug_array_new(void);
int32_t nyplug_array_get(NyBox arr, uint64_t i, NyBox* out);
int32_t nyplug_array_set(NyBox arr, uint64_t i, NyBox v);
uint64_t nyplug_array_len(NyBox arr);

#ifdef __cplusplus
}
#endif
```

## AOT（静的リンク）の流れ
- CLIF から `.o` を出力（未解決: `nyrt_*` / `nyplug_*`）。
- リンク（Linux/macOS の例）:
  ```bash
  # 1) NyRT静的ライブラリをビルド
  (cd crates/nyrt && cargo build --release)

  # 2) プラグイン静的ライブラリ（必要なもの）をビルド
  (cd plugins/nyash-array-plugin   && cargo build --release)
  (cd plugins/nyash-string-plugin  && cargo build --release)
  (cd plugins/nyash-integer-plugin && cargo build --release)

  # 3) ObjectModuleで生成した app.o を NyRT + plugins と静的リンク
  cc app.o \
    -L crates/nyrt/target/release \
    -L plugins/nyash-array-plugin/target/release \
    -L plugins/nyash-string-plugin/target/release \
    -L plugins/nyash-integer-plugin/target/release \
    -static -Wl,--whole-archive -lnyrt -Wl,--no-whole-archive \
    -lnyash_array_plugin -lnyash_string_plugin -lnyash_integer_plugin \
    -o app
  ```
- 実行: `./app`

補足（Phase 10.2 暫定→10.2b）
- `.o` には暫定シム `nyash_plugin_invoke3_i64` が未解決として現れますが、`crates/nyrt` の `libnyrt.a` がこれを実装します。
- 将来的に `nyrt_*` 命名へ整理予定（後方互換シムは残す）。

## 注意
- 例外/パニックの越境は不可（戻り値/エラーコードで返す）。
- C++ 側は必ず `extern "C"`、ELF は `visibility("default")`。
- ABI 破壊変更は `*_v1` などの別シンボルで導入（v0は凍結）。
