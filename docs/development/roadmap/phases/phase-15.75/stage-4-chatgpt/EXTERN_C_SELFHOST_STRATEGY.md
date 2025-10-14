# `extern_c` によるセルフホスト完全戦略

## 🎯 **核心的発見: Rustと同じことをする**

Rustが外部C関数を呼べるように、Hakoruneも外部C関数を呼べるようにする：

```rust
// Rust
extern "C" {
    fn llvm_compile_mir(mir: *const c_char, output: *const c_char) -> i32;
}

let result = unsafe { llvm_compile_mir(mir_ptr, output_ptr) };
```

```hakorune
// Hakorune（提案）
static box Compiler {
    compile(mir: StringBox, output: StringBox) -> IntegerBox {
        local result = extern_c "llvm_compile_mir" (
            mir.to_cstring(),
            output.to_cstring()
        )
        return result
    }
}
```

**これで何が変わるか**: Rust層が「スクリプト実行エンジン」のみに縮退！

---

## 📊 **3つのフェーズ**

### **Phase 1: Rust hakorune.exe (ブートストラップVM)**

```
┌─────────────────────────────────────────────┐
│ Rust hakorune.exe                           │
├─────────────────────────────────────────────┤
│ - 最小VM (extern_c サポート追加)          │
│ - Parser/MIR Builder (既存Rust実装)        │
│ - VM Interpreter (既存Rust実装)            │
└─────────────────────────────────────────────┘
          ↓ リンク（静的 or 動的）
┌─────────────────────────────────────────────┐
│ ネイティブライブラリ (C/Rust製)            │
├─────────────────────────────────────────────┤
│ - libhako_boxes.so (StringBox/ArrayBox/Map) │
│ - libllvm_backend.so (LLVM wrapper)         │
│ - libhako_parser.so (optional)              │
│ - libhako_mir.so (optional)                 │
└─────────────────────────────────────────────┘
```

**重要**: 静的リンク(.o)も動的リンク(.so)も両方サポート可能

---

### **Phase 2: Hakorune製コンパイラ（.hakoスクリプト）**

```hakorune
// apps/selfhost/compiler.hako
static box Compiler {
    compile_to_object(source_path: StringBox, output_path: StringBox) -> IntegerBox {
        // Step 1: Parse (C関数呼び出し)
        local ast_json = extern_c "hako_parse_to_json" (source_path.to_cstring())

        // Step 2: Build MIR (C関数呼び出し)
        local mir_json = extern_c "hako_build_mir_json" (ast_json)

        // Step 3: Compile to .o (C関数呼び出し)
        local result = extern_c "llvm_compile_mir_to_object" (
            mir_json,
            output_path.to_cstring()
        )

        return result
    }
}

static box Main {
    main() {
        local compiler = new Compiler()

        // すべてのコンポーネントをコンパイル
        compiler.compile_to_object("apps/selfhost/parser.hako", "build/parser.o")
        compiler.compile_to_object("apps/selfhost/mir_builder.hako", "build/mir_builder.o")
        compiler.compile_to_object("apps/selfhost/vm.hako", "build/vm.o")
        compiler.compile_to_object("apps/selfhost/main.hako", "build/main.o")

        // Link
        extern_c "system" ("clang build/*.o -o hakorune-selfhost.exe".to_cstring())

        return "✅ Self-host complete"
    }
}
```

**実行方法**:
```bash
# Rust VM で Hakorune製コンパイラを実行
./target/release/hako apps/selfhost/compiler.hako

# 出力:
# build/parser.o
# build/mir_builder.o
# build/vm.o
# build/main.o
# hakorune-selfhost.exe ← 完全ネイティブ！
```

---

### **Phase 3: hakorune-selfhost.exe (完全ネイティブ)**

```bash
# Rust VM 不要！すべてネイティブコード
./hakorune-selfhost.exe program.hako
```

```
┌─────────────────────────────────────────────┐
│ hakorune-selfhost.exe                       │
├─────────────────────────────────────────────┤
│ - Parser (Hakorune → .o → ネイティブ)      │
│ - MIR Builder (Hakorune → .o → ネイティブ) │
│ - VM (Hakorune → .o → ネイティブ)          │
│ - Main (Hakorune → .o → ネイティブ)        │
├─────────────────────────────────────────────┤
│ + StringBox/ArrayBox/MapBox (Hako ABI .o)   │
│ + LLVM Backend (C/Rust .o)                  │
│ + 最小ランタイム (libnyrt.a)                │
└─────────────────────────────────────────────┘
```

**重要**: すべてAOTコンパイル済み、Rust VMなし！

---

## 🔧 **`extern_c` 実装詳細**

### **1. 構文定義**

```bnf
extern_c_call ::= "extern_c" STRING_LITERAL "(" argument_list ")"

例:
extern_c "function_name" (arg1, arg2, arg3)
```

