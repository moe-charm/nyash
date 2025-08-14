# Box FFI/ABI v0 (Draft)

Purpose
- Define a language-agnostic ABI to call external libraries as Boxes.
- Serve as a single source of truth for MIR ExternCall, WASM RuntimeImports, VM stubs, and future language codegens (TS/Python/Rust/LLVM IR).

Design Goals
- Simple first: UTF-8 strings as (ptr,len), i32 for small integers, 32-bit linear memory alignment friendly.
- Deterministic and portable across WASM/VM/native backends.
- Align with MIR effect system (pure/mut/io/control) to preserve optimization safety.

Core Types
- i32, i64, f32, f64, bool (0|1)
- string: UTF-8 in linear memory as (ptr: i32, len: i32)
- boxref: opaque 32-bit handle or pointer (backend-dependent)
- array(T): (ptr: i32, len: i32, [cap: i32 optional])
- void, null (represented as 0 for pointer-like values)

Memory & Alignment
- All pointers are 32-bit in WASM MVP. Align to 4 bytes.
- Strings/arrays must be contiguous in linear memory; no NUL terminator required (len is authoritative).
- Box layout examples for built-ins (for backends that materialize Boxes in memory):
  - Header: [type_id:i32][ref_count:i32][field_count:i32]
  - StringBox: header + [data_ptr:i32][length:i32]

Naming & Resolution
- Interface namespace: `env.console`, `env.canvas`, etc.
- Method: `log`, `fillRect`, `fillText`.
- Fully-qualified name: `env.console.log`, `env.canvas.fillRect`.

Calling Convention (v0)
- Positional parameters, no varargs.
- Strings/arrays passed as (ptr,len) pairs.
- Return values:
  - Single scalar (i32/i64/f32/f64/bool) or void.
  - Box/complex returns are out-of-scope for v0; use out-params if necessary.

Error Model (v0)
- Prefer total functions for v0 demos.
- If needed, return i32 status (0=ok, nonzero=error) and use out-params.
- Exceptions/signals are out-of-scope.

Effects
- Each BID method declares one of: pure | mut | io | control.
- Optimizer/verifier rules:
  - pure: reordering permitted
  - mut: preserve order w.r.t same resource
  - io: preserve program order
  - control: affects CFG; handled as terminators or dedicated ops

BID (Box Interface Definition) — YAML
```yaml
version: 0
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
```

WASM Mapping (RuntimeImports)
- Import examples:
  - `(import "env" "console_log" (func $console_log (param i32 i32)))`  // (ptr,len)
  - `(import "env" "canvas_fillRect" (func $canvas_fillRect (param i32 i32 i32 i32 i32 i32)))`
  - `(import "env" "canvas_fillText" (func $canvas_fillText (param i32 i32 i32 i32 i32 i32 i32 i32)))` // two strings as (ptr,len) each
- Host responsibilities:
  - Resolve strings from memory via `(ptr,len)` using TextDecoder('utf-8')
  - Map to DOM/Canvas/Console as appropriate

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
ABIの確定事項（v0, 日本語）
============================================================

基本方針（v0）
- 文字列は UTF-8 の `(ptr:i32, len:i32)` で受け渡す（NUL終端不要）。
- 配列/バイト列は `(ptr:i32, len:i32[, cap:i32])` とし、v0では `(ptr,len)` を基本とする。
- 数値は WASM/LLVM と親和性の高い素のプリミティブ（i32/i64/f32/f64）。
- 真偽値は `i32` で 0=false, 1=true。
- ポインタは `i32`（WASM MVP）を基本。ネイティブAOTではプラットフォーム幅に合わせる（将来）。
- エンディアンはリトルエンディアン（WASM/一般的なネイティブと一致）。
- 呼出規約は位置パラメータのみ（可変長/キーワードは範囲外）。
- 戻り値は単一スカラ（void含む）。複合は out-param で表現（将来拡張）。
- メモリは `memory` をエクスポート（WASM）。`TextDecoder('utf-8')` 等で復元（Host側責務）。
- 効果（effect）は BID に必須。pure は再順序化可、mut/io は順序保持。
- 同期のみ（非同期は将来拡張）。

型と表現（v0）
- `i32`: 32bit 符号付き整数
- `i64`: 64bit 符号付き整数（WASMではJSブリッジ注意。Host側はBigInt等）
- `f32/f64`: IEEE 754
- `bool`: i32（0/1）
- `string`: UTF-8 `(ptr:i32, len:i32)`
- `array<T>`: `(ptr:i32, len:i32[, cap:i32])`（v0は `(ptr,len)` を優先）
- `boxref`: Opaque（数値ハンドル or ポインタ）。v0では数値 i32 を推奨。

アラインメント/境界
- `ptr` は 4byte アライン推奨（必須ではないが実装が簡潔）。
- 範囲外アクセスは未定義ではなく「Hostが防ぐ/検証する」方針（将来、Verifier/境界チェック生成）。

命名規約
- `env.console.log`, `env.canvas.fillRect` のように `<namespace>.<iface>.<method>`。
- WASM import 名は `env.console_log` 等の平坦化でも可（生成側で一貫）。

エラー/例外
- v0は例外なし。失敗は整数ステータス or 明示エラーコールバックに委譲（将来）。

セキュリティ/権限（将来）
- BID に必要権限（console/canvas/storage/net…）を記述。HostはAllowlistで制御（Phase 9.9）。

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
