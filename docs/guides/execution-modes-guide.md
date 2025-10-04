# Hakorune 実行モード完全ガイド

**最終更新**: 2025-10-04
**対象**: 開発者・ユーザー

---

## 🎯 3秒で理解

Hakoruneには**4つの実行モード**があります：

| モード | 用途 | コマンド例 | 出力責務 |
|--------|------|-----------|---------|
| **VM** | 開発・デバッグ | `./hakorune program.hkr` | FallbackVmEngine |
| **LLVM CLI** | 本番・最適化 | `./hakorune --backend llvm program.hkr` | Runner (src/runner/modes/llvm.rs) |
| **LLVM AOT** | スタンドアロンEXE | `clang -o app.exe program.o nyrt_stub.c` | nyrt スタブ main |
| **WASM** | Web実行 | `node wasm_runner.js program.wasm` | Node ランナー |

---

## 📊 詳細比較表

### 1. VM（Rust VM ライン）⭐ 開発・デバッグ用

**特徴**:
- Rust実装、型安全、高速
- デバッグ情報豊富
- `--dump-mir` など診断機能充実

**実行方法**:
```bash
# 基本実行
./target/release/hakorune program.hkr
./target/release/hakorune --backend vm program.hkr  # 明示的

# デバッグ実行
HAKO_CLI_VERBOSE=1 ./target/release/hakorune program.hkr
./target/release/hakorune --dump-mir program.hkr
```

**出力責務**:
- ✅ **FallbackVmEngine** (src/backend/vm/fallback_vm_engine.rs)
- 最深部で出力するため取りこぼしなし
- stdout + stderr ミラーで確実表示
- flush徹底でバッファ問題回避

**Result表示形式**:
```
📊 Result: 42
```

**コード箇所**:
```rust
// src/backend/vm/fallback_vm_engine.rs
eprintln!("📊 Result: {}", return_value);  // stderr ミラー
println!("📊 Result: {}", return_value);   // stdout
io::stdout().flush().unwrap();             // flush徹底
```

---

### 2. LLVM CLI（llvmlite ハーネス）⚡ 本番・最適化用

**特徴**:
- Python/llvmlite実装（実証済み安定性）
- LLVM IR最適化
- プラグインBox完全サポート

**実行方法**:
```bash
# 基本実行（環境変数必須）
HAKO_LLVM_USE_HARNESS=1 ./target/release/hakorune --backend llvm program.hkr

# プラグインテスト
echo 'local c = new CounterBox(); c.inc(); print(c.get())' > test.hkr
HAKO_LLVM_USE_HARNESS=1 ./target/release/hakorune --backend llvm test.hkr
```

**出力責務**:
- ✅ **Runner (src/runner/modes/llvm.rs)**
- LLVMハーネス実行後、戻り値を取得して出力
- "📊 Result: ..." を担当

**Result表示形式**:
```
📊 Result: 42
```

**コード箇所**:
```rust
// src/runner/modes/llvm.rs
println!("📊 Result: {}", exit_code);
```

**注意事項**:
- `HAKO_LLVM_USE_HARNESS=1` 環境変数必須
- llvmlite インストール必要 (`pip install llvmlite`)

---

### 3. LLVM AOT（スタンドアロン EXE）🚀 配布用

**特徴**:
- 単独実行可能なバイナリ生成
- Runnerから独立
- 配布・デプロイに最適

**生成方法**:
```bash
# Step 1: MIR JSON生成
./target/release/hakorune --emit-mir-json program.json program.hkr

# Step 2: LLVM IR → .o 生成（Python LLVM backend）
cd src/llvm_py
./venv/bin/python llvm_builder.py ../../program.json -o ../../program.o

# Step 3: スタブmainとリンク
cd ../..
clang -o program.exe program.o crates/hakorune_kernel/nyrt_stub_main.c \
      -L./target/release -lhakorune_kernel
```

**実行方法**:
```bash
# 単独実行
./program.exe

# 静音実行（必要に応じて）
NYASH_NYRT_SILENT_RESULT=1 ./program.exe
```

**出力責務**:
- ✅ **nyrt スタブ main** (crates/hakorune_kernel/nyrt_stub_main.c)
- `ny_main()` を呼び出し、戻り値を処理
- 1. "Result: <n>" を stdout へ出力（flush）
- 2. stderr にもミラー（2>&1 パイプ対応）
- 3. `exit(n & 0xFF)`

