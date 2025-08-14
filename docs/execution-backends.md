# Nyash実行バックエンド完全ガイド

Nyashプログラミング言語は、**Everything is Box**哲学を維持しながら、3つの異なる実行方式をサポートしています。用途に応じて最適な実行方式を選択できます。

## 🚀 実行方式一覧

| 実行方式 | 用途 | 特徴 | パフォーマンス |
|---------|------|------|---------------|
| **インタープリター** | 開発・デバッグ | 直接AST実行、詳細ログ | 低速・高機能 |
| **VM** | 本番・高速実行 | MIR→VM実行 | 中速・最適化 |
| **WASM** | Web・サンドボックス | MIR→WASM変換 | 高速・移植性 |

## 📋 CLIオプション

### 基本実行（インタープリター）
```bash
# デフォルト：インタープリター実行
nyash program.nyash

# デバッグ燃料制限付き
nyash --debug-fuel 50000 program.nyash

# 無制限デバッグ燃料
nyash --debug-fuel unlimited program.nyash
```

### VM実行
```bash
# VM実行（高速）
nyash --backend vm program.nyash
```

### MIR操作
```bash
# MIR表示（中間表現確認）
nyash --dump-mir program.nyash

# MIR検証
nyash --verify program.nyash

# 詳細MIR情報
nyash --mir-verbose --dump-mir program.nyash
```

### WASM生成・実行
```bash
# WASMコンパイル（WAT出力）
nyash --compile-wasm program.nyash

# ファイル出力
nyash --compile-wasm program.nyash -o output.wat

# ブラウザで実行可能なWASMを生成
nyash --compile-wasm program.nyash -o public/app.wat
```

### ⚡ ベンチマーク（パフォーマンス測定）
```bash
# 全バックエンド性能比較（デフォルト5回実行）
nyash --benchmark

# 実行回数指定（統計精度向上）
nyash --benchmark --iterations 100

# 結果をファイル保存
nyash --benchmark --iterations 50 > benchmark_results.txt
```

## 🎯 インタープリター（デフォルト）

### 特徴
- **用途**: 開発・デバッグ・学習
- **実行**: AST直接実行
- **速度**: 最も低速
- **機能**: 最も詳細なデバッグ情報

### 利点
- 詳細な実行ログ
- エラー位置の正確な特定
- リアルタイム変数監視
- メモリ使用量詳細表示

### デバッグ燃料システム
```bash
# パーサー無限ループ対策
nyash --debug-fuel 10000 problem.nyash

# エラー例:
🚨 PARSER INFINITE LOOP DETECTED at method call argument parsing
🔍 Current token: IDENTIFIER("from") at line 17
```

## 🏎️ VM実行（高速）

### 特徴
- **用途**: 本番実行・性能重視
- **実行**: AST→MIR→VM実行
- **速度**: 中〜高速
- **機能**: 最適化済み

### 実行パイプライン
```
Nyashソース → AST → MIR → VM → 結果
```

### MIR（中間表現）
```bash
# MIR確認
nyash --dump-mir simple.nyash

# 出力例:
; MIR Module: main
define void @main() {
bb0:
    0: safepoint
    1: %0 = const 42
    2: %1 = const 8
    3: %2 = %0 Add %1
    4: print %2
    5: ret %2
}
```

### VMの特徴
- **SSA形式**: 静的単一代入
- **基本ブロック**: 制御フロー最適化
- **効果追跡**: 副作用の管理
- **型安全**: 実行時型チェック

## 🌐 WASM実行（Web対応）

### 特徴
- **用途**: Webブラウザ・サンドボックス実行
- **実行**: AST→MIR→WASM→ブラウザ
- **速度**: 最高速（ネイティブ並み）
- **移植性**: 全プラットフォーム対応

### 実行パイプライン
```
Nyashソース → AST → MIR → WAT → WASM → ブラウザ
```

### 生成例
```nyash
// Nyashコード
static box Main {
    main() {
        return 42
    }
}
```

```wat
; 生成されるWAT
(module
  (import "env" "print" (func $print (param i32) ))
  (memory (export "memory") 1)
  (global $heap_ptr (mut i32) (i32.const 2048))
  (func $main (local $0 i32)
    nop             ; safepoint
    i32.const 42    ; const 42
    local.set $0    ; store to local
    local.get $0    ; load from local
    return          ; return 42
  )
  (export "main" (func $main))
)
```

### Web実行
```html
<!-- HTMLで読み込み -->
<script>
async function loadNyashWasm() {
    const response = await fetch('output.wat');
    const watText = await response.text();
    
    const wabt = await WabtModule();
    const module = wabt.parseWat('output.wat', watText);
    const binary = module.toBinary({});
    
    const importObject = {
        env: { print: console.log }
    };
    
    const wasmModule = await WebAssembly.instantiate(binary.buffer, importObject);
    const result = wasmModule.instance.exports.main(); // 42
}
</script>
```

