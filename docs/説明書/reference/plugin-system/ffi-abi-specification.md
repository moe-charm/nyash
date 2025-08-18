# Box FFI/ABI v0 (BID-1 Enhanced Edition)

Purpose
- Define a language-agnostic ABI to call external libraries as Boxes.
- Serve as a single source of truth for MIR ExternCall, WASM RuntimeImports, VM stubs, and future language codegens (TS/Python/Rust/LLVM IR).
- Support Everything is Box philosophy with efficient Handle design.

Design Goals
- Simple first: UTF-8 strings as (ptr,len), i32 for small integers, 32-bit linear memory alignment friendly.
- Deterministic and portable across WASM/VM/native backends.
- Align with MIR effect system (pure/mut/io/control) to preserve optimization safety.
- Efficient Handle design for Box instances (type_id + instance_id).

Core Types
- i32, i64, f32, f64, bool (0|1)
- string: UTF-8 in linear memory as (ptr: usize, len: usize)
- bytes: Binary data as (ptr: usize, len: usize)
- handle: Efficient Box handle as {type_id: u32, instance_id: u32} or packed u64
- array(T): (ptr: usize, len: usize)
- void (no return value)

Memory & Alignment
- All pointers are platform-dependent (usize): 32-bit in WASM MVP, 64-bit on native x86-64.
- Alignment: 8-byte boundary for all structures.
- Strings/arrays must be contiguous in linear memory; no NUL terminator required (len is authoritative).
- Box layout examples for built-ins (for backends that materialize Boxes in memory):
  - Header: [type_id:i32][ref_count:i32][field_count:i32]
  - StringBox: header + [data_ptr:usize][length:usize]

Handle Design (BID-1 Enhancement)
- Handle represents a Box instance with two components:
  - type_id: u32 (1=StringBox, 6=FileBox, 7=FutureBox, 8=P2PBox, etc.)
  - instance_id: u32 (unique instance identifier)
- Can be passed as single u64 (type_id << 32 | instance_id) or struct

Naming & Resolution
- Interface namespace: `env.console`, `env.canvas`, `nyash.file`, etc.
- Method: `log`, `fillRect`, `fillText`, `open`, `read`, `write`, `close`.
- Fully-qualified name: `env.console.log`, `env.canvas.fillRect`, `nyash.file.open`.

Calling Convention (BID-1)
- Single entry point: `nyash_plugin_invoke`
- Arguments passed as BID-1 TLV format
- Return values in BID-1 TLV format
- Two-call pattern for dynamic results:
  1. First call with null result buffer to get size
  2. Second call with allocated buffer to get data

Error Model (BID-1)
- Standardized error codes:
  - NYB_SUCCESS (0): Operation successful
  - NYB_E_SHORT_BUFFER (-1): Buffer too small
  - NYB_E_INVALID_TYPE (-2): Invalid type ID
  - NYB_E_INVALID_METHOD (-3): Invalid method ID
  - NYB_E_INVALID_ARGS (-4): Invalid arguments
  - NYB_E_PLUGIN_ERROR (-5): Plugin internal error

Effects
- Each BID method declares one of: pure | mut | io | control.
- Optimizer/verifier rules:
  - pure: reordering permitted, memoization possible
  - mut: preserve order w.r.t same resource
  - io: preserve program order strictly
  - control: affects CFG; handled as terminators or dedicated ops

BID-1 TLV Format
```c
// BID-1 TLV specification - unified format for arguments and results
struct BidTLV {
    u16 version;     // 1 (BID-1)
    u16 argc;        // argument count
    // followed by TLVEntry array
};

struct TLVEntry {
    u8 tag;          // type tag
    u8 reserved;     // future use (0)
    u16 size;        // payload size
    // followed by payload data
};

// Tag definitions (Phase 1)
#define BID_TAG_BOOL    1   // payload: 1 byte (0/1)
#define BID_TAG_I32     2   // payload: 4 bytes (little-endian)
#define BID_TAG_I64     3   // payload: 8 bytes (little-endian)
#define BID_TAG_F32     4   // payload: 4 bytes (IEEE 754)
#define BID_TAG_F64     5   // payload: 8 bytes (IEEE 754)
#define BID_TAG_STRING  6   // payload: UTF-8 bytes
#define BID_TAG_BYTES   7   // payload: binary data
#define BID_TAG_HANDLE  8   // payload: 8 bytes (type_id + instance_id)

// Phase 2 reserved
#define BID_TAG_RESULT  20  // Result<T,E>
#define BID_TAG_OPTION  21  // Option<T>
#define BID_TAG_ARRAY   22  // Array<T>
```

