#!/usr/bin/env node
// WASM Runner for Nyash/Hakorune WASM binaries

const fs = require('fs');
const path = require('path');

if (process.argv.length < 3) {
    console.error('Usage: node wasm_runner.js <file.wasm>');
    process.exit(1);
}

const wasmPath = process.argv[2];

// WASI runtime implementation
const wasiRuntime = {
    // fd_write(fd, iovs, iovs_len, nwritten)
    fd_write: (fd, iovs, iovs_len, nwritten) => {
        // Simple implementation: just print to stdout
        return 0n;
    },

    // proc_exit(code)
    proc_exit: (code) => {
        console.log(`[WASI] Process exited with code: ${code}`);
        process.exit(Number(code));
    },

    // ny_safepoint(count, live_vals)
    ny_check_safepoint: (count, live_vals) => {
        // No-op for now
        return 0n;
    }
};

// Nyash runtime functions
const nyashRuntime = {
    'nyash.console.log': (strPtr) => {
        // For now, just print the pointer value
        console.log(`[Nyash] console.log called with ptr: ${strPtr}`);
        return 0n;
    },
    'nyash.box.from_i8_string': (ptr) => {
        // Stub: return handle (just the pointer)
        return BigInt(ptr);
    },
    'nyash.string.concat_hh': (h1, h2) => {
        // Stub: return first handle
        return h1;
    },
    'nyash.string.to_i8p_h': (handle) => {
        // Stub: convert i64 handle to i8* pointer (just return as number)
        return Number(handle);
    }
};

// Load and run WASM
async function runWasm() {
    try {
        const wasmBuffer = fs.readFileSync(wasmPath);

        const wasmModule = await WebAssembly.compile(wasmBuffer);

        // Create linear memory (16MB = 256 pages)
        const memory = new WebAssembly.Memory({ initial: 256, maximum: 256 });

        const instance = await WebAssembly.instantiate(wasmModule, {
            env: {
                __linear_memory: memory,
                ...wasiRuntime,
                ...nyashRuntime
            }
        });

        // Try ny_main first, then Main.main, then main, then test_fn
        let entryFunc = null;
        let entryName = null;

        if (instance.exports.ny_main) {
            entryFunc = instance.exports.ny_main;
            entryName = 'ny_main';
        } else if (instance.exports['Main.main']) {
            entryFunc = instance.exports['Main.main'];
            entryName = 'Main.main';
        } else if (instance.exports.test_fn) {
            entryFunc = instance.exports.test_fn;
            entryName = 'test_fn';
        } else if (instance.exports.main) {
            entryFunc = instance.exports.main;
            entryName = 'main';
        }

        if (entryFunc) {
            console.log(`🚀 Calling ${entryName}()...`);
            const result = entryFunc();
            console.log(`✅ ${entryName}() returned: ${result}`);
            // Unify exit behavior: map return value to process exit code when available.
            try {
                const ec = (typeof result === 'number' ? (result|0) : 0) & 0xFF;
                if (typeof process !== 'undefined' && process && typeof process.exit === 'function') {
                    process.exit(ec);
                }
            } catch (_) {
                // ignore and just return
            }
            return result;
        } else {
            console.error('❌ Error: No entry point found (tried: ny_main, Main.main, main)');
            console.log('Available exports:', Object.keys(instance.exports));
            process.exit(1);
        }
    } catch (error) {
        console.error('❌ Error running WASM:', error);
        process.exit(1);
    }
}

runWasm();
