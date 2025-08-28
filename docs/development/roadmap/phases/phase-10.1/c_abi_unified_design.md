# C ABI統一設計 - JIT/AOT共通基盤

*ChatGPT5さんからのアドバイスに基づく設計文書*

## 🎯 核心的洞察

**プラグインBoxのC ABI = そのままJIT/AOTの呼び出し土台**

JITで今呼んでいるC ABIをAOTでは静的リンクに差し替えるだけでexe化まで一直線！

## 📊 全体アーキテクチャ

```
Nyash → MIR → VM/JIT/Cranelift ──┐
                                 ├─ 呼ぶ先は全部 C ABI: nyrt_* / nyplug_*
NyRT (libnyrt.a/.so)  ←──────────┘
PluginBox 実装 (libnyplug_*.a/.so)
```

- **JIT**: `extern "C"` シンボル（`nyrt_*`/`nyplug_*`）をその場で呼ぶ
- **AOT**: 同じシンボルを`.o`に未解決のまま出力→`libnyrt.a`とプラグイン`.a`をリンク
- **動的配布**: `.so/.dll`に差し替え（同じC ABIでOK）

## 🔧 C ABIルール（小さく強い）

### 1. 命名/可視性
- コア: `nyrt_*`（Box/weak/bus/gc/sync/alloc...）
- プラグイン: `nyplug_{name}_*`（ArrayBox, StringBox など）
- `extern "C"` + 明示の可視性（ELF: `__attribute__((visibility("default")))`）

### 2. ABIの型
- 引数/戻り: `int32_t/int64_t/uint64_t/double/void*` のみに限定
- `bool`は`uint8_t`統一、構造体は不透明ポインタ（ハンドル）
- `varargs`と例外のABI横断は**禁止**（戻り値でエラーコード/out-paramで返す）

### 3. レイアウト/アライン
```c
// Boxハンドル例
struct NyBox { 
    void* data; 
    uint64_t typeid; 
    uint32_t flags; 
    uint32_t gen; 
};
```
※JIT側は中身に触らない。操作はAPI経由のみ。

### 4. 呼び出し規約
- x86_64 SysV / aarch64 AAPCS64 / Win64 をターゲットごとに固定
- Craneliftの`call_conv`を上記に合わせる（JIT/AOT共通）

### 5. バージョン管理（fail-fast）
- `nyrt_abi_version()` / `nyplug_{name}_abi_version()`（v0=1）。不一致は起動時に即fail（ローダ側で検査）。

## 📝 最小ヘッダ雛形

### nyrt.h（コアランタイム）
```c
#pragma once
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif

typedef struct NyBox { 
    void* data; 
    uint64_t typeid; 
    uint32_t flags; 
    uint32_t gen; 
} NyBox;

int32_t  nyrt_abi_version(void);

// Box/weak
NyBox    nyrt_box_new(uint64_t typeid, uint64_t size);
void     nyrt_box_free(NyBox b);
int32_t  nyrt_adopt(NyBox parent, NyBox child);
int32_t  nyrt_release(NyBox parent, NyBox child, NyBox* out_weak);
NyBox    nyrt_weak_load(NyBox weak);   // gen一致ならBox, 失効なら{0}

// GC/epoch
void     nyrt_epoch_collect(void);

// Sync（最低限）
void*    nyrt_mutex_lock(NyBox sync);
void     nyrt_mutex_unlock(void* guard);

// Bus
int32_t  nyrt_bus_send(NyBox port, NyBox msg);

#ifdef __cplusplus
}
#endif
```

### nyplug_array.h（プラグイン例：ArrayBox）
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
int32_t nyplug_array_push(NyBox arr, NyBox v);

#ifdef __cplusplus
}
#endif
```

## 🚀 ビルド・配布フロー（AOT静的リンク）

1. **JITと同じLowering**でCLIF生成
2. **ObjectWriter**で`mod.o`出力（未解決：`nyrt_*`/`nyplug_*`）
3. **リンク**
   - Linux/macOS: `cc mod.o -static -L. -lnyrt -lnyplug_array -o app`
   - Windows: `link mod.obj nyrt.lib nyplug_array.lib /OUT:app.exe`
4. 実行：`./app`でJIT無しに動作

補足: 現行実装ではプラグインは `nyash_plugin_invoke`（BID-FFI v1, TLV）を用いる。v0ではこれを固定し、将来的に `nyplug_*` 直関数を併置する場合も `*_abi_version()` で互換を担保する。

## ⚡ 実装順序（重要！）

1. **必要なビルトインBoxをプラグインBoxに変換**
2. **VM動作確認**
3. **JIT動作確認**
4. **AOT実装**

## ⚠️ 地雷と回避策

- **名前修飾/装飾**: C++で実装するなら`extern "C"`を絶対忘れない
- **サイズ違い**: `bool`/`size_t`のプラットフォーム差 → 明示幅型で統一
- **例外越境**: C ABI越しに`throw`/`panic`禁止。エラーコード＋out-paramで返す
- **並行**: JITから`nyrt_mutex_*`を呼ぶ箇所はSafepointとも整合するように（長保持しない）

## 📋 即実行ToDo（30分で前進）

- [ ] `nyrt.h`最小セット確定（上の雛形でOK）
- [ ] Craneliftの`call_conv`をターゲットに合わせて固定
- [ ] JIT経路で`nyrt_abi_version()==NYRT_ABI`を起動時チェック
- [ ] AOT試作：`add.o`を吐いて`libnyrt.a`とリンク→`add()`を呼ぶ最小exe

## 💡 まとめ

> プラグインBoxのC ABI＝JIT/AOTの土台だから、
> **いまのJITが動いている＝静的リンクexeの最短ルートはもう目の前。**
> まずは`nyrt.h`を固定して、JITとAOTの両方で同じシンボルを呼ぶ状態にしよう。
> それで**"今日のJIT"が"明日のexe"**に化けるにゃ。

---

*最終更新: 2025-08-28*
