# MIR→LLVM変換計画

Date: 2025-08-31  
Status: Draft

## 📊 変換マッピング概要

### Core-15命令のLLVM IR対応（第三案・Box統一）

#### 1. 基本演算(5)
```rust
// Const
MIR::Const(i64) → LLVMConstInt(i64_type, val)
MIR::Const(f64) → LLVMConstReal(f64_type, val)
MIR::Const(bool) → LLVMConstInt(i1_type, val)
MIR::Const(string) → @nyash_string_new(ptr, len)

// UnaryOp
MIR::UnaryOp(Neg, x) → LLVMBuildNeg(x) / LLVMBuildFNeg(x)
MIR::UnaryOp(Not, x) → LLVMBuildNot(x)

// BinOp
MIR::BinOp(Add, a, b) → LLVMBuildAdd(a, b) / LLVMBuildFAdd(a, b)
MIR::BinOp(Sub, a, b) → LLVMBuildSub(a, b) / LLVMBuildFSub(a, b)
// 注: Box型の場合は@nyash_box_add等のランタイム呼び出し

// Compare
MIR::Compare(Eq, a, b) → LLVMBuildICmp(EQ, a, b) / LLVMBuildFCmp(OEQ, a, b)
MIR::Compare(Lt, a, b) → LLVMBuildICmp(SLT, a, b) / LLVMBuildFCmp(OLT, a, b)

// TypeOp
MIR::TypeOp(Check, val, type) → @nyash_type_check(val, type_id)
MIR::TypeOp(Cast, val, type) → @nyash_type_cast(val, type_id)
```

#### 2. メモリ(2)
```rust
// Load
MIR::Load(local_id) → LLVMBuildLoad(local_ptr[local_id])

// Store
MIR::Store(local_id, val) → LLVMBuildStore(val, local_ptr[local_id])
```

#### 3. 制御(4)
```rust
// Branch
MIR::Branch(cond, then_bb, else_bb) → LLVMBuildCondBr(cond, then_bb, else_bb)

// Jump
MIR::Jump(bb) → LLVMBuildBr(bb)

// Return
MIR::Return(val) → LLVMBuildRet(val)
MIR::Return(None) → LLVMBuildRetVoid()

// Phi
MIR::Phi([(bb1, val1), (bb2, val2)]) → {
    phi = LLVMBuildPhi(type)
    LLVMAddIncoming(phi, [val1, val2], [bb1, bb2])
}
```

#### 4. Box操作(3)
```rust
// NewBox
MIR::NewBox(type_name, args) → @nyash_box_new(type_id, args_ptr, args_len)

// BoxCall（注釈活用・名前/スロット両対応）
MIR::BoxCall(obj, method, args) → {
    if annotations.inline_hint {
        // インライン展開候補
        LLVMSetInlineHint(call)
    }
    if annotations.method_id { /* vtableスロット解決 */ } 
    else { @nyash_box_call_by_name(obj, method_name, args) }
}
// PluginInvoke は BoxCall に統一（Optimizerで正規化）
```

#### 5. 配列（BoxCallに統一）
```rust
// Arrayは BoxCall("get"/"set") で表現
// Lowering方針は2段階:
//  (A) 安全パス: @nyash_array_get/@nyash_array_set を呼ぶ（ランタイム側で境界/バリア）
//  (B) 型特化: 注釈/型情報が十分な場合に inline 化（bounds check + GEP + load/store + write barrier）
```

#### 6. 外部呼び出し(1)
```rust
// ExternCall
MIR::ExternCall("env.console.log", args) → @nyash_console_log(args)
MIR::ExternCall("env.gc.collect", []) → @nyash_gc_collect()
MIR::ExternCall("env.runtime.checkpoint", []) → @nyash_safepoint()
```

## 🎯 注釈システムの活用

### 1. 最適化ヒント
```rust
pub struct OptimizationHints {
    pub inline: Option<InlineHint>,      // always/never/hint
    pub pure: bool,                      // 副作用なし
    pub no_escape: bool,                 // エスケープしない
    pub hot: bool,                       // ホットパス
    pub cold: bool,                      // コールドパス
}
```

### 2. GCヒント
```rust
pub struct GcHint {
    pub no_barrier: bool,        // バリア不要（新規オブジェクトへの書き込み等）
    pub immortal: bool,          // 不死オブジェクト（定数等）
    pub thread_local: bool,      // スレッドローカル（並列GCで重要）
}
```

### 3. 型情報ヒント
```rust
pub struct TypeHint {
    pub concrete_type: Option<String>,   // 具体的な型が判明
    pub never_null: bool,               // NULL不可
    pub value_range: Option<(i64, i64)>, // 値の範囲
}
```

## 🔧 LLVM属性の活用

### 関数属性
```llvm
; 純粋関数
define i64 @add(i64 %a, i64 %b) #0 {
  %result = add i64 %a, %b
  ret i64 %result
}
attributes #0 = { nounwind readnone speculatable }

; GC セーフポイント
define void @long_loop() gc "nyash-gc" {
  ; ループバックエッジにセーフポイント
  call void @llvm.experimental.gc.statepoint(...)
}
```

### メモリ属性
```llvm
; Box用アドレス空間（1）
%box_ptr = addrspace(1)* %obj

; TBAA（Type-Based Alias Analysis）
!0 = !{!"nyash.box"}
!1 = !{!"nyash.integer", !0}
!2 = !{!"nyash.string", !0}
```

## 📈 段階的実装計画

### Phase 1: 基本変換（1週間）
- [ ] inkwell セットアップ
- [ ] 基本演算・メモリ・制御の変換
- [ ] 最小限の実行可能コード生成

### Phase 2: Box統合（1週間）
- [ ] NewBox/BoxCall実装（PluginInvokeはOptimizerでBoxCallに正規化）
- [ ] ランタイム経由の安全パス（by-name/slot）
- [ ] 基本的なGCバリア（安全パスはランタイム関数内で処理）

### Phase 3: 最適化（1週間）
- [ ] 注釈システム統合
- [ ] インライン展開
- [ ] エスケープ解析

### Phase 4: 高度な最適化（1週間）
- [ ] 脱箱化（Box → プリミティブ）
- [ ] TBAA統合
- [ ] ベクトル化ヒント

## 🎨 コード例

```rust
// MIR
function add(a: Box, b: Box) -> Box {
    %1 = Load $a
    %2 = Load $b
    %3 = BoxCall(%1, "add", [%2])
    Return %3
}

// LLVM IR（最適化前・安全パス）
define i8* @add(i8* %a, i8* %b) {
    %1 = call i8* @nyash_box_call(i8* %a, i8* @.str.add, i8** %b, i64 1)
    ret i8* %1
}

// LLVM IR（最適化後 - IntegerBox特化）
define i64 @add(i64 %a, i64 %b) {
    %1 = add i64 %a, %b
    ret i64 %1
}
```

## 🚀 期待される効果

1. **実行速度**: 2-3倍高速化
2. **メモリ使用量**: 脱箱化で50%削減
3. **バイナリサイズ**: 最適化で30%削減
4. **ビルド時間**: Cranelift比で50%削減
