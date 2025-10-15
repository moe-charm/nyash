# Rust最小化計画（Phase 15.77）

## 🎯 目標

**99,406行 → 100-200行（99.8%削減）**

Rust層を極限まで薄くし、VM実行エンジン+FFI基盤のみ残す。

---

## 📊 削減計画の全体像

### Phase 15.75開始時（2025-10-05）
```
総行数: 99,406行

内訳:
├── Parser層        : ~30,000行
├── AST層           : ~10,000行
├── MIR Builder層   : ~40,000行
├── VM層            : ~5,000行
├── FFI/Runtime層   : ~2,000行
└── その他          : ~12,406行
```

### Phase 15.77完了時（目標）
```
総行数: 100-200行

内訳:
├── main.rs         : ~50行（エントリーポイント）
├── VM実行エンジン  : ~100行（最小ループ）
├── FFI基盤         : ~50行（extern_c実行）
└── その他削除不可  : ~0行
```

---

## 📅 週次削減計画

### Week 3（2025-11-23 - 11-29）Parser削除

#### 削除対象（~40,000行）
```
src/front/parser_layer/
├── parser.rs               (~5,000行) ❌
├── lexer.rs                (~3,000行) ❌
├── token.rs                (~1,000行) ❌
└── grammar/                (~8,000行) ❌

src/front/ast/
├── ast_nodes.rs            (~4,000行) ❌
├── ast_visitor.rs          (~2,000行) ❌
├── ast_printer.rs          (~1,000行) ❌
└── ast_validator.rs        (~3,000行) ❌

その他Parser関連:
├── src/front/syntax/       (~8,000行) ❌
└── src/front/source_map/   (~5,000行) ❌
```

#### 置き換え方法
```rust
// Before（Rust Parser）
fn parse_source(source: &str) -> Result<Ast> {
    let tokens = lexer::tokenize(source)?;
    let ast = parser::parse(tokens)?;
    Ok(ast)
}

// After（凍結EXE経由）
fn parse_source_frozen(source: &str) -> Result<Mir> {
    // hako-frozen-v1を呼び出し
    let mir_json = Command::new("hako-frozen-v1")
        .arg("--emit-mir-json")
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?
        .wait_with_output()?;

    let mir = serde_json::from_slice(&mir_json.stdout)?;
    Ok(mir)
}
```

#### スモークテスト修正
```bash
# Before
./hako program.hako  # Rust Parser使用

# After
./hako program.hako  # 凍結EXE経由でParser実行
```

#### 期待削減
- **削減**: ~40,000行
- **残り**: ~59,000行（59.4%削減）

---

### Week 4（2025-11-30 - 12-06）MIR Builder削除準備

#### 検証対象（~40,000行）
```
src/backend/mir_builder/
├── builder.rs              (~8,000行) 検証
├── expression.rs           (~10,000行) 検証
├── statement.rs            (~8,000行) 検証
├── control_flow.rs         (~6,000行) 検証
└── type_inference.rs       (~8,000行) 検証
```

#### 並行動作確認
```rust
// Rust MIR Builder
fn build_mir_rust(ast: &Ast) -> Result<Mir> {
    let mut builder = MirBuilder::new();
    builder.visit_ast(ast)?;
    Ok(builder.finish())
}

// Hakorune MIR Builder
fn build_mir_hakorune(ast_json: &str) -> Result<Mir> {
    let mir_json = Command::new("hako")
        .arg("apps/selfhost-compiler/mir_builder.hako")
        .arg("--input")
        .arg(ast_json)
        .output()?;

    let mir = serde_json::from_slice(&mir_json.stdout)?;
    Ok(mir)
}

// 両方動作確認
#[test]
fn test_mir_builder_parity() {
    let source = "local x = 42; return x;";
    let ast = parse_source(source).unwrap();

    let mir_rust = build_mir_rust(&ast).unwrap();
    let mir_hako = build_mir_hakorune(&ast_to_json(&ast)).unwrap();

    assert_eq!(mir_rust, mir_hako);  // パリティ確認
}
```

#### Week 4では削除しない
- 理由: パリティ確認・安全性検証
- 削除: Week 5で実行

---

### Week 5（2025-12-07 - 12-13）MIR Builder削除実行

#### 削除実行（~40,000行）
```bash
# MIR Builder完全削除
rm -rf src/backend/mir_builder/

# Hakorune実装をデフォルトへ
# src/main.rs
fn build_mir(source: &str) -> Result<Mir> {
    // Hakorune MIR Builder使用
    build_mir_hakorune(source)
}
```

#### スモークテスト確認
```bash
# 全テスト実行（Hakorune MIR Builder使用）
bash tools/smokes/v2/run.sh --profile quick-selfhost

# 期待結果
# Total: 185
# Passed: 170
# Failed: 15
```

