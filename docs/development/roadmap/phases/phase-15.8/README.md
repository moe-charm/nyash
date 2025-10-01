# Phase 15.8: LLVM→WASM実装（WebAssembly出力）

**期間**: 2025-10-01 ~ 2025-10-22 (3週間)
**前提**: Phase 15.7完了（MIR18命令凍結）
**目標**: MIR18命令をWASMに変換し、ブラウザ/エッジ環境で実行可能にする

---

## 🎯 **目的と範囲**

### **主目的**
1. **LLVM→WASM変換パイプライン構築**
   - llvmlite経由でLLVM IR生成
   - LLVM toolchain（llc, wasm-ld）でWASM変換
   - クリーンなMIR18命令セットでWASM実装

2. **ブラウザ/エッジ環境対応**
   - WASI（WebAssembly System Interface）基本対応
   - Node.js/Deno runtime対応
   - 基本I/O（print, error）動作確認

3. **Parity確認**
   - VM/LLVM/WASMで同一出力
   - スモークテスト全グリーン維持

### **非目標（Phase 16以降に延期）**
- ❌ WASM GC統合（WASM GC proposal待ち）
- ❌ 最適化・バンドルサイズ削減（Phase 17）
- ❌ フル非同期runtime（Promise完全統合はPhase 17）
- ❌ DOM API連携（Phase 18）

---

## 📊 **技術スタック**

### **ビルドチェーン**
```
MIR(JSON v0)
  ↓ (llvmlite Python)
LLVM IR (.ll)
  ↓ (llc --march=wasm32)
WASM Object (.o)
  ↓ (wasm-ld)
WASM Binary (.wasm)
  ↓ (Node.js/Deno/Browser)
実行
```

### **依存関係**
- **必須**:
  - Python 3.8+ + llvmlite
  - LLVM toolchain 14+ (llc, wasm-ld)
  - Node.js 18+ or Deno 1.30+（テスト用）
- **オプション**:
  - wabt tools（wasm2wat, wat2wasm - デバッグ用）
  - wasm-opt（最適化 - Phase 17）

---

## 🗓️ **タイムラインと成果物**

### **Week 1: LLVM→WASM変換パイプライン** (2025-10-01 ~ 10-07)
#### 目標
- llvmlite → LLVM IR → WASM 基本変換確立
- Hello World (.wasm) 生成成功

#### タスク
1. **llvmlite WASM target対応**
   - `target = "wasm32-unknown-wasi"`設定
   - WASM calling convention調整
2. **ビルドスクリプト作成**
   - `tools/build_wasm.sh`: MIR JSON → WASM一括変換
3. **Hello World動作確認**
   - `print("Hello, WASM!")` → WASM実行成功

#### 成果物 ✅ **完了** (2025-10-01)
- ✅ `tools/build_wasm.sh` - WASMビルドスクリプト
- ✅ `tools/wasm_runner.js` - Node.js実行確認スクリプト
- ✅ `tools/test_wasm_init.sh` - 統合テストスクリプト（Phase 1.1+1.2+1.3）
- ✅ `src/llvm_py/llvm_builder.py` - WASM target対応（wasm32-unknown-wasi）
- 📝 `apps/tests/hello_wasm.hkr` - 保留（Week 2で関数エクスポート解決後）

**Week 1完了報告**:
- 🎉 llvmliteだけでWASMバイナリ生成成功（LLC/wasm-ld不要）
- ✅ Native/WASMコンパイル両対応（--target native/wasm32）
- ✅ WASM triple検証PASS（wasm32-unknown-wasi）
- ⚠️ 関数エクスポート制限（LLVM toolchain or wabt後処理で解決予定）

---

### **Week 2: MIR18命令WASM変換完成** (2025-10-08 ~ 10-14)
#### 目標
- MIR18命令すべてWASM変換対応
- Const, BinOp, Branch, Jump, Return, Phi動作確認

#### タスク
1. **基本命令対応** (5命令)
   - Const, UnaryOp, BinOp, Compare, TypeOp
2. **メモリ命令対応** (2命令)
   - Load, Store (WASM linear memory)
3. **制御命令対応** (4命令)
   - Branch, Jump, Return, Phi (WASM block/br)
4. **Box/GC命令対応** (4命令)
   - MirCall, Safepoint, Barrier, Copy/Nop
5. **外部連携** (2命令)
   - ExternCall → WASI import
   - Print → fd_write

#### 成果物
- ✅ `src/llvm_py/wasm_emitter.py`: WASM専用emitter
- ✅ スモークテスト5本（基本演算/制御/Box/外部）
- ✅ WASM命令マッピング表（docs）

---

### **Week 3: WASI runtime連携 + Parity確認** (2025-10-15 ~ 10-22)
#### 目標
- WASI基本I/O動作
- VM/LLVM/WASMパリティ確認

#### タスク
1. **WASI runtime連携**
   - fd_write (print)
   - fd_read (基本入力 - optional)
   - clock_time_get (now関数)
2. **Parity テスト**
   - 既存quickスモーク → WASM変換
   - VM/LLVM/WASM同一出力確認
3. **ドキュメント整備**
   - WASM実行ガイド
   - トラブルシューティング