### **2. MIR命令**（既存 `extern_call` の拡張）

```json
{
  "op": "extern_call",
  "interface": "ffi.dynamic",
  "method": "llvm_compile_mir_to_object",
  "args": ["%1", "%2"],
  "dst": "%3"
}
```

**`interface` の種類**:
- `"nyrt.time"` - 既存（時刻関数）
- `"nyrt.array"` - 既存（配列操作）
- `"ffi.dynamic"` - **新規！** 動的FFI呼び出し

---

### **3. VM実装**

```rust
// src/backend/mir_interpreter/helpers/externs.rs
pub fn call_extern(interface: &str, method: &str, args: &[Value]) -> Result<Value> {
    match interface {
        "ffi.dynamic" => call_dynamic_ffi(method, args),  // 新規！
        "nyrt.time" => handle_time_externs(method, args),
        "nyrt.array" => handle_array_externs(method, args),
        _ => Err(format!("Unknown extern interface: {}", interface).into())
    }
}

fn call_dynamic_ffi(symbol: &str, args: &[Value]) -> Result<Value> {
    use libloading::{Library, Symbol};

    // 現在のプロセスから関数を探す
    // (メインEXE + リンクされた .so/.a から)
    let lib = unsafe { Library::new(std::ptr::null())? };

    match args.len() {
        // 2引数版（最も一般的）
        2 => {
            type Func2 = unsafe extern "C" fn(*const c_char, *const c_char) -> i64;
            let func: Symbol<Func2> = unsafe { lib.get(symbol.as_bytes())? };

            let arg0 = args[0].as_cstring()?;
            let arg1 = args[1].as_cstring()?;
            let result = unsafe { func(arg0, arg1) };

            Ok(Value::Integer(result))
        }

        // 1引数版
        1 => {
            type Func1 = unsafe extern "C" fn(*const c_char) -> i64;
            let func: Symbol<Func1> = unsafe { lib.get(symbol.as_bytes())? };

            let arg0 = args[0].as_cstring()?;
            let result = unsafe { func(arg0) };

            Ok(Value::Integer(result))
        }

        // 0引数版
        0 => {
            type Func0 = unsafe extern "C" fn() -> i64;
            let func: Symbol<Func0> = unsafe { lib.get(symbol.as_bytes())? };

            let result = unsafe { func() };

            Ok(Value::Integer(result))
        }

        _ => Err(format!("Unsupported argument count for FFI: {}", args.len()).into())
    }
}
```

---

### **4. LLVM AOT実装**

```python
# src/llvm_py/llvm_builder.py
def handle_extern_call(self, inst):
    interface = inst.get("interface", "")
    method = inst.get("method", "")

    if interface == "ffi.dynamic":
        # 外部関数宣言
        arg_types = [ir.IntType(8).as_pointer()] * len(inst["args"])
        func_type = ir.FunctionType(ir.IntType(64), arg_types)

        if method not in self.module.globals:
            ir.Function(self.module, func_type, name=method)

        # 呼び出し
        func = self.module.get_global(method)
        args = [self.get_value(arg) for arg in inst["args"]]
        result = self.builder.call(func, args)

        self.values[inst["dst"]] = result
    else:
        # 既存のextern処理
        self.handle_registered_extern(inst)
```

**生成されるLLVM IR**:
```llvm
; 宣言（リンク時に解決される）
declare i64 @llvm_compile_mir_to_object(i8*, i8*)

; 呼び出し
%result = call i64 @llvm_compile_mir_to_object(i8* %mir_json, i8* %output_path)
```

---

## 🏗️ **ネイティブライブラリ実装例**

### **libllvm_backend.so の C API**

```c
// libs/llvm_backend/llvm_backend.h
#ifndef LLVM_BACKEND_H
#define LLVM_BACKEND_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

// MIR JSON → .o コンパイル
int64_t llvm_compile_mir_to_object(const char* mir_json_path, const char* output_path);

// MIR JSON → .ll (LLVM IR) 出力
int64_t llvm_compile_mir_to_ll(const char* mir_json_path, const char* output_path);

// 最適化レベル設定
void llvm_set_opt_level(int32_t level);  // 0-3

#ifdef __cplusplus
}
#endif

#endif
```

### **実装（Rust wrapper）**