Plugin API (BID-1)
```c
// src/bid/plugin_api.h - Plugin API complete version

// Host function table
typedef struct {
    void* (*alloc)(size_t size);        // Memory allocation
    void (*free)(void* ptr);            // Memory deallocation
    void (*wake)(u32 future_id);        // FutureBox wake
    void (*log)(const char* msg);       // Log output
} NyashHostVtable;

// Plugin information
typedef struct {
    u32 type_id;                        // Box type ID
    const char* type_name;              // "FileBox" etc.
    u32 method_count;                   // Method count
    const NyashMethodInfo* methods;     // Method table
} NyashPluginInfo;

typedef struct {
    u32 method_id;                      // Method ID
    const char* method_name;            // "open", "read" etc.
    u32 signature_hash;                 // Type signature hash
} NyashMethodInfo;

// Plugin API (required implementation)
extern "C" {
    // Get ABI version
    u32 nyash_plugin_abi(void);
    
    // Initialize (host integration & metadata registration)
    i32 nyash_plugin_init(const NyashHostVtable* host, NyashPluginInfo* info);
    
    // Unified method invocation
    i32 nyash_plugin_invoke(
        u32 type_id,        // Box type ID
        u32 method_id,      // Method ID  
        u32 instance_id,    // Instance ID
        const u8* args,     // BID-1 TLV arguments
        size_t args_len,    // Arguments size
        u8* result,         // BID-1 TLV result
        size_t* result_len  // Result size (in/out)
    );
    
    // Shutdown
    void nyash_plugin_shutdown(void);
}
```

BID (Box Interface Definition) — YAML
```yaml
version: 1  # BID-1 format
interfaces:
  - name: env.console
    box: Console
    methods:
      - name: log
        params: [ {string: msg} ]
        returns: void
        effect: io

  - name: env.canvas
    box: Canvas
    methods:
      - name: fillRect
        params:
          - {string: canvas_id}
          - {i32: x}
          - {i32: y}
          - {i32: w}
          - {i32: h}
          - {string: color}
        returns: void
        effect: io

      - name: fillText
        params:
          - {string: canvas_id}
          - {string: text}
          - {i32: x}
          - {i32: y}
          - {string: font}
          - {string: color}
        returns: void
        effect: io

  - name: nyash.file
    box: FileBox
    type_id: 6
    methods:
      - name: open
        method_id: 1
        params:
          - {string: path}
          - {string: mode}
        returns: {handle: FileBox}
        effect: io
      
      - name: read
        method_id: 2
        params:
          - {handle: handle}
          - {i32: size}
        returns: {bytes: data}
        effect: io
        
      - name: write
        method_id: 3
        params:
          - {handle: handle}
          - {bytes: data}
        returns: {i32: written}
        effect: io
        
      - name: close
        method_id: 4
        params:
          - {handle: handle}
        returns: void
        effect: io
```

