# MIR14命令セット仕様

**Version**: Phase 15.0
**Last Updated**: 2025-09-27

## 📚 概要

MIR14（Middle-level Intermediate Representation 14）は、Nyash言語のコア中間表現です。**たった14命令**で、Interpreter/VM/LLVM/JITすべての実行形態をサポートします。

### **設計哲学**

```
Everything is Box
    ↓
Box化により命令を最小化
    ↓
14命令で完全実装 ✨
```

---

## 🎯 MIR14命令一覧

### **基本演算（5命令）**

#### 1. **Const** - 定数値生成
```rust
Const { dst: ValueId, value: ConstValue }
```
- 用途: 即値（整数/文字列/null）の生成
- 例: `const r1 = 42`

#### 2. **UnaryOp** - 単項演算
```rust
UnaryOp { dst: ValueId, op: UnaryOperator, operand: ValueId }
```
- 演算子: `not`, `-` (負数)
- 例: `r2 = not r1`

#### 3. **BinOp** - 二項演算
```rust
BinOp { dst: ValueId, op: BinaryOperator, left: ValueId, right: ValueId }
```
- 演算子: `+`, `-`, `*`, `/`, `%`, `&`, `|`, `^`, `<<`, `>>`
- 例: `r3 = r1 + r2`

#### 4. **Compare** - 比較演算
```rust
Compare { dst: ValueId, op: CompareOp, left: ValueId, right: ValueId }
```
- 演算子: `==`, `!=`, `<`, `<=`, `>`, `>=`
- 例: `r4 = r1 < r2`

#### 5. **TypeOp** - 型操作
```rust
TypeOp { dst: ValueId, op: TypeOperator, operand: ValueId, type_info: Option<TypeInfo> }
```
- 操作: `typeof`, `cast`, `is_a`
- 例: `r5 = typeof r1`

---

### **メモリ操作（2命令）**

#### 6. **Load** - メモリ読み込み
```rust
Load { dst: ValueId, source: ValueId, field: Option<String> }
```
- 用途: 変数読み込み、フィールドアクセス
- 例: `r6 = load r1.name`

#### 7. **Store** - メモリ書き込み
```rust
Store { target: ValueId, field: Option<String>, value: ValueId }
```
- 用途: 変数代入、フィールド更新
- 例: `store r1.name = r2`

---

### **制御フロー（4命令）**

#### 8. **Branch** - 条件分岐
```rust
Branch { condition: ValueId, true_target: BlockId, false_target: BlockId }
```
- 用途: if文、条件式
- 例: `branch r1 -> B2, B3`

#### 9. **Jump** - 無条件ジャンプ
```rust
Jump { target: BlockId }
```
- 用途: ループ継続、合流
- 例: `jump B1`

#### 10. **Return** - 関数リターン
```rust
Return { value: Option<ValueId> }
```
- 用途: 関数終了、値返却
- 例: `return r1`

#### 11. **Phi** - SSA合流ノード
```rust
Phi { dst: ValueId, incoming: Vec<(BlockId, ValueId)> }
```
- 用途: 制御フロー合流点での値統合
- 例: `r7 = phi [B1: r1, B2: r2]`
- **Phase 15**: PHI-on標準化（LoopForm実装）

---

### **Box操作（2命令）**

#### 12. **NewBox** - Box生成
```rust
NewBox { dst: ValueId, box_name: String, args: Vec<ValueId> }
```
- 用途: Box インスタンス生成
- 例: `r8 = newbox StringBox("hello")`
- **Note**: 生成後に自動的に `birth()` を呼び出し

#### 13. **BoxCall** - Boxメソッド呼び出し
```rust
BoxCall {
    dst: Option<ValueId>,
    receiver: ValueId,
    method: String,
    args: Vec<ValueId>,
    callee: Option<Callee>  // Phase 15追加
}
```
- 用途: メソッド呼び出し、演算子Box
- 例: `r9 = boxcall r8.length()`
- **Phase 15新機能**: Callee型による型安全化

---

### **外部連携（1命令）**