```rust
// libs/llvm_backend/src/lib.rs
use std::ffi::CStr;
use std::os::raw::c_char;

#[no_mangle]
pub extern "C" fn llvm_compile_mir_to_object(
    mir_json_path: *const c_char,
    output_path: *const c_char
) -> i64 {
    let mir_path = unsafe {
        if mir_json_path.is_null() { return -1; }
        CStr::from_ptr(mir_json_path).to_str().unwrap_or("")
    };

    let out_path = unsafe {
        if output_path.is_null() { return -1; }
        CStr::from_ptr(output_path).to_str().unwrap_or("")
    };

    // Python llvmlite harness を呼ぶ
    let result = std::process::Command::new("python3")
        .arg("tools/llvmlite_harness.py")
        .arg(mir_path)
        .arg("-o")
        .arg(out_path)
        .status();

    match result {
        Ok(status) if status.success() => 0,
        _ => -1,
    }
}
```

**ビルド**:
```bash
cd libs/llvm_backend
cargo build --release --crate-type cdylib

# 出力:
# target/release/libllvm_backend.so (Linux)
# target/release/libllvm_backend.dylib (macOS)
# target/release/llvm_backend.dll (Windows)
```

---

## 📋 **実装スケジュール**

### **Week 1: `extern_c` VM実装（MVP）**

#### Day 1-2: パーサー拡張
- `src/front/parser/statement.rs` に `parse_extern_c_call()` 追加
- AST定義追加: `ASTNode::ExternCCall { function, args }`
- スモークテスト: `extern_c "strlen" ("hello")` が動く

#### Day 3-4: MIR Builder拡張
- `src/mir/compiler.rs` に `compile_extern_c_call()` 追加
- `extern_call` 命令に `"ffi.dynamic"` interface 追加
- スモークテスト: MIR JSON正しく生成される

#### Day 5-7: VM実行拡張
- `src/backend/mir_interpreter/helpers/externs.rs` 拡張
- `libloading` クレート追加
- `call_dynamic_ffi()` 実装（1/2/3引数対応）
- スモークテスト: `extern_c "getpid" ()` が動く

---

### **Week 2: ネイティブライブラリ作成**

#### Day 1-3: libllvm_backend.so
- `libs/llvm_backend/` プロジェクト作成
- `llvm_compile_mir_to_object()` 実装
- Python llvmlite harness 統合
- スモークテスト: 簡単なMIR JSON → .o コンパイル成功

#### Day 4-5: libhako_boxes.so
- StringBox/ArrayBox/MapBox を Hako ABI形式で出力
- TypeBox descriptor 定義
- スモークテスト: Hakorune から呼び出し成功

#### Day 6-7: 統合テスト
- Hakoruneスクリプトから `extern_c "llvm_compile_mir_to_object"` 呼び出し
- .hako → .o パイプライン確立
- スモークテスト: 簡単な .hako が .o にコンパイルされる

---

### **Week 3: LLVM AOT対応**

#### Day 1-3: llvm_builder.py 拡張
- `handle_extern_call()` に `ffi.dynamic` 対応追加
- 外部関数宣言生成（`declare i64 @func(i8*, ...)`）
- スモークテスト: LLVM IR 正しく生成される

#### Day 4-5: リンクテスト
- .o ファイル生成成功
- `clang` でリンク成功（libllvm_backend.so 含む）
- スモークテスト: 生成されたEXEが動く

#### Day 6-7: 統合テスト
- VM実行 vs LLVM AOT実行 比較
- パリティテスト（同じ結果になることを確認）

---

### **Week 4: セルフホスト統合**

#### Day 1-3: apps/selfhost/compiler.hako 実装
- `Compiler` Box 実装
- `compile_to_object()` メソッド実装
- `Main.main()` でパイプライン実行

#### Day 4-5: セルフホストテスト
- `hako apps/selfhost/compiler.hako` 実行
- すべてのコンポーネント（parser/mir_builder/vm/main）が .o にコンパイルされる
- `clang` でリンク成功

#### Day 6-7: 完全セルフホストテスト
- `hakorune-selfhost.exe` が自分自身をコンパイル可能
- パリティテスト（Rust版 vs selfhost版）
- パフォーマンステスト

---

## ✅ **各Phaseの完了基準（DoD）**

### **Phase 1: Rust VM (extern_c サポート追加)**
- ✅ `extern_c "function" (args)` 構文がパース可能
- ✅ MIR `extern_call` 命令に `"ffi.dynamic"` 追加
- ✅ VM実行時に `dlsym()` でシンボル解決成功
- ✅ 3つのテストケース PASS:
  - `extern_c "getpid" ()` → プロセスID取得
  - `extern_c "strlen" ("hello")` → 5 を返す
  - `extern_c "system" ("echo test")` → コマンド実行成功
- ✅ quick-selfhost: 170/185 PASS 維持

---

### **Phase 2: ネイティブライブラリ**
- ✅ `libllvm_backend.so` ビルド成功
- ✅ `llvm_compile_mir_to_object(mir, out)` 関数動作
- ✅ Hakorune から呼び出し成功:
  ```hakorune
  local result = extern_c "llvm_compile_mir_to_object" (
      "test.mir.json", "test.o"
  )
  ```