#### 期待削減
- **削減**: ~40,000行
- **残り**: ~19,000行（80.9%削減）

---

### Week 6（2025-12-14 - 12-20）雑多なコード削除

#### 削除対象（~18,800行）
```
未使用依存削除:
├── serde_derive        ❌（AST/MIR生成不要）
├── syn                 ❌（Parser不要）
├── quote               ❌（コード生成不要）
└── proc-macro2         ❌（マクロ不要）

デッドコード削除:
├── src/utils/          (~2,000行) ❌
├── src/diagnostics/    (~3,000行) ❌（凍結EXE側）
├── src/optimizer/      (~5,000行) ❌（LLVM側）
└── src/codegen/        (~8,000行) ❌（LLVM側）

テスト削除:
├── tests/parser/       (~500行) ❌
├── tests/mir_builder/  (~300行) ❌
└── 残すテスト: tests/vm/ tests/ffi/ のみ
```

#### Cargo.toml整理
```toml
# Before（多数の依存）
[dependencies]
serde = "1.0"
serde_derive = "1.0"
serde_json = "1.0"
syn = "2.0"
quote = "1.0"
proc-macro2 = "1.0"
clap = "4.0"
# ... 30個以上

# After（最小限）
[dependencies]
serde = "1.0"          # MIR JSONデシリアライズ
serde_json = "1.0"     # MIR JSON読み込み
libloading = "0.8"     # FFI動的ロード
# 合計3個
```

#### 期待削減
- **削減**: ~18,800行
- **残り**: ~200行（99.8%削減）

---

## 🎯 最終構成（目標100-200行）

### ファイル構成
```
src/
├── main.rs                 (~50行) ✅
├── vm/
│   └── executor.rs        (~100行) ✅
├── ffi/
│   └── extern_c.rs        (~50行) ✅
└── lib.rs                 (~10行) ✅

Cargo.toml                 (~30行) ✅
build.rs                   (~20行) ✅ (FFIビルド用)

合計: ~260行（少しオーバー、調整必要）
```

### src/main.rs（~50行）
```rust
use std::process::Command;
use serde_json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // Step 1: 凍結EXE経由でMIR取得
    let mir_json = Command::new("hako-frozen-v1")
        .arg("--emit-mir-json")
        .arg(&args[1])
        .output()?
        .stdout;

    // Step 2: MIRデシリアライズ
    let mir: Mir = serde_json::from_slice(&mir_json)?;

    // Step 3: VM実行
    let result = vm::execute(mir)?;

    println!("{}", result);
    Ok(())
}

// MIR構造体（最小限）
#[derive(serde::Deserialize)]
struct Mir {
    functions: Vec<Function>,
    entry: String,
}

#[derive(serde::Deserialize)]
struct Function {
    name: String,
    blocks: Vec<BasicBlock>,
}

#[derive(serde::Deserialize)]
struct BasicBlock {
    instructions: Vec<Instruction>,
}

#[derive(serde::Deserialize)]
enum Instruction {
    Const { dst: u32, value: Value },
    BinOp { dst: u32, lhs: u32, rhs: u32, op: BinOp },
    Call { dst: u32, callee: String, args: Vec<u32> },
    ExternC { dst: u32, symbol: String, args: Vec<u32> },
    Ret { value: u32 },
    // ... 16命令のみ
}
```

### src/vm/executor.rs（~100行）
```rust
use crate::Mir;
use std::collections::HashMap;

pub fn execute(mir: Mir) -> Result<Value, String> {
    let mut vm = VM::new(mir);
    vm.run()
}

struct VM {
    mir: Mir,
    registers: Vec<Value>,
    current_block: usize,
    current_instruction: usize,
}

impl VM {
    fn new(mir: Mir) -> Self {
        Self {
            mir,
            registers: vec![Value::Null; 256],
            current_block: 0,
            current_instruction: 0,
        }
    }

    fn run(&mut self) -> Result<Value, String> {
        loop {
            let func = &self.mir.functions[0];  // エントリー関数
            let block = &func.blocks[self.current_block];
            let inst = &block.instructions[self.current_instruction];

            match inst {
                Instruction::Const { dst, value } => {
                    self.registers[*dst as usize] = value.clone();
                }
                Instruction::BinOp { dst, lhs, rhs, op } => {
                    let lhs = &self.registers[*lhs as usize];
                    let rhs = &self.registers[*rhs as usize];
                    self.registers[*dst as usize] = binop(lhs, rhs, *op);
                }
                Instruction::ExternC { dst, symbol, args } => {
                    let result = crate::ffi::call_extern(symbol, args, &self.registers)?;
                    self.registers[*dst as usize] = result;
                }
                Instruction::Ret { value } => {
                    return Ok(self.registers[*value as usize].clone());
                }
                // ... 他の命令
            }

            self.current_instruction += 1;
        }
    }
}

fn binop(lhs: &Value, rhs: &Value, op: BinOp) -> Value {
    match (lhs, rhs, op) {
        (Value::Int(a), Value::Int(b), BinOp::Add) => Value::Int(a + b),
        (Value::Int(a), Value::Int(b), BinOp::Sub) => Value::Int(a - b),
        // ... 基本演算のみ
        _ => panic!("unsupported binop"),
    }
}
```