FileBox Plugin Example (BID-1)
```c
// plugins/nyash-file/src/lib.c

#include "nyash_plugin_api.h"
#include <stdio.h>
#include <stdlib.h>

static const NyashHostVtable* g_host = NULL;

// ABI version
u32 nyash_plugin_abi(void) {
    return 1;  // BID-1 support
}

// Method table
static const NyashMethodInfo FILE_METHODS[] = {
    {0, "birth", 0xBEEFCAFE},  // birth(path: string, mode: string) - Constructor
    {1, "open",  0x12345678},  // open(path: string, mode: string) -> Handle
    {2, "read",  0x87654321},  // read(handle: Handle, size: i32) -> Bytes  
    {3, "write", 0x11223344},  // write(handle: Handle, data: Bytes) -> i32
    {4, "close", 0xABCDEF00},  // close(handle: Handle) -> Void
    {5, "fini",  0xDEADBEEF},  // fini() - Destructor
};

// Initialize
i32 nyash_plugin_init(const NyashHostVtable* host, NyashPluginInfo* info) {
    info->type_id = 6;  // FileBox
    info->type_name = "FileBox";
    info->method_count = 4;
    info->methods = FILE_METHODS;
    
    // Save host functions
    g_host = host;
    return NYB_SUCCESS;
}

// Method execution
i32 nyash_plugin_invoke(u32 type_id, u32 method_id, u32 instance_id,
                       const u8* args, size_t args_len,
                       u8* result, size_t* result_len) {
    if (type_id != 6) return NYB_E_INVALID_TYPE;
    
    switch (method_id) {
        case 1: return file_open(args, args_len, result, result_len);
        case 2: return file_read(instance_id, args, args_len, result, result_len);
        case 3: return file_write(instance_id, args, args_len, result, result_len);
        case 4: return file_close(instance_id, args, args_len, result, result_len);
        default: return NYB_E_INVALID_METHOD;
    }
}

// File open implementation
static i32 file_open(const u8* args, size_t args_len, 
                     u8* result, size_t* result_len) {
    // BID-1 TLV parsing
    BidTLV* tlv = (BidTLV*)args;
    if (tlv->version != 1 || tlv->argc != 2) {
        return NYB_E_INVALID_ARGS;
    }
    
    // Extract arguments: path, mode
    const char* path = extract_string_arg(tlv, 0);
    const char* mode = extract_string_arg(tlv, 1);
    
    // Open file
    FILE* fp = fopen(path, mode);
    if (!fp) return NYB_E_PLUGIN_ERROR;
    
    // Generate handle
    u32 handle_id = register_file_handle(fp);
    
    // BID-1 result creation
    if (!result) {
        *result_len = sizeof(BidTLV) + sizeof(TLVEntry) + 8;  // Handle
        return NYB_SUCCESS;
    }
    
    // Return Handle{type_id: 6, instance_id: handle_id} in TLV
    encode_handle_result(result, 6, handle_id);
    return NYB_SUCCESS;
}

// birth implementation - Box constructor
static i32 file_birth(u32 instance_id, const u8* args, size_t args_len,
                     u8* result, size_t* result_len) {
    // Parse constructor arguments
    BidTLV* tlv = (BidTLV*)args;
    const char* path = extract_string_arg(tlv, 0);
    const char* mode = extract_string_arg(tlv, 1);
    
    // Create instance
    FileInstance* instance = malloc(sizeof(FileInstance));
    instance->fp = fopen(path, mode);
    instance->buffer = NULL;
    
    // Register instance
    register_instance(instance_id, instance);
    
    // No return value for birth
    *result_len = 0;
    return NYB_SUCCESS;
}

// fini implementation - Box destructor  
static i32 file_fini(u32 instance_id) {
    FileInstance* instance = get_instance(instance_id);
    if (!instance) return NYB_E_INVALID_HANDLE;
    
    // Free plugin-allocated memory
    if (instance->buffer) {
        free(instance->buffer);
    }
    
    // Close file handle
    if (instance->fp) {
        fclose(instance->fp);
    }
    
    // Free instance
    free(instance);
    unregister_instance(instance_id);
    
    return NYB_SUCCESS;
}

void nyash_plugin_shutdown(void) {
    // Cleanup all remaining instances
    cleanup_all_instances();
}
```

WASM Mapping (RuntimeImports)
- Import examples:
  - `(import "env" "console_log" (func $console_log (param i32 i32)))`  // (ptr,len)
  - `(import "env" "canvas_fillRect" (func $canvas_fillRect (param i32 i32 i32 i32 i32 i32)))`
  - `(import "env" "canvas_fillText" (func $canvas_fillText (param i32 i32 i32 i32 i32 i32 i32 i32)))` // two strings as (ptr,len) each
- Host responsibilities:
  - Resolve strings from memory via `(ptr,len)` using TextDecoder('utf-8')
  - Map to DOM/Canvas/Console as appropriate
  - For plugins: use dlopen/dlsym to load and invoke nyash_plugin_* functions

WASM Mapping Rules (v0)
- String marshalling: UTF-8 `(ptr:i32, len:i32)`; memory exported as `memory`.
- Alignment: `ptr` 4-byte aligned is推奨（必須ではないが実装簡素化のため）。
- Import naming: `env.<iface>_<method>` or nested `env` modules（実装都合でどちらでも可）。
  - 推奨: `env.console_log`, `env.canvas_fillRect`, `env.canvas_fillText`。
- Argument order: 文字列は `(ptr,len)` を1引数扱いで連続配置。複数文字列はその都度 `(ptr,len)`。
- Return: v0では`void`または整数のみ（複合戻りはout-paramに委譲）。
- Memory growth: ホストは`memory.buffer`の再割当を考慮（必要に応じて毎回ビューを取り直す）。

RuntimeImportsとBIDの関係
- `RuntimeImports` は ABI/BID をWASM向けに具体化した実装レイヤー（WASM専用の橋渡し）。
- 生成方針: 将来的にBID（YAML/JSON）から`importObject`と`(import ...)`宣言を自動生成する。
- 例（BID→WASM）:
  - `env.console.log(string msg)` → `console_log(ptr:i32, len:i32)`
  - `env.canvas.fillRect(string canvasId, i32 x, i32 y, i32 w, i32 h, string color)`
    → `canvas_fillRect(id_ptr, id_len, x, y, w, h, color_ptr, color_len)`