## 📊 パフォーマンス比較

### 🚀 実際のベンチマーク結果（2025-08-14測定・修正）

#### ⚠️ **重要**: 性能測定の正確な説明

**真の実行性能比較**（wasmtime統合・100回実行平均）:
| Backend | 実行時間 | 速度比 | 測定内容 | 最適用途 |
|---------|---------|---------|----------|----------|
| **🌐 WASM** | **8.12ms** | **13.5x faster** | 真の実行性能 | Web配布・高速実行 |
| **📝 Interpreter** | **110.10ms** | **1x (baseline)** | AST直接実行 | 開発・デバッグ |
| **🏎️ VM** | **119.80ms** | **0.9x slower** | MIR→VM実行 | 🚨要改善 |

**コンパイル性能参考**（従来のベンチマーク）:
| Backend | コンパイル時間 | 速度比 | 測定内容 |
|---------|-------------|---------|----------|
| **🌐 WASM** | **0.17ms** | **280x faster** | MIR→WASM変換 |
| **🏎️ VM** | **16.97ms** | **2.9x faster** | MIR→VM変換 |
| **📝 Interpreter** | **48.59ms** | **1x (baseline)** | AST→実行 |

### 📈 ベンチマーク詳細

#### 🚨 **VM性能問題の発見**
**異常事象**: VMがインタープリターより遅い結果が判明
- **推定原因**: MIR変換オーバーヘッド、VM実行エンジン未最適化
- **対策**: Phase 9でのJIT化、VM最適化が急務

#### 実行性能詳細（wasmtime統合測定）
```
🌐 WASM (wasmtime):  8.12 ms   (13.5x faster - 真の実行性能)
📝 Interpreter:     110.10 ms  (1x baseline)
🏎️ VM:              119.80 ms  (0.9x slower - 要改善)
```

#### コンパイル性能詳細（従来測定）
```
🌐 WASM変換:   0.15-0.21 ms  (280x faster - コンパイル速度)
🏎️ VM変換:    4.44-25.08 ms (3-120x faster - コンパイル速度)
📝 実行のみ:  14.85-84.88 ms (1x baseline)
```

### 💡 ベンチマーク実行方法
```bash
# 現在のマシンで性能測定
nyash --benchmark --iterations 100

# 軽量テスト（開発中）
nyash --benchmark --iterations 10
```

### メモリ使用量
```
インタープリター ████████████████████ 高い（AST+実行情報）
VM             ████████████          中程度（MIR+実行時）
WASM           ████                  低い（最適化済み）
```

## 🎁 Everything is Box の維持

全ての実行方式で、Nyashの核心哲学「Everything is Box」が維持されます：

### インタープリター
```rust
// RustのArc<Mutex<dyn NyashBox>>として実装
StringBox::new("Hello") → Arc<Mutex<StringBox>>
```

### VM
```
// MIRのValueIdとして管理
%0 = const "Hello"    ; StringBox相当
%1 = %0.length()      ; メソッド呼び出し
```

### WASM
```wat
;; WASMの線形メモリでBox表現
;; [type_id:4][field_count:4][field0:4][field1:4]...
i32.const 1001        ;; StringBox type ID
i32.store offset=0    ;; メモリにBox情報格納
```

## 🚀 用途別推奨

### 開発・デバッグ時
```bash
# 詳細ログでエラー特定
nyash --debug-fuel unlimited debug_me.nyash
```

### 本番実行時
```bash
# 高速・安定実行
nyash --backend vm production.nyash
```

### Web配布時
```bash
# ブラウザ対応WASM生成
nyash --compile-wasm app.nyash -o public/app.wat
```

## 🔧 トラブルシューティング

### パーサーエラー
```bash
# 無限ループ検出時
🚨 PARSER INFINITE LOOP DETECTED
→ nyash --debug-fuel 1000 problem.nyash
```

### MIRエラー
```bash
# 未対応AST構文
❌ MIR compilation error: Unsupported AST node type: BoxDeclaration
→ 現在はstatic box Mainのみ対応
```

### WASMエラー
```bash
# 未対応MIR命令
❌ WASM compilation error: Instruction not yet supported: ComplexInstruction
→ Phase 8.3で順次対応予定
```

## 📈 今後の拡張予定

### Phase 8.3: Box操作のWASM対応
- RefNew/RefGet/RefSet
- オブジェクト指向プログラミング
- メモリ管理の高度化

### Phase 8.4: 非同期処理のWASM対応
- nowait/await構文
- Future操作
- 並列処理

### Phase 8.5: 最適化
- デッドコード除去
- インライン展開
- ループ最適化

---

**💡 Tip**: 開発中は**インタープリター**、テスト時は**VM**、配布時は**WASM**という使い分けが効果的です！

最終更新: 2025-08-14
作成者: Nyash Development Team