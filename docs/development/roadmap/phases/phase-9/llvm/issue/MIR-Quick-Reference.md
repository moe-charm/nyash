# 📚 MIR クイックリファレンス for LLVM実装

## 🎯 Week 1で対応するMIR命令

### 1. **Const命令**
```rust
// MIR表現
MirInstruction::Const(value_id, constant_value)

// 例
Const(v1, MirConstant::Integer(42))
Const(v2, MirConstant::Float(3.14))  
Const(v3, MirConstant::Bool(true))

// LLVM変換
let int_val = ctx.i32_type().const_int(42, false);
let float_val = ctx.f64_type().const_float(3.14);
let bool_val = ctx.bool_type().const_int(1, false);
```

### 2. **Return命令**
```rust
// MIR表現
MirInstruction::Return(Option<ValueId>)

// 例
Return(Some(v1))  // 値を返す
Return(None)      // voidを返す

// LLVM変換
builder.build_return(Some(&value));
builder.build_return(None);
```

## 📄 参考: 現在のMIR構造

```rust
// src/mir/instruction.rs の主要部分
pub enum MirInstruction {
    // Week 1対象
    Const(ValueId, MirConstant),
    Return(Option<ValueId>),
    
    // Week 2対象
    BinOp(ValueId, BinaryOp, ValueId, ValueId),
    Compare(ValueId, CompareOp, ValueId, ValueId),
    Branch(ValueId, BasicBlockId, BasicBlockId),
    Jump(BasicBlockId),
    
    // Week 3以降
    BoxNew(ValueId, MirType),
    BoxCall(ValueId, ValueId, String, Vec<ValueId>),
    // ... 他の命令
}

// 定数の型
pub enum MirConstant {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Null,
}
```

## 🔄 MIR→LLVM変換の基本パターン

```rust
// 基本的な変換ループ
for instruction in &block.instructions {
    match instruction {
        MirInstruction::Const(value_id, constant) => {
            let llvm_value = match constant {
                MirConstant::Integer(n) => {
                    ctx.i64_type().const_int(*n as u64, true).into()
                }
                MirConstant::Float(f) => {
                    ctx.f64_type().const_float(*f).into()
                }
                MirConstant::Bool(b) => {
                    ctx.bool_type().const_int(*b as u64, false).into()
                }
                _ => todo!("Other constants"),
            };
            // value_idとllvm_valueをマッピングに保存
            value_map.insert(*value_id, llvm_value);
        }
        
        MirInstruction::Return(value_id) => {
            match value_id {
                Some(id) => {
                    let value = value_map.get(id).unwrap();
                    builder.build_return(Some(value));
                }
                None => {
                    builder.build_return(None);
                }
            }
        }
        
        _ => {} // Week 1では他の命令は無視
    }
}
```

## 🎯 テスト用のMIRサンプル

### 1. **return 42のMIR**
```rust
MirModule {
    functions: vec![
        MirFunction {
            name: "Main.main",
            params: vec![],
            return_type: MirType::Integer,
            blocks: vec![
                BasicBlock {
                    id: 0,
                    instructions: vec![
                        Const(v1, MirConstant::Integer(42)),
                        Return(Some(v1)),
                    ],
                },
            ],
        },
    ],
}
```

### 2. **簡単な計算のMIR**（Week 2用）
```rust
// return 10 + 5
BasicBlock {
    instructions: vec![
        Const(v1, MirConstant::Integer(10)),
        Const(v2, MirConstant::Integer(5)),
        BinOp(v3, BinaryOp::Add, v1, v2),
        Return(Some(v3)),
    ],
}
```

## 💡 実装のヒント

1. **ValueIdマッピング**: `HashMap<ValueId, BasicValueEnum>`で管理
2. **型情報**: MIRは型情報を持つので、LLVM型への変換テーブルを作る
3. **基本ブロック**: MIRのBasicBlockIdをLLVMのBasicBlockにマッピング
4. **エラー処理**: 最初は`todo!()`でOK、後から実装

## 📁 関連ファイル

- MIR定義: `src/mir/instruction.rs`
- MIR生成: `src/mir/lowering.rs`
- 参考実装: `src/backend/vm.rs`（VMのMIR処理）

---

**注**: このリファレンスはWeek 1の実装に必要な最小限の情報です。
詳細は実際のソースコードを参照してください。