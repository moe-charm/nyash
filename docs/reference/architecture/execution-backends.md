# Nyash実行バックエンド完全ガイド

Nyashプログラミング言語は、**Everything is Box**哲学を維持しながら、4つの異なる実行方式をサポートしています。用途に応じて最適な実行方式を選択できます。

## 🚀 実行方式一覧

| 実行方式 | 用途 | 特徴 | パフォーマンス |
|---------|------|------|---------------|
| **インタープリター** | 開発・デバッグ | 直接AST実行、詳細ログ | 低速・高機能 |
| **VM** | 本番・高速実行 | MIR→VM実行 | 中速・最適化 |
| **WASM** | Web・サンドボックス | MIR→WASM変換 | 高速・移植性 |
| **AOT** (実験的) | 高速起動 | MIR→WASM→ネイティブ | 最高速・要wasmtime |

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

### AOT/ネイティブコンパイル（実験的機能）
```bash
# ビルド時に wasm-backend feature が必要
cargo build --release --features wasm-backend

# AOTコンパイル（.cwasm生成）
nyash --compile-native program.nyash -o program
# または
nyash --aot program.nyash -o program

# 注意: 現在は完全なスタンドアロン実行ファイルではなく、
# wasmtime用のプリコンパイル済みWASM（.cwasm）が生成されます
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
 - **対応状況**: 命令ごとの実装は「MIR → VM Mapping」を参照（欠落・暫定箇所の把握に）
   - docs/reference/architecture/mir-to-vm-mapping.md

### 🧮 VM実行統計（命令カウント・時間計測）
VMバックエンドは命令ごとの実行回数と総実行時間(ms)を出力できます。

有効化方法（CLI推奨）:
```bash
# 人間向け表示
nyash --backend vm --vm-stats program.nyash

# JSON出力（機械可読）
nyash --backend vm --vm-stats --vm-stats-json program.nyash
```

環境変数でも制御可能:
```bash
NYASH_VM_STATS=1 ./target/release/nyash --backend vm program.nyash
NYASH_VM_STATS=1 NYASH_VM_STATS_JSON=1 ./target/release/nyash --backend vm program.nyash
# もしくは NYASH_VM_STATS_FORMAT=json でも可
```

JSON出力例:
```json
{
  "total": 1234,
  "elapsed_ms": 5.123,
  "counts": { "Const": 200, "BinOp": 300, "Return": 100 },
  "top20": [ { "op": "BinOp", "count": 300 } ],
  "timestamp_ms": 1724371200000
}
```

ベンチマークと併用して、ホット命令の抽出・命令セット最適化に活用できます。

### ⏱️ 協調スケジューラ（Phase 10.6b）
- VMはMIRの`safepoint`命令到達時にランタイムのスケジューラ`poll()`を呼びます。
- シングルスレ実装（既定）では、`spawn`/`spawn_after`で投入されたタスクを safepoint ごとに最大N件実行します。
- 制御: `NYASH_SCHED_POLL_BUDGET`（既定: 1）でNを指定。

デモ実行:
```bash
cargo build --release -j32
NYASH_SCHED_DEMO=1 NYASH_SCHED_POLL_BUDGET=2 \
  ./target/release/nyash --backend vm examples/scheduler_demo.nyash
```

### 🧹 GCトレーシング（Phase 10.4）
- カウンタ有効化: `NYASH_GC_COUNTING=1`（CountingGcを注入）
- 出力レベル: `NYASH_GC_TRACE=1/2/3`
  - 1: safepoint/barrierログ＋カウンタ
  - 2: + ルート内訳
  - 3: + depth=2 リーチャビリティ概要
- 厳格検証: `NYASH_GC_BARRIER_STRICT=1`（Write-Barrier未増分ならpanic）

```bash
NYASH_GC_COUNTING=1 NYASH_GC_TRACE=2 \
  ./target/release/nyash --backend vm examples/scheduler_demo.nyash
