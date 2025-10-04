# 実行モード技術詳解 - 関数解決の仕組み

**最終更新**: 2025-10-04
**対象**: 開発者（内部実装理解）

---

## 🎯 質問: LLVM CLIは Hakoruneの実行ファイルで関数を解決？

**答え**: **YES！**

LLVM CLI（llvmliteハーネス）モードでも、最終的には**libhakorune_kernel.a**（Hakoruneカーネル静的ライブラリ）とリンクして、すべての外部関数（ExternCall, BoxCall等）を解決します。

---

## 📊 各モードの関数解決の仕組み

### 1. VM（Rust VM ライン）

**解決方法**: Rust実装の直接呼び出し

```
プログラム実行
    ↓
FallbackVmEngine (Rust VM)
    ↓
ExternCall命令 → src/backend/mir_interpreter/handlers/externals.rs
BoxCall命令 → Box trait実装（Rust側）
    ↓
結果返却
```

**特徴**:
- Rust側で完結
- 外部ライブラリ不要
- 型安全・高速

**コード箇所**:
- `src/backend/vm/fallback_vm_engine.rs` - VM本体
- `src/backend/mir_interpreter/handlers/externals.rs` - ExternCall処理
- `src/runtime/boxes/` - 各種Box実装

---

### 2. LLVM CLI（llvmlite ハーネス）⚡ 詳解

**解決方法**: Python/llvmlite でLLVM IR生成 → .o生成 → libhakorune_kernel.aとリンク

#### ステップ1: LLVM IR生成（Python側）

```
MIR JSON
    ↓
Python/llvmlite ハーネス (tools/llvmlite_harness.py)
    ↓
LLVM IR生成
  - 外部関数は declare するだけ（実装なし）
  - 例: declare i64 @nyash.console.log(i8*)
    ↓
.o ファイル生成
```

**重要**: この時点では外部関数の**実装は含まれない**（宣言のみ）

**コード箇所**:
- `src/llvm_py/instructions/externcall.py` - ExternCall LLVM IR生成
  ```python
  # 外部関数を declare（実装なし）
  func = ir.Function(module, fnty, name="nyash.console.log")
  # LLVM IRのcall命令発行
  result = builder.call(func, call_args)
  ```
- `src/llvm_py/instructions/boxcall.py` - BoxCall LLVM IR生成
  ```python
  # 箱メソッドを外部関数として declare
  callee = _declare(module, "nyash.any.length_h", i64, [i64])
  result = builder.call(callee, [recv_h])
  ```

#### ステップ2: リンク（ny-llvmc）

```
.o ファイル（生成済み）
    ↓
ny-llvmc (crates/nyash-llvm-compiler)
    ↓
リンカー（clang/gcc）呼び出し:
  clang -o exe program.o \
        -Wl,--whole-archive \
        libhakorune_kernel.a \  ← ここで解決！
        -Wl,--no-whole-archive \
        -ldl -lpthread -lm
    ↓
実行可能ファイル（すべての関数が解決済み）
```

**重要**: `libhakorune_kernel.a`に、すべての外部関数の実装が含まれている！

**コード箇所**:
- `crates/nyash-llvm-compiler/src/main.rs` - `link_executable`関数:
  ```rust
  fn link_executable(obj: &Path, out_exe: &Path, ...) -> Result<()> {
      // libhakorune_kernel.a を探す
      let libnyrt = nyrt_dir.join("libhakorune_kernel.a");

      // リンカー呼び出し
      let mut cmd = Command::new(linker);
      cmd.arg("-o").arg(out_exe);
      cmd.arg(obj);
      // --whole-archive で完全リンク
      cmd.arg("-Wl,--whole-archive")
         .arg(libnyrt)  // ← ここで libhakorune_kernel.a とリンク！
         .arg("-Wl,--no-whole-archive");
      cmd.arg("-ldl").arg("-lpthread").arg("-lm");
      // ...
  }
  ```

#### ステップ3: 実行

```
実行可能ファイル
    ↓
./tmp/nyash_llvm_run
    ↓
ny_main() 呼び出し
    ↓
nyash.console.log 等の関数呼び出し
    ↓
libhakorune_kernel.a の実装が実行される！
```

