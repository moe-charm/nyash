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

        // Find and call ny_main
        if (instance.exports.ny_main) {
            console.log('🚀 Calling ny_main()...');
            const result = instance.exports.ny_main();
            console.log(`✅ ny_main() returned: ${result}`);
            return result;
        } else {
            console.error('❌ Error: ny_main not found in exports');
            console.log('Available exports:', Object.keys(instance.exports));
            process.exit(1);
        }
    } catch (error) {
        console.error('❌ Error running WASM:', error);
        process.exit(1);
    }
}

runWasm();
