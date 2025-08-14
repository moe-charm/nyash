// Node.js用WASM実行テストランナー
const fs = require('fs');

async function runWasm() {
    try {
        // WASMバイナリ読み込み
        const wasmBuffer = fs.readFileSync('test_local_vars.wasm');
        
        // Import関数定義
        const importObject = {
            env: {
                print: (value) => {
                    console.log(`WASM print: ${value}`);
                }
            }
        };
        
        // WASM インスタンス作成・実行
        const wasmModule = await WebAssembly.instantiate(wasmBuffer, importObject);
        
        console.log('🌐 WASM module loaded successfully!');
        
        // main関数実行
        const startTime = performance.now();
        const result = wasmModule.instance.exports.main();
        const endTime = performance.now();
        
        console.log(`🏆 WASM Execution Result: ${result}`);
        console.log(`⚡ WASM Execution Time: ${(endTime - startTime).toFixed(3)} ms`);
        
        return {
            result: result,
            executionTime: endTime - startTime
        };
        
    } catch (error) {
        console.error('❌ WASM execution error:', error);
        return null;
    }
}

// 実行
runWasm().then(result => {
    if (result) {
        console.log(`✅ Test completed - Result: ${result.result}, Time: ${result.executionTime.toFixed(3)}ms`);
    }
});