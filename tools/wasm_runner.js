#!/usr/bin/env node
/**
 * WASM Runner for Hakorune
 * Phase 15.8: WASM execution test
 */

const fs = require('fs');
const path = require('path');

// Parse command line arguments
const args = process.argv.slice(2);
if (args.length === 0) {
  console.error('Usage: node wasm_runner.js <wasm_file>');
  process.exit(1);
}

const wasmPath = args[0];

// Check file exists
if (!fs.existsSync(wasmPath)) {
  console.error(`Error: File not found: ${wasmPath}`);
  process.exit(1);
}

console.log(`Loading WASM module: ${wasmPath}`);

// Read WASM binary
const wasmBuffer = fs.readFileSync(wasmPath);

// Create WebAssembly memory
const memory = new WebAssembly.Memory({
  initial: 256,  // 256 pages = 16MB
  maximum: 512   // 512 pages = 32MB
});

// Helper: Read string from WASM memory
function readString(ptr, len) {
  const bytes = new Uint8Array(memory.buffer, ptr, len);
  return new TextDecoder().decode(bytes);
}

// Helper: Read i8* string (null-terminated)
function readCString(ptr) {
  const bytes = new Uint8Array(memory.buffer);
  let len = 0;
  while (bytes[ptr + len] !== 0) len++;
  return new TextDecoder().decode(bytes.subarray(ptr, ptr + len));
}

// WASI fd_write implementation
function wasi_fd_write(fd, iovs, iovs_len, nwritten_ptr) {
  const view = new DataView(memory.buffer);
  let nwritten = 0;
  let output = '';

  for (let i = 0; i < iovs_len; i++) {
    const ptr = view.getUint32(iovs + i * 8, true);
    const len = view.getUint32(iovs + i * 8 + 4, true);

    const str = readString(ptr, len);
    output += str;
    nwritten += len;
  }

  // Write to stdout/stderr
  if (fd === 1) {
    process.stdout.write(output);
  } else if (fd === 2) {
    process.stderr.write(output);
  }

  // Store nwritten
  view.setUint32(nwritten_ptr, nwritten, true);
  return 0; // Success
}

// WASI imports (for Phase 15.8)
const importObject = {
  wasi_snapshot_preview1: {
    fd_write: wasi_fd_write,
    proc_exit: (code) => {
      console.log(`[WASI] proc_exit (code=${code})`);
      process.exit(code);
    }
  },
  env: {
    // Linear memory
    __linear_memory: memory,

    // Hakorune console.log (i8* -> i64)
    'nyash.console.log': (ptr) => {
      try {
        const str = readCString(ptr);
        console.log(str);
        return 0n; // Success (BigInt for i64)
      } catch (e) {
        console.error('[nyash.console.log] Error:', e.message);
        return -1n; // Error (BigInt for i64)
      }
    },

    // Boxing helpers (Phase 15.8: stub implementation)
    'nyash.box.from_i8_string': (ptr) => {
      // Convert i8* string to Box handle (stub: just return a dummy handle)
      return 42n; // Dummy StringBox handle
    },

    'nyash.string.len_h': (handle) => {
      // Return length of string (stub)
      return 0n;
    },

    'nyash.string.concat_hh': (h1, h2) => {
      // Concat two string handles (stub)
      return h1; // Just return first handle
    },

    // Safepoint stub (no-op for now)
    ny_check_safepoint: () => {
      // No-op: GC safepoint check
    }
  },

  // NyRT extern calls (Phase 15.8: Registry integration)
  nyrt: {
    // nyrt.time.now_ms: () -> i64
    'time_now_ms': () => {
      // Return milliseconds since UNIX epoch
      return BigInt(Date.now());
    }
  }
};

// Instantiate WASM module
WebAssembly.instantiate(wasmBuffer, importObject)
  .then(result => {
    console.log('✅ WASM module loaded successfully');

    const { instance } = result;
    const exports = instance.exports;

    // List exported functions
    console.log('\nExported functions:');
    for (const name in exports) {
      if (typeof exports[name] === 'function') {
        console.log(`  - ${name}`);
      }
    }

    // Try to call ny_main
    if (exports.ny_main) {
      console.log('\nCalling ny_main()...');
      const result = exports.ny_main();
      // Convert BigInt to Number if needed
      const exitCode = typeof result === 'bigint' ? Number(result) : result;
      console.log(`✅ ny_main() returned: ${exitCode}`);
      process.exit(exitCode);
    } else {
      console.log('\n⚠️  ny_main not found in exports');
      process.exit(0);
    }
  })
  .catch(err => {
    console.error('❌ Error loading WASM module:');
    console.error(err);
    process.exit(1);
  });
