# Evaluation Plan

## E1: 意味論等価性検証

### 目的
全実行バックエンド（Interpreter/VM/JIT/AOT/WASM）で完全に同じ動作を保証

### テストケース

```nyash
// test_equivalence.nyash
box Counter @must_drop {
    init { value }
    
    increment() {
        me.value = me.value + 1
        print("Count: " + me.value)
    }
}

static box Main {
    main() {
        local c = new Counter(0)
        loop(i < 3) {
            c.increment()
        }
        // 自動的にfiniが呼ばれる
    }
}
```

### 検証項目
- [ ] 出力が完全一致
- [ ] fini呼び出し順序が一致
- [ ] エラーハンドリングが一致
- [ ] メモリ使用パターンが同等

### 実行コマンド
```bash
# 各バックエンドで実行
./nyash --backend interpreter test.nyash > interp.log
./nyash --backend vm test.nyash > vm.log
./nyash --backend vm --jit-threshold 1 test.nyash > jit.log
./nyashc --aot test.nyash -o test && ./test > aot.log
./nyashc --wasm test.nyash -o test.wasm && wasmtime test.wasm > wasm.log

# 比較
diff interp.log vm.log
diff vm.log jit.log
diff jit.log aot.log
diff aot.log wasm.log
```

## E2: GCオン/オフ等価性

### 目的
GCの有無でプログラムの意味論が変わらないことを証明

### テストケース

```nyash
box DataHolder @gcable {
    init { data }
    
    process() {
        // 大量のメモリ割り当て
        local temp = new ArrayBox()
        loop(i < 1000000) {
            temp.push(i)
        }
        return temp.length()
    }
}
```

### 測定項目
- I/Oトレース差分: 0
- 最終結果: 同一
- レイテンシ分布: p95/p99で比較

## E3: プラグインオーバーヘッド測定

### 目的
プラグインシステムのオーバーヘッドを定量化

### ベンチマーク

```nyash
// bench_plugin_overhead.nyash
static box Benchmark {
    measure_array_access() {
        local arr = new ArrayBox()
        local sum = 0
        
        // 初期化
        loop(i < 1000000) {
            arr.push(i)
        }
        
        // アクセス性能測定
        local start = new TimeBox().now()
        loop(i < 1000000) {
            sum = sum + arr.get(i)
        }
        local end = new TimeBox().now()
        
        return end - start
    }
}
```

### 比較対象
- ビルトイン実装（現在）
- プラグイン実装（動的リンク）
- プラグイン実装（静的リンク）
- インライン展開後

## E4: スケーラビリティ評価

### 大規模プログラムでの性能

| ベンチマーク | 行数 | Interp | VM | JIT | AOT |
|------------|------|--------|-----|-----|-----|
| json_parser | 500 | 1.0x | ? | ? | ? |
| http_server | 1000 | 1.0x | ? | ? | ? |
| game_engine | 5000 | 1.0x | ? | ? | ? |

### メモリ使用量

```bash
# メモリプロファイリング
valgrind --tool=massif ./nyash --backend vm large_app.nyash
ms_print massif.out.*
```

## E5: プラットフォーム移植性

### テスト環境
- Linux (x86_64, aarch64)
- macOS (x86_64, M1)
- Windows (x86_64)
- WebAssembly (ブラウザ, Wasmtime)

### ビルドスクリプト

```bash
#!/bin/bash
# cross_platform_test.sh

platforms=(
    "x86_64-unknown-linux-gnu"
    "aarch64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-msvc"
    "wasm32-wasi"
)

for platform in "${platforms[@]}"; do
    echo "Building for $platform..."
    cargo build --target $platform --release
    
    # プラグインもビルド
    (cd plugins/nyash-array-plugin && cargo build --target $platform --release)
done
```

## 実験スケジュール

### Phase 1: 基礎評価（1週間）
- E1: 意味論等価性
- E2: GCオン/オフ等価性

### Phase 2: 性能評価（1週間）
- E3: プラグインオーバーヘッド
- E4: スケーラビリティ

### Phase 3: 移植性評価（3日）
- E5: クロスプラットフォーム

### Phase 4: 論文執筆（1週間）
- 結果分析
- グラフ作成
- 考察執筆