**実行フロー**:
1. `src/runner/modes/llvm.rs` - Runner側でny-llvmcを呼び出し
2. `crates/nyash-llvm-compiler` - .o生成 + libhakorune_kernel.aとリンク
3. 生成されたEXEを実行
4. `src/runner/modes/common_util/exec.rs` - `run_executable`で実行
5. Runner側で "📊 Result: ..." 出力

**まとめ**: LLVM CLIモードは、**Hakoruneの実行ファイル（正確にはlibhakorune_kernel.a）を使って関数解決している！**

---

### 3. LLVM AOT（スタンドアロンEXE）

**解決方法**: 同様に libhakorune_kernel.aとリンク（LLVM CLIと同じ）

```
MIR JSON
    ↓
Python/llvmlite ハーネス
    ↓
.o ファイル
    ↓
clang リンク:
  clang -o program.exe program.o \
        libhakorune_kernel.a \
        nyrt_stub_main.c
    ↓
スタンドアロンEXE（配布可能）
```

**違い**:
- LLVM CLI: 毎回EXE生成 → 即座に実行 → 破棄
- LLVM AOT: EXEを保存 → 配布・デプロイ

**コード箇所**:
- リンク処理は LLVM CLIと共通（`ny-llvmc`）
- スタブmain: `crates/hakorune_kernel/nyrt_stub_main.c` (想定)

---

### 4. WASM

**解決方法**: WASI runtime経由（実験的）

```
MIR JSON
    ↓
Python/llvmlite ハーネス (--target wasm32)
    ↓
WASM生成
    ↓
Node.js + WASI
    ↓
外部関数をJavaScript側でエミュレート
```

**注意**: WASMモードは現在実験的で、branch命令の変換に不具合あり。

**コード箇所**:
- `src/llvm_py/tools/wasm_runner.js` - Node.jsランナー
- `src/llvm_py/llvm_builder.py` - `--target wasm32` 対応

---

## 🔍 libhakorune_kernel.a の役割

### 含まれる実装

**ExternCall系**:
- `nyash.console.log` - 標準出力
- `nyash.string.len_h` - 文字列長取得
- `nyash.string.concat_hh` - 文字列結合
- `nyash.string.eq_hh` - 文字列比較
- ... など多数

**BoxCall系**:
- `nyash.any.length_h` - Any.length（Array/String/Map共通）
- `nyash.box.from_i8_string` - i8*→ハンドル変換
- ... など

**Runtime系**:
- `ny_main` - エントリーポイント
- GC機能
- メモリ管理
- ... など

### ビルド方法

```bash
# libhakorune_kernel.a のビルド
cargo build --release -p hakorune-kernel

# 生成場所
target/release/libhakorune_kernel.a
```

### リンク方法

```bash
# --whole-archive で完全リンク（推奨）
clang -o program.exe program.o \
      -Wl,--whole-archive \
      target/release/libhakorune_kernel.a \
      -Wl,--no-whole-archive \
      -ldl -lpthread -lm
```

**`--whole-archive`の意味**:
- 通常: リンカーは使用されるシンボルのみをアーカイブから取り出す
- `--whole-archive`: アーカイブのすべてのオブジェクトをリンク
- 必要な理由: 動的に呼ばれる関数（method_id経由等）も確実にリンク

---

## 📊 モード別関数解決マトリックス

| モード | LLVM IR生成 | 関数解決方法 | 使用ライブラリ |
|--------|------------|-------------|---------------|
| **VM** | なし | Rust直接呼び出し | なし |
| **LLVM CLI** | Python/llvmlite | リンク時（clang） | libhakorune_kernel.a |
| **LLVM AOT** | Python/llvmlite | リンク時（clang） | libhakorune_kernel.a |
| **WASM** | Python/llvmlite | WASI runtime | JavaScript エミュレート |

---

## 🔧 デバッグ方法

### LLVM CLI の関数解決確認