============================================================
ABIの確定事項（BID-1, 日本語）
============================================================

基本方針（BID-1）
- 文字列は UTF-8 の `(ptr:usize, len:usize)` で受け渡す（NUL終端不要、内部NUL禁止）。
- 配列/バイト列は `(ptr:usize, len:usize)` とする。
- 数値は WASM/LLVM と親和性の高い素のプリミティブ（i32/i64/f32/f64）。
- 真偽値は `i32` で 0=false, 1=true。
- ポインタは `usize`（WASM MVPは32bit、ネイティブx86-64は64bit）。
- エンディアンはリトルエンディアン（WASM/一般的なネイティブと一致）。
- 呼出規約は単一エントリーポイント `nyash_plugin_invoke` + BID-1 TLV形式。
- 戻り値はBID-1 TLV形式（2回呼び出しパターンでサイズ取得）。
- メモリは `memory` をエクスポート（WASM）。ホスト側で管理。
- 効果（effect）は BID に必須。pure は再順序化可、mut/io は順序保持。
- 同期のみ（非同期は将来拡張）。
- スレッド前提：シングルスレッド（Phase 1）。

メモリ管理戦略
- 2回呼び出しパターン：
  1. result=NULLでサイズ取得
  2. ホストがallocateして結果取得
- 文字列エンコーディング：UTF-8必須、内部NUL禁止
- ハンドル再利用対策：generation追加で ABA問題回避（将来）

Boxライフサイクル管理
- **birth/fini原則**：
  - method_id=0 は必ず`birth()`（コンストラクタ）
  - method_id=最大値 は必ず`fini()`（デストラクタ）
  - birthで割り当てたリソースはfiniで解放
- **メモリ所有権**：
  - プラグインがmalloc()したメモリ → プラグインがfree()
  - ホストが提供したバッファ → ホストが管理
  - 引数として渡されたメモリ → read-onlyとして扱う
- **インスタンス管理**：
  - instance_idはホストが発行・管理
  - プラグインは内部マップでinstance_id → 実装構造体を管理
  - nyash_plugin_shutdown()で全インスタンスをクリーンアップ

型と表現（BID-1）
- `i32`: 32bit 符号付き整数
- `i64`: 64bit 符号付き整数（WASMではJSブリッジ注意。Host側はBigInt等）
- `f32/f64`: IEEE 754
- `bool`: i32（0/1）
- `string`: UTF-8 `(ptr:usize, len:usize)`
- `bytes`: バイナリデータ `(ptr:usize, len:usize)`
- `array<T>`: `(ptr:usize, len:usize)`
- `handle`: Box参照 `{type_id:u32, instance_id:u32}` または packed u64
- `void`: 戻り値なし

BidType Rust実装
```rust
#[derive(Clone, Debug, PartialEq)]
pub enum BidType {
    // プリミティブ（FFI境界で値渡し）
    Bool, I32, I64, F32, F64,
    String, Bytes,
    
    // Handle設計
    Handle { type_id: u32, instance_id: u32 },
    
    // メタ型
    Void,
    
    // Phase 2予約
    Option(Box<BidType>),
    Result(Box<BidType>, Box<BidType>),
    Array(Box<BidType>),
}
```

アラインメント/境界
- `ptr` は 8byte アライン必須（構造体の効率的アクセス）。
- 範囲外アクセスは未定義ではなく「Hostが防ぐ/検証する」方針（将来、Verifier/境界チェック生成）。
- プラットフォーム依存：
  - Linux x86-64: 8バイト境界
  - WASM MVP: 4バイト境界（互換性のため）

命名規約
- `env.console.log`, `env.canvas.fillRect` のように `<namespace>.<iface>.<method>`。
- WASM import 名は `env.console_log` 等の平坦化でも可（生成側で一貫）。
- プラグイン関数名: `nyash_plugin_*` プレフィックス必須。

エラー/例外
- BID-1標準エラーコード使用（NYB_*）。
- 失敗は整数ステータスで返却。
- 例外/シグナルは範囲外。

セキュリティ/権限（将来）
- BID に必要権限（console/canvas/storage/net…）を記述。HostはAllowlistで制御（Phase 9.9）。

実装上の注意点
- ハンドル再利用/ABA: generation追加で回避
- スレッド前提: シングルスレッド前提を明記
- メソッドID衝突: ビルド時固定で回避
- エラー伝播: トランスポート/ドメインエラー分離
- 文字列エンコード: UTF-8必須、内部NUL禁止

