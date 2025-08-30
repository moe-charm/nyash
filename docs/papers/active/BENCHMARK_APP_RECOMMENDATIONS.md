# ベンチマークアプリ推奨案（AI会議結果）

Date: 2025-08-31
相談先: Gemini + Codex

## 🎯 AI先生たちの推奨ベンチマーク

### 📊 Gemini先生の推奨（論文・実用重視）

#### Tier 1: 論文の主役候補
1. **レイトレーサー (Ray Tracer)**
   - **理由**: CPU負荷高、JIT/AOT効果が劇的
   - **効果**: Interpreter vs VM vs LLVM の性能差を視覚的に証明
   - **規模**: 200-300行

2. **Lispインタープリター**
   - **理由**: 「Everything is Box」の優位性を完璧に証明
   - **効果**: 動的型付け言語での統一モデルの威力
   - **規模**: 300-400行

#### Tier 2: 実用性アピール
3. **静的サイトジェネレータ**
   - **理由**: Markdownパーサー + HTMLコンパイラ
   - **効果**: Fileプラグイン、文字列処理、実用性
   - **規模**: 400-500行

4. **REST APIサーバー**
   - **理由**: 現代的アプリ開発での適性証明
   - **効果**: Netプラグイン、JSON操作、並行性
   - **規模**: 300-400行

### 📈 Codex先生の推奨（ベンチマーク標準重視）

#### CLBG（Computer Language Benchmarks Game）定番
1. **n-body** - 数値計算の定番
2. **spectral-norm** - 行列・ベクトル演算
3. **mandelbrot** - 純粋計算 + 画像出力
4. **fannkuch-redux** - 組み合わせ計算
5. **k-nucleotide** - 文字列・ハッシュ処理
6. **binary-trees** - メモリ割り当て・GC性能

#### 現実的なアプリ（Nyash特化）
7. **JSON/CSV Stream Aggregator**
   - File/Netプラグインから同じコードで処理
   - 「Everything is Box」の威力を実証

## 🚀 推奨実装順序（論文説得力重視）

### Phase 1: CLBG標準ベンチマーク（信頼性）
```bash
# 1. binary-trees（メモリ・GC基準）
# 2. n-body（数値計算）  
# 3. mandelbrot（計算 + I/O）
```

### Phase 2: Nyash特色ベンチマーク（独自性）
```bash
# 4. JSON Stream Aggregator（プラグイン統一）
# 5. レイトレーサー（視覚的インパクト）
```

### Phase 3: 実用アプリ（実践性）
```bash
# 6. Lispインタープリター（言語能力）
# 7. 静的サイトジェネレータ（実用性）
```

## 📊 期待される性能差

### Interpreter → VM → LLVM
- **n-body**: 1x → 10x → 50x（数値計算）
- **mandelbrot**: 1x → 15x → 80x（ループ最適化）
- **binary-trees**: 1x → 8x → 20x（GC最適化）
- **JSON Stream**: 1x → 12x → 30x（プラグイン + ループ）

## 🎯 AOTスモークテストの具体例

ChatGPT5さんが言う「代表サンプルのAOTスモーク」：

### Array get/set スモーク
```nyash
// smoke_array_getset.nyash
local arr = new ArrayBox()
arr.set(0, 42)
arr.set(1, 100)
local result = arr.get(0) + arr.get(1)
print("Result: " + result)  // Expected: Result: 142
```

### Console.log スモーク
```nyash
// smoke_console_log.nyash  
local console = new ConsoleBox()
console.log("Hello from LLVM!")
print("Result: success")
```

### 実行・比較
```bash
# VM実行
./nyash --backend vm smoke_array_getset.nyash > vm_out.txt

# LLVM AOT実行
./nyash --compile-native smoke_array_getset.nyash -o test_app
./test_app > aot_out.txt

# 結果比較（一致すればOK）
diff vm_out.txt aot_out.txt
```

## 💡 論文での活用方法

1. **Section 5: Evaluation**
   - CLBGベンチマークで他言語と性能比較
   - Nyash特色ベンチマークで独自性証明

2. **Section 6: Case Studies**
   - レイトレーサーでJIT/LLVM効果の視覚化
   - Stream Aggregatorで「Everything is Box」実証

3. **Appendix: Reproducibility**
   - 全ベンチマークコードを掲載
   - 実行手順を詳細に記載

これで論文の説得力とインパクトが大幅に向上しますね！