```bash
# Step 1: .o ファイル生成
NYASH_LLVM_OBJ_OUT=/tmp/test.o ./target/release/hakorune --backend llvm program.hkr

# Step 2: シンボル確認
nm /tmp/test.o | grep -E 'nyash\.|ny_main'
# 出力例:
#                  U nyash.console.log     ← 未定義（extern）
#                  U nyash.string.len_h    ← 未定義（extern）
# 0000000000000000 T ny_main                ← 定義済み

# Step 3: libhakorune_kernel.a のシンボル確認
nm target/release/libhakorune_kernel.a | grep 'nyash.console.log'
# 出力例:
# 0000000000000000 T nyash.console.log      ← 定義済み！

# Step 4: リンク後のシンボル確認
nm tmp/nyash_llvm_run | grep 'nyash.console.log'
# 出力例:
# 000000000040abcd T nyash.console.log      ← 解決済み（アドレス割り当て済み）
```

### 詳細診断

```bash
# リンクプロセスを可視化
NYASH_CLI_VERBOSE=1 NYASH_LLVM_USE_HARNESS=1 ./target/release/hakorune --backend llvm program.hkr

# Python ハーネスの出力確認
python3 tools/llvmlite_harness.py --in test.json --out /tmp/test.o --verbose

# リンカーコマンド確認
ny-llvmc --in test.json --emit exe --out /tmp/test.exe --nyrt target/release
# → 内部で実行されるclangコマンドを確認
```

---

## 💡 よくある誤解

### ❌ 誤解1: LLVM CLIはPython側で関数を実装している

**真実**: Python側は LLVM IR生成のみ。関数の実体は `libhakorune_kernel.a` に含まれる。

### ❌ 誤解2: llvmliteがJIT実行している

**真実**: llvmliteは`.o`生成のみ。実行は clangでリンクしたネイティブEXEを実行。

### ❌ 誤解3: VMとLLVM CLIで異なる実装を呼んでいる

**真実**: 両方とも同じ`libhakorune_kernel.a`の実装を使用（VMはRust側で静的リンク、LLVM CLIは動的リンク）。

---

## 🚀 実装者向けガイド

### 新しい外部関数を追加する方法

#### Step 1: Rust側実装（libhakorune_kernel.a）

```rust
// crates/hakorune-kernel/src/console.rs

#[no_mangle]
pub extern "C" fn nyash_console_error(msg: *const i8) -> i64 {
    unsafe {
        let c_str = std::ffi::CStr::from_ptr(msg);
        eprintln!("{}", c_str.to_string_lossy());
    }
    0
}
```

#### Step 2: Python側宣言（LLVM IR生成）

```python
# src/llvm_py/instructions/externcall.py

sig_map = {
    # 既存...
    "nyash.console.error": (i64, [i8p]),  # 追加！
}
```

#### Step 3: MIR側対応（必要に応じて）

```rust
// src/mir/builder/ops.rs
// ExternCall命令生成時に "nyash.console.error" を指定
```

#### Step 4: ビルド・テスト

```bash
# カーネル再ビルド
cargo build --release -p hakorune-kernel

# テスト
echo 'externcall("nyash.console.error", "Test!")' > test.hkr
NYASH_LLVM_USE_HARNESS=1 ./target/release/hakorune --backend llvm test.hkr
```

---

## 🔗 関連ドキュメント

- **[実行モード完全ガイド](execution-modes-guide.md)** - モード選択・トラブルシューティング
- **[CLAUDE.md](../../CLAUDE.md)** - 開発者入口
- **[MIR命令セット](../reference/mir/INSTRUCTION_SET.md)** - MIR仕様

---

## 📝 まとめ

### 重要ポイント

1. **LLVM CLIモードでも、libhakorune_kernel.aを使って関数解決している**
   - Python/llvmliteはLLVM IR生成と.o生成のみ
   - 実際の関数実装はlibhakorune_kernel.aに含まれる
   - ny-llvmcがリンク時にlibhakorune_kernel.aと結合

2. **すべてのモードが同じカーネル実装を共有**
   - VM: Rust静的リンク
   - LLVM CLI/AOT: libhakorune_kernel.a動的リンク
   - WASM: JavaScript エミュレート（実験的）

3. **外部関数の追加は3ステップ**
   - Rust側実装（libhakorune_kernel.a）
   - Python側宣言（LLVM IR生成）
   - ビルド・テスト

---

**作成日**: 2025-10-04
**作成者**: Claude Code (Sonnet 4.5)
**バージョン**: 1.0