============================================================
BIDサンプル（YAML, 日本語）
============================================================

```yaml
version: 0
interfaces:
  - name: env.console
    box: Console
    methods:
      - name: log
        params: [ { string: msg } ]
        returns: void
        effect: io

  - name: env.canvas
    box: Canvas
    methods:
      - name: fillRect
        params:
          - { string: canvas_id }
          - { i32: x }
          - { i32: y }
          - { i32: w }
          - { i32: h }
          - { string: color }
        returns: void
        effect: io

      - name: fillText
        params:
          - { string: canvas_id }
          - { string: text }
          - { i32: x }
          - { i32: y }
          - { string: font }
          - { string: color }
        returns: void
        effect: io
```

ファイルとしてのサンプル（同等内容）
- `docs/nyir/bid_samples/console.yaml`
- `docs/nyir/bid_samples/canvas.yaml`

============================================================
Host側 importObject サンプル（ブラウザ, 日本語）
============================================================

```js
// 文字列(ptr,len)の復元ヘルパ
function utf8FromMemory(memory, ptr, len) {
  const u8 = new Uint8Array(memory.buffer, ptr, len);
  return new TextDecoder('utf-8').decode(u8);
}

const importObject = {
  env: {
    print: (v) => console.log(v),
    print_str: (ptr, len) => {
      console.log(utf8FromMemory(wasmInstance.exports.memory, ptr, len));
    },
    console_log: (ptr, len) => {
      console.log(utf8FromMemory(wasmInstance.exports.memory, ptr, len));
    },
    canvas_fillRect: (idPtr, idLen, x, y, w, h, colorPtr, colorLen) => {
      const mem = wasmInstance.exports.memory;
      const id = utf8FromMemory(mem, idPtr, idLen);
      const color = utf8FromMemory(mem, colorPtr, colorLen);
      const cv = document.getElementById(id);
      if (!cv) return;
      const ctx = cv.getContext('2d');
      ctx.fillStyle = color;
      ctx.fillRect(x, y, w, h);
    },
    canvas_fillText: (idPtr, idLen, textPtr, textLen, x, y, fontPtr, fontLen, colorPtr, colorLen) => {
      const mem = wasmInstance.exports.memory;
      const id = utf8FromMemory(mem, idPtr, idLen);
      const text = utf8FromMemory(mem, textPtr, textLen);
      const font = utf8FromMemory(mem, fontPtr, fontLen);
      const color = utf8FromMemory(mem, colorPtr, colorLen);
      const cv = document.getElementById(id);
      if (!cv) return;
      const ctx = cv.getContext('2d');
      ctx.font = font;
      ctx.fillStyle = color;
      ctx.fillText(text, x, y);
    }
  }
};
```

============================================================
ExternCall → WASM 呼び出しの例（日本語）
============================================================

Nyash コード（概念）:
```
console = new WebConsoleBox("output")
console.log("Hello Nyash!")

canvas = new WebCanvasBox("game-canvas", 400, 300)
canvas.fillRect(50, 50, 80, 60, "red")
```

MIR（ExternCall化のイメージ）:
```
ExternCall { iface: "env.console", method: "log", args: [ string("Hello Nyash!") ] }
ExternCall { iface: "env.canvas", method: "fillRect", args: [ string("game-canvas"), 50, 50, 80, 60, string("red") ] }
```

WASM import 呼び出し（概念）:
```
call $console_log(msg_ptr, msg_len)
call $canvas_fillRect(id_ptr, id_len, 50, 50, 80, 60, color_ptr, color_len)
```

備考
- 文字列定数は data segment に配置し、実行時に (ptr,len) を与える。
- 動的文字列はランタイムでバッファ確保→(ptr,len) を渡す。


VM Mapping (Stub v0)
- Maintain a registry of externs by FQN (e.g., env.console.log) → function pointer.
- Console: print to stdout; Canvas: log params or no-op.

LLVM IR Mapping (Preview)
- Declare external functions with matching signatures (i32/i64/f32/f64/bool, i8* + i32 for strings).
- Example: `declare void @env_console_log(i8* nocapture, i32)`
- Strings allocated in data segment or heap; pass pointer + length.

Versioning
- `version: 0` for the first public draft.
- Backward-compatible extensions should add new methods/imports; breaking changes bump major.

Open Points (to validate post v0)
- Boxref passing across FFI boundaries (opaque handles vs pointers).
- Async externs and scheduling.
- Error model harmonization (status vs result-box).