**Result表示形式**:
```
Result: 42
```

**コード箇所**:
```c
// crates/hakorune_kernel/nyrt_stub_main.c (想定)
int main(void) {
    int result = ny_main();

    // 環境変数で静音化可能
    if (!getenv("NYASH_NYRT_SILENT_RESULT")) {
        fprintf(stdout, "Result: %d\n", result);
        fflush(stdout);
        fprintf(stderr, "Result: %d\n", result);  // ミラー
    }

    exit(result & 0xFF);
}
```

**メリット**:
- Runner不要、単独で動作
- 配布が容易
- 起動オーバーヘッド最小

**デメリット**:
- リンク手順が複雑
- デバッグ情報が少ない

---

### 4. WASM（Web実行）🌐 実験的

**特徴**:
- WebAssembly出力
- ブラウザ/Node.js実行
- 実験的実装

**生成・実行方法**:
```bash
# Step 1: MIR JSON生成
./target/release/hakorune --emit-mir-json program.json program.hkr

# Step 2: WASM生成（Python LLVM backend）
cd src/llvm_py
./venv/bin/python llvm_builder.py --target wasm32 ../../program.json -o ../../program.wasm

# Step 3: エクスポート追加（必要に応じて）
./venv/bin/python tools/wasm_add_export.py ../../program.wasm ../../fixed.wasm "Main.main" 0

# Step 4: Node.js実行
cd ../..
node src/llvm_py/tools/wasm_runner.js fixed.wasm
```

**出力責務**:
- ✅ **Node ランナー** (src/llvm_py/tools/wasm_runner.js)
- WASMモジュール実行後、戻り値を取得
- "returned: ..." を出力

**Result表示形式**:
```
returned: 42
```

**コード箇所**:
```javascript
// src/llvm_py/tools/wasm_runner.js
console.log("returned:", result);
```

**注意事項**:
- branch命令の変換に不具合あり（条件分岐が常にfalse）
- fibonacci等の複雑な制御フロー未対応
- 開発中・実験的機能

---

## 🎯 モード選択ガイド

### 開発・デバッグ時 → **VM**
```bash
./target/release/hakorune program.hkr
./target/release/hakorune --dump-mir program.hkr  # MIR確認
```
**理由**:
- デバッグ情報豊富
- 高速ビルド・実行
- エラーメッセージ詳細

---

### 本番・プラグインテスト → **LLVM CLI**
```bash
HAKO_LLVM_USE_HARNESS=1 ./target/release/hakorune --backend llvm program.hkr
```
**理由**:
- LLVM最適化
- プラグインBox完全対応
- 実証済み安定性

---

### 配布・デプロイ → **LLVM AOT**
```bash
# 生成
clang -o app.exe program.o nyrt_stub_main.c -L./target/release -lhakorune_kernel

# 配布
./app.exe  # 単独実行可能
```
**理由**:
- Runner不要
- 単一バイナリ
- 起動高速

---

### Web実行 → **WASM**（実験的）
```bash
node wasm_runner.js program.wasm
```
**理由**:
- ブラウザ実行可能
- プラットフォーム非依存

---

## 🔍 トラブルシューティング

### 問題: print() が stdout に出力されない

**モード別診断**:

#### VM
```bash
# 解決策1: stderr確認
./target/release/hakorune program.hkr 2>&1

# 解決策2: FallbackVmEngine修正確認
# src/backend/vm/fallback_vm_engine.rs にflush処理があるか確認
```

#### LLVM CLI
```bash
# 解決策1: Runner確認
# src/runner/modes/llvm.rs の "📊 Result: ..." 出力確認

# 解決策2: 環境変数確認
HAKO_LLVM_USE_HARNESS=1 ./target/release/hakorune --backend llvm program.hkr
```

#### LLVM AOT
```bash
# 解決策1: スタブmain実装確認
# crates/hakorune_kernel/nyrt_stub_main.c のfprintf/fflush確認

# 解決策2: 手動実行でstdout確認
./program.exe | cat -A  # 制御文字表示
```

#### WASM
```bash
# 解決策1: Node ランナー確認
# src/llvm_py/tools/wasm_runner.js のconsole.log確認

# 解決策2: 直接WASM実行
node -e "const fs=require('fs'); const buf=fs.readFileSync('program.wasm'); ..."
```