#### 14. **ExternCall** - 外部関数呼び出し
```rust
ExternCall {
    dst: Option<ValueId>,
    function: String,
    args: Vec<ValueId>
}
```
- 用途: C ABI関数、ランタイム関数
- 例: `externcall print(r1)`
- **最小5関数**: print, error, panic, exit, now

---

## 🌟 Phase 15新機能

### 1. **LoopForm - 制御構造のBox化**

```rust
LoopForm {
    header: BlockId,      // ループヘッダー
    body: BlockId,        // ループ本体
    exit: BlockId,        // ループ出口
    condition: ValueId,   // ループ条件
    phis: Vec<Phi>,      // PHIノード自動生成
}
```

**特徴**:
- 制御構造もBox化（Everything is Box完成！）
- PHI自動生成
- break/continue自動処理

**例**:
```nyash
loop(i < 10) {
    print(i)
    i = i + 1
}
```

**MIR生成**:
```
LoopForm {
    header: B1,
    body: B2,
    exit: B3,
    phis: [phi_i: [B0: 0, B2: i+1]]
}
```

---

### 2. **Callee型 - 型安全な関数呼び出し**

```rust
enum Callee {
    Global(String),                       // グローバル関数
    Method {
        box_name: String,
        method: String,
        receiver: ValueId
    },
    Value(ValueId),                       // 第一級関数
    Extern(String),                       // C ABI統合
}
```

**特徴**:
- コンパイル時型解決
- シャドウイング問題解決
- VM/LLVM両対応

**例**:
```nyash
print("hello")  // Callee::Extern("print")
obj.method()    // Callee::Method { box_name: "Obj", ... }
```

---

### 3. **演算子Box統一**

```rust
// 演算もBoxCallに統一
BinOp { op: Add, left: r1, right: r2 }
    ↓
BoxCall {
    receiver: AddOperator,
    method: "apply",
    args: [r1, r2],
    callee: Callee::Method { ... }
}
```

**特徴**:
- observe/adopt段階的移行
- デバッグ可視化（Void混入即座特定）
- Everything is Boxの完全実現

---

## 📊 命令数の進化

```
Phase 1:  27命令（汎用的）
    ↓ Box哲学による削減
Phase 2:  13命令（Core-13）
    ↓ 算術演算の追加
Phase 3:  14命令（MIR14）
    ↓ PHI-on標準化
Phase 15: 14命令 + LoopForm + Callee型
```

---

## 🎯 Everything is Boxの完全実現

### **データBox**
```
StringBox, IntegerBox, ArrayBox, MapBox...
```

### **演算子Box**（世界初！）
```
AddOperator, CompareOperator, UnaryOperator...
```

### **制御Box**（世界初！）
```
LoopForm: 制御構造もBox化
```

**結論**: すべてがBoxである → 14命令で完全実装可能 ✨

---

## 🚀 実装状況

### **Rust VM** （開発・デバッグ・検証用）
- 実装: 712行
- MIR14完全対応 ✅
- Callee型実装済み ✅
- gdb/lldbデバッグ可能 ✅

### **LLVM** （本番・最適化・配布用）
- 実装: Python/llvmlite
- MIR14完全対応 ✅
- PHI最適化 ✅
- ネイティブEXE生成 ✅

### **PyVM** （JSON v0ブリッジ専用）
- 実装: 1074行
- セルフホスティング・using処理専用
- MIR14対応 ✅

---

## 📚 関連ドキュメント

- [INSTRUCTION_SET.md](../../../../reference/mir/INSTRUCTION_SET.md): 命令詳細
- [PHI Policy](../../../../reference/mir/phi_policy.md): PHI設計方針
- [LoopForm Design](../../../../reference/architecture/loopform-design.md): LoopForm詳細
- [Callee Revolution](../../../../development/architecture/mir-callee-revolution.md): Callee型設計

---

## 🎉 結論

**MIR14は、Everything is Box哲学の完全実装により、たった14命令で全実行形態をサポートする革新的IR です。**

世界初の成果:
- ✅ データ/演算/制御すべてBox化
- ✅ 14命令で完全実装
- ✅ VM/LLVM両対応
- ✅ 型安全な関数呼び出し

**これが、Nyash言語の実行基盤です。** ✨