```

#### Boxからの切替（GcConfigBox）
環境変数ではなくNyashスクリプトから切り替える場合は `GcConfigBox` を使います。`apply()` で環境に反映され、その後の実行に適用されます。

```nyash
// 最小例: CountingGc + trace をオン
static box Main {
  main() {
    local G
    G = new GcConfigBox()
    G = G.setFlag("counting", true)
    G = G.setFlag("trace", true)   // 1/2/3 は環境。Box では on/off を切替
    G.apply()                       // ← ここで反映
    return "ok"
  }
}
```

代表デモ（書き込みバリア/カウンタの可視化）:
```bash
./target/release/nyash --backend vm examples/gc_counting_demo.nyash
```

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

## 🚀 AOT/ネイティブコンパイル（実験的）

### 特徴
- **用途**: 高速起動・配布用実行ファイル
- **実行**: AST→MIR→WASM→プリコンパイル済みネイティブ
- **速度**: 最高速（JIT起動オーバーヘッドなし）
- **制約**: wasmtimeランタイムが必要

### コンパイルパイプライン
```
Nyashソース → AST → MIR → WASM → .cwasm（プリコンパイル済み）
```

### ビルド方法
```bash
# 1. wasm-backend feature付きでNyashをビルド
cargo build --release --features wasm-backend

# 2. AOTコンパイル実行
./target/release/nyash --compile-native hello.nyash -o hello
# または短縮形
./target/release/nyash --aot hello.nyash -o hello

# 3. 生成されたファイル
# hello.cwasm が生成される（wasmtimeプリコンパイル形式）
```

### 現在の実装状況
- ✅ MIR→WASM変換
- ✅ WASM→.cwasm（wasmtimeプリコンパイル）
- ❌ 完全なスタンドアロン実行ファイル生成（TODO）
- ❌ ランタイム埋め込み（将来実装予定）

### 使用例
```bash
# コンパイル
./target/release/nyash --aot examples/hello_world.nyash -o hello_aot

# 実行（将来的な目標）
# ./hello_aot  # 現在は未実装

# 現在は wasmtime で実行
# wasmtime --precompiled hello_aot.cwasm
```

### 技術的詳細
AOTバックエンドは内部的に以下の処理を行います：
1. **MirCompiler**: NyashコードをMIRに変換
2. **WasmBackend**: MIRをWASMバイトコードに変換
3. **wasmtime::Engine::precompile**: WASMをネイティブコードにプリコンパイル
4. **出力**: .cwasm形式で保存（wasmtime独自形式）

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
### 🔥 JIT実行（Phase 10_c 最小経路）
- 有効化: `NYASH_JIT_EXEC=1` とし、`NYASH_JIT_THRESHOLD=1` でホット判定しきい値を下げる
- 追加情報: `NYASH_JIT_STATS=1` でJITコンパイル/実行時間、サイト集計を出力
- ダンプ: `NYASH_JIT_DUMP=1` でLowerカバレッジ/emit統計を表示
- HostCall（配列/Map最小）: `NYASH_JIT_HOSTCALL=1`

例:
```bash
NYASH_JIT_EXEC=1 NYASH_JIT_THRESHOLD=1 NYASH_JIT_HOSTCALL=1 NYASH_JIT_STATS=1 \
  ./target/release/nyash --backend vm examples/scheduler_demo.nyash
```

現状のカバレッジ（Core-1）
- Const(i64/bool), BinOp(Add/Sub/Mul/Div/Mod), Compare(Eq/Ne/Lt/Le/Gt/Ge), Return
- Paramのi64経路（複数引数対応）
- Array/Mapの最小HostCall（len/get/set/push/size）
- Branch/JumpはPhase 10.7でCranelift配線を導入（feature: `cranelift-jit`）。
  - 分岐条件はb1化（i64の場合は !=0 で正規化）
  - 直線＋if/elseでのreturnをJITで実行（副作用は未対応のためVMへ）
  - PHIは将来の`NYASH_JIT_PHI_MIN=1`で最小導入予定