### src/ffi/extern_c.rs（~50行）
```rust
use libloading::{Library, Symbol};
use std::collections::HashMap;

static mut ALLOWLIST: Vec<String> = Vec::new();

pub fn call_extern(
    symbol: &str,
    args: &[u32],
    registers: &[Value],
) -> Result<Value, String> {
    // Step 1: 許可チェック
    unsafe {
        if !ALLOWLIST.contains(&symbol.to_string()) {
            return Err(format!("ExternCDenied: {}", symbol));
        }
    }

    // Step 2: ライブラリロード
    let lib = Library::new("libc.so.6")
        .map_err(|e| format!("Failed to load library: {}", e))?;

    // Step 3: シンボル取得
    unsafe {
        let func: Symbol<extern "C" fn() -> i64> = lib.get(symbol.as_bytes())
            .map_err(|e| format!("Symbol not found: {}", e))?;

        // Step 4: 実行（引数0個のみ簡略化）
        let result = func();
        Ok(Value::Int(result))
    }
}

pub fn init_allowlist() {
    unsafe {
        ALLOWLIST = vec![
            "getpid".to_string(),
            "strlen".to_string(),
            "system".to_string(),
        ];

        // ENV変数から追加
        if let Ok(list) = std::env::var("HAKO_FFI_ALLOW_LIST") {
            for symbol in list.split(',') {
                ALLOWLIST.push(symbol.to_string());
            }
        }
    }
}
```

---

## ✅ 各週の検証ポイント

### Week 3: Parser削除後
```bash
# 動作確認
./hako local_tests/hello.nyash
# Expected: Hello, World!

# スモークテスト
bash tools/smokes/v2/run.sh --profile quick-selfhost
# Expected: 170 PASS / 15 FAIL

# 行数確認
tokei src/
# Expected: ~59,000行
```

### Week 4: MIR Builder削除準備
```bash
# パリティテスト
cargo test test_mir_builder_parity
# Expected: ok

# 並行動作確認
HAKO_USE_RUST_MIR_BUILDER=1 ./hako test.hako  # Rust
HAKO_USE_RUST_MIR_BUILDER=0 ./hako test.hako  # Hakorune
# Expected: 両方OK
```

### Week 5: MIR Builder削除後
```bash
# スモークテスト（Hakoruneのみ）
bash tools/smokes/v2/run.sh --profile quick-selfhost
# Expected: 170 PASS / 15 FAIL

# 行数確認
tokei src/
# Expected: ~19,000行
```

### Week 6: 最終調整後
```bash
# 最終動作確認
./hako local_tests/hello.nyash
./hako --backend llvm local_tests/arithmetic.nyash

# 最終スモークテスト
bash tools/smokes/v2/run.sh --profile quick-selfhost
# Expected: 170 PASS / 15 FAIL

# 最終行数確認
tokei src/
# Expected: ~200行
```

---

## ⚠️ ロールバック手順

### 各週でロールバック可能
```bash
# Week 3で問題発生 → Week 2に戻る
git revert HEAD~10  # Week 3のコミット取り消し
cargo build --release
bash tools/smokes/v2/run.sh --profile quick-selfhost

# Week 5で問題発生 → Week 4に戻る
git revert HEAD~20  # Week 5のコミット取り消し
# ... 同様
```

### 凍結EXEへの完全ロールバック
```bash
# 最悪の場合: 凍結EXE使用に戻る
cp hako-frozen-v1 ./hako
./hako program.hako  # 凍結EXE経由で安定動作
```

---

## 📊 削減行数の追跡

### Git commit messageテンプレート
```
Phase 15.77: Week X - [作業内容]

削減行数: ~XX,XXX行
残り行数: ~XX,XXX行
削減率: XX.X%

動作確認:
- スモークテスト: 170 PASS / 15 FAIL
- AOT導線: OK

関連: #issue-number
```

### 週次レポート
```markdown
## Week X削減レポート

### 削減内容
- [削除したディレクトリ/ファイル]

### 削減行数
- 削減: ~XX,XXX行
- 残り: ~XX,XXX行
- 削減率: XX.X%

### 動作確認
- [ ] スモークテスト: 170 PASS維持
- [ ] AOT導線動作
- [ ] 凍結EXEロールバック確認

### 問題点
- [なし/あれば記載]
```

---

**作成日**: 2025-10-14
**Phase**: 15.77（Week 3-6）
**目標**: 99,406行 → 100-200行（99.8%削減）
