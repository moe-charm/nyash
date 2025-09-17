Nyash Plugin ABI v2 (TypeBox)

Overview
- Purpose: unify plugin invocation via per‑Box TypeBox exports and a simple TLV wire format.
- Design: loader resolves a per‑Box dispatch function once, then calls by `(instance_id, method_id, TLV args) → TLV result`.
- Stability: v2 opts for simplicity over backward compatibility. Migrate plugins to v2; legacy C ABI entry remains available only for bootstrapping during transition.

Error Codes
- 0: OK
- -1: E_SHORT (output buffer too small; two‑phase protocol)
- -2: E_TYPE (invalid type for call)
- -3: E_METHOD (unknown method id)
- -4: E_ARGS (invalid/malformed args)
- -5: E_PLUGIN (internal plugin error/missing TypeBox)
- -8: E_HANDLE (invalid instance handle)

TLV Encoding (version=1)
- Header: 2 bytes `u16` version (1), 2 bytes `u16` argc
- Entries: repeated blocks [tag: u8, reserved: u8=0, size: u16 LE, payload: [size]]
- Tags
  - 1: bool (size=1; 0/1)
  - 2: i32 (size=4; LE)
  - 3: i64 (size=8; LE)
  - 4: f32 (size=4; LE)
  - 5: f64 (size=8; LE)
  - 6: string (UTF‑8)
  - 7: bytes
  - 8: plugin handle (type_id:u32 + instance_id:u32)
  - 9: host handle (u64)

Two‑Phase Result Protocol
- On first call, host may pass `result=NULL` or a small buffer.
- Plugin returns `E_SHORT` and writes required size to `*result_len`.
- Host re‑allocates and retries; on success return 0 and fill result.

TypeBox Export
- Each plugin Box publishes a single symbol: `nyash_typebox_<BoxName>` with C layout:

  struct NyashTypeBoxFfi {
    uint32_t abi_tag;     // 'TYBX' = 0x54594258
    uint16_t version;     // 1
    uint16_t struct_size; // sizeof(NyashTypeBoxFfi)
    const char* name;     // "FileBox\0", etc.
    uint32_t (*resolve)(const char* method_name); // optional
    int32_t  (*invoke_id)(uint32_t instance_id, uint32_t method_id,
                          const uint8_t* args, size_t args_len,
                          uint8_t* out, size_t* out_len);
    uint64_t capabilities; // reserved
  };

Naming Conventions
- Symbol: `nyash_typebox_<Box>` where `<Box>` is the Box type (e.g., `FileBox`, `RegexBox`).
- Methods: prefer lowerCamelCase in resolve names (e.g., `setStatus`, `readBody`, `isMatch`).
- Boxes live in one shared library and are listed under `[libraries."<libname>"].boxes` in `nyash.toml`.

Examples
- RegexBox (methods: birth/compile/isMatch/find/replaceAll/split/fini)
  - resolve → method_id → invoke_id: decode TLV → run regex → encode TLV
  - Export `nyash_typebox_RegexBox` with `resolve` and `invoke_id` pointers.

- Net (Client/Response/Request minimal)
  - ClientBox: `get(url)`, `post(url, body)` → returns Handle(Response)
  - ResponseBox: `setStatus(i32)`, `setHeader(name,value)`, `write(bytes|string)`, `getStatus()`, `readBody()`
  - RequestBox: `path()`, `readBody()`, `respond(Response)`
  - Each Box exports its own TypeBox symbol (e.g., `nyash_typebox_ClientBox`).

nyash.toml Layout
- Declare libraries and per‑Box metadata:

  [libraries]
  [libraries."libnyash_regex_plugin.so"]
  boxes = ["RegexBox"]
  path = "plugins/nyash-regex-plugin/target/release/libnyash_regex_plugin.so"

  [libraries."libnyash_regex_plugin.so".RegexBox]
  type_id = 52
  abi_version = 1
  [libraries."libnyash_regex_plugin.so".RegexBox.methods]
  birth = { method_id = 0 }
  compile = { method_id = 1 }
  isMatch = { method_id = 2 }

Error Handling Patterns
- Two‑phase output (E_SHORT) is mandatory for large or variable results.
- For invalid handles/args, return `-8` / `-4` respectively; host commonly wraps these into `ResultBox` in higher layers.

Invoke Semantics
- Birth: method_id=0, instance_id=0. Returns 4‑byte LE instance_id (not TLV).
- Fini: method_id=UINT32_MAX, instance_id=target. Should release resources and return OK with TLV void or 0‑entry TLV.
- Regular methods: return TLV payloads per method contract.

Host Expectations
- Host looks up per‑Box function from the TypeBox symbol and calls it via a library‑level shim.
- If no TypeBox is exported for a box, the call fails with `E_PLUGIN`.
- Method ids and type ids are configured in `nyash.toml` under `[libraries]`.

Minimal C Header
- See `include/nyash_abi.h` for a ready‑to‑use header and constants.

Notes
- Keep result payloads small; prefer handles (tag=8) for large data and stream APIs.
- Use UTF‑8 strings (tag=6) for human‑readable values; use bytes (tag=7) otherwise.

Phase‑12 Alignment & Roadmap
- Alignment: The Phase‑12 “Unified TypeBox ABI” is a superset of this v2 minimal design. It shares the same core shape: per‑Box TypeBox export, method_id dispatch, and TLV arguments/results.
- Current focus (v2 minimal): simplicity and portability for first‑party plugins (Regex/Net/File/Path/Math/Time/Python family). The loader probes `nyash_typebox_<Box>` and calls `invoke_id` with `(instance_id, method_id, TLV)`.
- Forward compatibility: hosts validate `struct_size` and `version`. Future minor extensions can add optional fields to NyashTypeBoxFfi while preserving existing layout. Keep `capabilities=0` for now.
- Planned extensions (Phase‑16/17 candidates):
  - create/destroy function pointers (birth/fini remain for backward safety)
  - get_type_info() style metadata discovery (can be added via a resolved method first)
  - method table slotting and stricter signature IDs for faster dispatch
  - NyValue bridge and BoxHeader/vtable integration for RC/GC cooperation and JIT IC
- Guidance: implementers should target v2 minimal today. When Phase‑12 features land, they will be additive, with migration guides and host shims to keep existing v2 plugins working.