#### 成果物
- ✅ `tools/smokes/v2/profiles/wasm/`: WASM専用スモークプロファイル
- ✅ `docs/guides/wasm-execution.md`: WASM実行ガイド
- ✅ Parity確認レポート（VM/LLVM/WASM）

---

## 🎨 **設計方針**

### **MIR18命令 → WASM変換戦略**

#### **基本演算 (5命令)**
```
Const(Int)    → i32.const
Const(Float)  → f64.const
BinOp(Add)    → i32.add / f64.add
Compare(Lt)   → i32.lt_s / f64.lt
TypeOp(Cast)  → i32.wrap_i64 / f64.promote_f32
```

#### **制御フロー (4命令)**
```
Branch(cond, then, else)
  → (block $else
      (block $then
        (br_if $then (local.get $cond))
        (br $else)
      )
      ... then_block ...
      (br $merge)
    )
    ... else_block ...
    (block $merge)

Jump(target)  → br $target
Return(val)   → (local.get $val) (return)
Phi(inputs)   → WASM locals + edge-copy materialization
```

#### **Box/外部 (6命令)**
```
MirCall
  → Method: call_indirect $table_index
  → Global: call $builtin_function
  → Extern: import "wasi_snapshot_preview1" "fd_write"

Safepoint     → (call $gc_safepoint)  // WASM GC proposal待ち
Barrier       → nop  // 将来WASM GC対応
```

### **Memory Model**
```
WASM Linear Memory:
  [0..1MB]       : Stack (local variables)
  [1MB..16MB]    : Heap (Box instances)
  [16MB..]       : Reserved

GC Strategy (Phase 15.8):
  - Manual reference counting（WASM GC proposal未対応）
  - Phase 16でWASM GC proposal採用検討
```

---

## 🚀 **実行方法**

### **ビルド**
```bash
# WASM生成
./tools/build_wasm.sh apps/tests/hello.hkr -o hello.wasm

# 詳細: MIR JSON経由
./target/release/hakorune --emit-mir-json hello.json apps/tests/hello.hkr
python3 src/llvm_py/wasm_emitter.py hello.json -o hello.wasm
```

### **実行**
```bash
# Node.js
node --experimental-wasi-unstable-preview1 tools/wasm_runner.js hello.wasm

# Deno
deno run --allow-read tools/wasm_runner.ts hello.wasm

# Browser（開発サーバー必要）
python3 -m http.server 8000
# → http://localhost:8000/wasm_demo.html
```

---

## 📋 **成功条件**

### **P0（必須）**
1. ✅ Hello World WASM実行成功
2. ✅ MIR18命令すべてWASM変換対応
3. ✅ 基本演算/制御フローのパリティ確認（VM/LLVM/WASM）
4. ✅ print関数（fd_write）動作

### **P1（推奨）**
1. ✅ 既存quickスモーク10本WASM変換成功
2. ✅ WASI clock_time_get対応（now関数）
3. ✅ エラー処理（panic, error）動作

### **P2（オプション）**
1. ⏸️ async/await基本対応（Promise連携 - Phase 17推奨）
2. ⏸️ バンドルサイズ<100KB（最適化 - Phase 17推奨）
3. ⏸️ WASM GC proposal検証（Phase 16推奨）

---

## 🔧 **開発環境セットアップ**

### **Linux/WSL**
```bash
# LLVM toolchain
sudo apt install llvm-14 lld-14

# llvmlite
pip install llvmlite

# Node.js (WASI対応版)
nvm install 18
nvm use 18

# wabt (optional)
sudo apt install wabt
```

### **macOS**
```bash
# LLVM
brew install llvm

# llvmlite
pip3 install llvmlite

# Node.js
brew install node

# wabt
brew install wabt
```

---

## 📚 **参考資料**

### **WASM仕様**
- [WebAssembly Core Specification](https://webassembly.github.io/spec/core/)
- [WASI (WebAssembly System Interface)](https://github.com/WebAssembly/WASI)
- [WASM GC Proposal](https://github.com/WebAssembly/gc)

### **LLVM→WASM**
- [LLVM WebAssembly Backend](https://llvm.org/docs/WebAssembly.html)
- [Emscripten Documentation](https://emscripten.org/)
- [llvmlite WASM target](https://llvmlite.readthedocs.io/)

### **参考実装**
- AssemblyScript (TypeScript → WASM)
- Rust wasm32-wasi target
- Go WASM target

---

## 🚨 **既知の課題と制約**

### **Phase 15.8制約**
1. **WASM GC未対応**
   - 手動reference counting使用
   - メモリリーク可能性あり（短時間実行のみ推奨）

2. **async/await制限**
   - Promise連携は基本のみ
   - フル非同期はPhase 17

3. **パフォーマンス**
   - 最適化なし（デバッグビルド相当）
   - Phase 17で最適化パス追加

### **回避策**
- 長時間実行: LLVM native推奨
- GC重要: Rust VM推奨
- 本番環境: Phase 16以降推奨

---

## 📝 **関連ドキュメント**
- [MIR18命令セット](../../mir/INSTRUCTION_SET.md)
- [LLVM Backend実装](../../../guides/llvm-backend.md)
- [スモークテスト v2](../../../../tools/smokes/v2/README.md)
- [Phase 15.7完了報告](../phase-15.7/README.md)
