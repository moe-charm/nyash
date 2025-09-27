# MIR Call系命令の現状（2025-09-23）

## 📊 現在のCall系命令（6種類）

### 1. Call（関数呼び出し）
```rust
Call {
    dst: Option<ValueId>,
    func: ValueId,           // 関数オブジェクト（ValueId）
    callee: Option<Callee>,  // ← 今日追加した型安全解決
    args: Vec<ValueId>,
    effects: EffectMask,
}
```
**用途**: 関数呼び出し全般（今日Callee型追加でシャドウイング対策）

### 2. BoxCall（Boxメソッド呼び出し）
```rust
BoxCall {
    dst: Option<ValueId>,
    box_val: ValueId,        // レシーバー
    method: String,          // メソッド名
    method_id: Option<u16>,  // 統一レジストリID
    args: Vec<ValueId>,
    effects: EffectMask,
}
```
**用途**: StringBox.concat()等、すべてのBoxメソッド

### 3. PluginInvoke（プラグイン強制）
```rust
PluginInvoke {
    dst: Option<ValueId>,
    box_val: ValueId,
    method: String,
    args: Vec<ValueId>,
    effects: EffectMask,
}
```
**用途**: プラグインメソッド呼び出し（builtinフォールバックなし）

### 4. ExternCall（環境直結）
```rust
ExternCall {
    dst: Option<ValueId>,
    iface_name: String,      // "env.console"
    method_name: String,     // "log"
    args: Vec<ValueId>,
    effects: EffectMask,
}
```
**用途**: env.console.log等、ホスト環境への直接呼び出し

### 5. NewBox（Box生成）
```rust
NewBox {
    dst: ValueId,
    box_type: String,        // "StringBox"等
    args: Vec<ValueId>,      // コンストラクタ引数
}
```
**用途**: new StringBox("hello")等

### 6. NewClosure（クロージャ生成）
```rust
NewClosure {
    dst: ValueId,
    params: Vec<String>,
    body: Vec<MirInstruction>,
    captures: Vec<(String, ValueId)>,
    me: Option<ValueId>,
}
```
**用途**: 関数リテラル、クロージャ

## 🔴 問題点

1. **重複**: CallとBoxCallの役割が重複
2. **不統一**: PluginInvokeとBoxCallが分離
3. **複雑**: 6種類のCall系命令は多すぎる
4. **メンテ困難**: 各実行器（VM/JIT/LLVM）で6種類実装が必要

## ✅ Callee型（今日追加）

```rust
pub enum Callee {
    Global(String),          // print, error等
    Method {                 // Box.method()
        box_name: String,
        method: String,
        receiver: Option<ValueId>,
        certainty: TypeCertainty, // Known/Union（型確度）
    },
    Value(ValueId),         // 第一級関数
    Extern(String),         // 外部関数
}
```

**現状**: Callのみ対応、他は未統合

## 📈 統計（ソースコード分析）

| 命令 | 使用箇所数 | 主な用途 |
|------|-----------|----------|
| Call | 多数 | 一般関数呼び出し |
| BoxCall | 多数 | Boxメソッド全般 |
| PluginInvoke | 少数 | プラグイン限定 |
| ExternCall | 少数 | print正規化後 |
| NewBox | 多数 | Box生成全般 |
| FunctionNew | 少数 | クロージャ |

## 🎯 今後の方向性

### ChatGPT5 Pro提案（A++案）
- すべてをCall + Calleeに統一
- receiverをCalleeに含める
- EffectMaskで安全性管理
- 段階的移行で実装

### 移行優先順位
1. ExternCall → Call + Callee::Extern
2. BoxCall → Call + Callee::Method
3. PluginInvoke → Call + Callee::Method（effects=Sandbox）
4. NewBox → そのまま残す（or Call + Callee::Constructor検討）

## 📝 更新履歴
- 2025-09-23: Callee型追加（ChatGPT5 Pro設計）
- 2025-09-27: Callee::Method に certainty（Known/Union）を追加
- 2025-09-23: 本ドキュメント作成