- ✅ 生成された `test.o` が正しいマシンコード
- ✅ `clang test.o -o test.exe` でリンク成功
- ✅ `./test.exe` 実行成功

---

### **Phase 3: LLVM AOT対応**
- ✅ `src/llvm_py/llvm_builder.py` が `ffi.dynamic` 対応
- ✅ LLVM IR 生成時に `declare` + `call` 正しく出力
- ✅ .hako → MIR JSON → LLVM IR → .o → EXE パイプライン成功
- ✅ VM実行 vs LLVM AOT実行 のパリティテスト PASS
- ✅ パフォーマンス: LLVM版が VM版より速い（期待値）

---

### **Phase 4: セルフホスト統合**
- ✅ `apps/selfhost/compiler.hako` 実装完了
- ✅ `hako apps/selfhost/compiler.hako` でビルド成功
- ✅ 生成物:
  - `build/parser.o`
  - `build/mir_builder.o`
  - `build/vm.o`
  - `build/main.o`
- ✅ `clang build/*.o -o hakorune-selfhost.exe` 成功
- ✅ `hakorune-selfhost.exe program.hako` 実行成功
- ✅ **完全セルフホスト**: `hakorune-selfhost.exe` が自分自身をビルド可能
- ✅ パリティテスト: Rust版 vs selfhost版 で同じ結果
- ✅ Rust 99,406行 → 残存 < 5,000行（95%削減）

---

## 🚨 **Rollback戦略**

### **Level 1: Feature無効（2分）**
```bash
# extern_c 無効化（既存動作に戻る）
git checkout src/front/parser/statement.rs
git checkout src/mir/compiler.rs
cargo build --release
```

### **Level 2: VM拡張削除（5分）**
```bash
rm src/backend/mir_interpreter/helpers/externs.rs.bak
git checkout src/backend/mir_interpreter/helpers/externs.rs
cargo build --release
bash tools/smokes/v2/run.sh --profile quick-selfhost
```

### **Level 3: Full Rollback（30分）**
```bash
git revert --no-commit HEAD~10..HEAD
git commit -m "Rollback: extern_c 実装全削除"
rm -rf libs/llvm_backend/
cargo build --release
bash tools/smokes/v2/run.sh --profile quick-selfhost
# 170/185 PASS 復帰確認
```

---

## 🔥 **革命的なポイント**

### **1. Rustと同じ発想**
```rust
// Rust
extern "C" { fn foo(); }

// Hakorune
extern_c "foo" ()
```
↑ まったく同じコンセプト！学習コスト低い

### **2. Rust層の完全最小化**
```
Before: Rust 99,406行（Parser/MIR/VM/Backend すべてRust）
After:  Rust < 5,000行（最小VMエンジンのみ）
削減率: 95%
```

### **3. 段階的実装可能**
- Week 1: VM対応のみ（LLVM AOT不要）
- Week 2: ライブラリ作成（既存技術）
- Week 3: LLVM AOT対応（拡張）
- Week 4: セルフホスト統合（ゴール）

### **4. 既存資産活用**
- LLVM/llvmlite - そのまま使える
- 既存plugins/ - そのまま使える
- MIR JSON仕様 - 変更不要

### **5. デバッグ可能**
```bash
# Hakoruneスクリプトレベルでトレース
HAKO_VM_TRACE="op=externcall" ./hako apps/selfhost/compiler.hako

# 出力:
# [vm] externcall ffi.dynamic llvm_compile_mir_to_object args=[...]
```

---

## 📊 **期待される成果**

### **技術的成果**
1. ✅ Rust層 95%削減（99,406行 → < 5,000行）
2. ✅ 完全セルフホストコンパイラ達成
3. ✅ VM/LLVM AOT 両対応
4. ✅ C/Rust資産を直接活用可能
5. ✅ プラグインシステムとの完全統合

### **開発速度**
- Week 1: extern_c VM実装
- Week 2: ライブラリ作成
- Week 3: LLVM AOT対応
- Week 4: セルフホスト統合
- **合計: 1ヶ月で完全セルフホスト達成！**

### **保守性**
- Hakoruneスクリプトは読みやすい（Rust不要）
- デバッグ容易（トレース可能）
- 段階的改善可能（C関数を追加するだけ）

---

## 🎯 **次のアクション**

1. **ChatGPTと相談**: この戦略の妥当性確認
2. **Week 1開始**: Parser拡張から着手
3. **スモークテスト整備**: extern_c 専用テスト追加
4. **ドキュメント更新**: MIR命令セット仕様に `ffi.dynamic` 追加

---

**作成者**: Claude (Sonnet 4.5)
**作成日**: 2025-10-14
**バージョン**: 完全統合版
**核心発見**: "Rustと同じことをする" ← シンプルで強力！