---

### 問題: "📊 Result: ..." が表示されない

**チェックリスト**:
1. ✅ モードに応じた出力責務確認（上記表参照）
2. ✅ stdout/stderr リダイレクト確認（`2>&1`）
3. ✅ バッファリング問題（flush呼び出し確認）
4. ✅ 環境変数設定（LLVM CLIは`HAKO_LLVM_USE_HARNESS=1`必須）

---

## 📋 環境変数一覧

### 共通
```bash
HAKO_CLI_VERBOSE=1           # 詳細診断
HAKO_DISABLE_PLUGINS=1       # プラグイン無効化
```

### VM専用
```bash
HAKO_VM_DUMP_MIR=1          # MIR出力
HAKO_DEBUG_MIR_PRINTER=1    # MIRプリンターデバッグ
```

### LLVM CLI専用
```bash
HAKO_LLVM_USE_HARNESS=1     # llvmliteハーネス使用（必須）
HAKO_LLVM_OBJ_OUT=/tmp/out.o  # .o出力先指定
```

### LLVM AOT専用
```bash
NYASH_NYRT_SILENT_RESULT=1  # Result出力抑制
```

### WASM専用
```bash
# (現在なし)
```

---

## 🚀 クイックスタート

### 初めての場合
```bash
# Step 1: ビルド
cargo build --release

# Step 2: Hello World実行（VM）
echo 'print("Hello, Hakorune!")' > hello.hkr
./target/release/hakorune hello.hkr

# Step 3: LLVM CLI試す
HAKO_LLVM_USE_HARNESS=1 ./target/release/hakorune --backend llvm hello.hkr
```

### 既存ユーザー
```bash
# 普段はVM
./target/release/hakorune program.hkr

# 本番テストはLLVM CLI
HAKO_LLVM_USE_HARNESS=1 ./target/release/hakorune --backend llvm program.hkr

# 配布時はAOT
# (ビルド手順は上記参照)
```

---

## 📊 モード別コード責務マトリックス

| 責務 | VM | LLVM CLI | LLVM AOT | WASM |
|------|----|-----------|-----------|----|
| プログラム実行 | FallbackVmEngine | LLVMハーネス | スタブmain | WASMランタイム |
| Result出力 | FallbackVmEngine | Runner (llvm.rs) | スタブmain | Node ランナー |
| エラーハンドリング | Runner | Runner | スタブmain | Node ランナー |
| MIR生成 | MIR Builder | MIR Builder | MIR Builder | MIR Builder |
| 最適化 | なし | LLVM | LLVM | LLVM |

---

## 🔗 関連ドキュメント

- **[技術詳解: 関数解決の仕組み](execution-modes-technical-deep-dive.md)** ⭐内部実装理解
  - LLVM CLIがどのように関数を解決するか
  - libhakorune_kernel.aの役割
  - 各モードの関数解決マトリックス
  - デバッグ方法・新機能追加ガイド
- **[CLAUDE.md](../../CLAUDE.md)** - 開発者入口
- **[README.md](../../README.md)** - プロジェクト概要
- **[スモークテストガイド](../../tools/smokes/README.md)** - テスト実行方法
- **[Phase 15 ROADMAP](../development/roadmap/phases/phase-15/ROADMAP.md)** - 現在の進捗

---

## 💡 まとめ

### 覚えておくべき3つのこと

1. **開発はVM、本番はLLVM CLI、配布はAOT**
   ```bash
   # 開発
   ./target/release/hakorune program.hkr

   # 本番
   HAKO_LLVM_USE_HARNESS=1 ./target/release/hakorune --backend llvm program.hkr

   # 配布
   ./program.exe  # 単独実行
   ```

2. **出力責務は「葉」が持つ**
   - VM: FallbackVmEngine
   - LLVM CLI: Runner (llvm.rs)
   - LLVM AOT: スタブmain
   - WASM: Node ランナー

3. **print()問題は各モードで解決箇所が異なる**
   - VM: FallbackVmEngine修正
   - LLVM CLI: Runner修正
   - LLVM AOT: スタブmain修正
   - WASM: Node ランナー修正

---

**作成日**: 2025-10-04
**作成者**: Claude Code (Sonnet 4.5)
**バージョン**: 1.0
