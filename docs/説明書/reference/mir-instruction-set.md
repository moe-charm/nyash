# 🔧 Nyash MIR Instruction Set - Complete Reference

*ChatGPT5設計・20命令以内コア + intrinsic逃がし戦略*

## 🎯 **現在の実装状況分析**

### ⚠️ **「太り過ぎ」問題確認**
- **現在実装**: **35命令**（175%超過）
- **ChatGPT5推奨**: **20命令以内**
- **対策**: **intrinsic逃がし** + **Tier-0コア集約**

### 📊 **命令分類・整理必要度**

| 分類 | 現在命令数 | 推奨数 | 優先度 | 対策 |
|------|------------|--------|--------|------|
| **基本演算** | 8個 | 3個 | 🔴 緊急 | BinOp統合 |
| **制御フロー** | 4個 | 4個 | ✅ 適正 | 維持 |
| **メモリ** | 12個 | 3個 | 🔴 緊急 | intrinsic逃がし |
| **Box操作** | 6個 | 2個 | 🟡 要整理 | 統合検討 |
| **Future/Weak** | 5個 | 2個 | 🟡 要整理 | 段階実装 |

## 🔧 **ChatGPT5推奨: Tier-0 Core (15命令)**

### **1. 算術・比較（3命令）**
```mir
// 統合命令1: 定数ロード
Const { dst: ValueId, value: ConstValue }
// 使用例: %1 = const 42, %2 = const "hello", %3 = const null

// 統合命令2: 二項演算（算術・論理・比較すべて）
BinOp { dst: ValueId, op: BinaryOp, lhs: ValueId, rhs: ValueId }
// 使用例: %4 = %1 add %2, %5 = %1 eq %2, %6 = %1 and %2

// 統合命令3: 単項演算
UnaryOp { dst: ValueId, op: UnaryOp, operand: ValueId }
// 使用例: %7 = not %5, %8 = neg %1
```

#### **演算子統合戦略**
```rust
// 現在分離→統合へ
pub enum BinaryOp {
    // 算術（現在のBinOp）
    Add, Sub, Mul, Div, Mod,
    
    // 比較（現在のCompare統合）
    Eq, Ne, Lt, Le, Gt, Ge,
    
    // 論理（現在のBinOp統合）
    And, Or, BitAnd, BitOr, BitXor, Shl, Shr,
}

// 3つの別命令 → 1つのBinOp に統合
// BinOp + Compare + LogicalOp → BinOp
```

### **2. 制御フロー（4命令）**
```mir
// 条件分岐
Branch { condition: ValueId, then_bb: BasicBlockId, else_bb: BasicBlockId }

// 無条件ジャンプ  
Jump { target: BasicBlockId }

// 関数リターン
Return { value: Option<ValueId> }

// SSA合流（必須）
Phi { dst: ValueId, inputs: Vec<(BasicBlockId, ValueId)> }
```

### **3. 関数・メソッド（2命令）**
```mir
// 関数呼び出し（static関数・ビルトイン）
Call { dst: Option<ValueId>, func: ValueId, args: Vec<ValueId>, effects: EffectMask }

// Box メソッド呼び出し（動的ディスパッチ）
BoxCall { dst: Option<ValueId>, box_val: ValueId, method: String, args: Vec<ValueId>, effects: EffectMask }
```

### **4. Everything is Box基本（3命令）**
```mir
// Box生成（統合）
NewBox { dst: ValueId, box_type: String, args: Vec<ValueId> }
// 使用例: %obj = new_box "StringBox"("hello"), %arr = new_box "ArrayBox"()

// フィールド読み取り（統合）
Load { dst: ValueId, ptr: ValueId, field: Option<String> }
// 使用例: %val = load %obj.field, %item = load %arr[%idx]

// フィールド書き込み（統合）  
Store { value: ValueId, ptr: ValueId, field: Option<String> }
// 使用例: store %val -> %obj.field, store %item -> %arr[%idx]
```

### **5. Bus（分散・非同期一次市民）（2命令）**
```mir
// Bus送信（分散通信の核心）
Send { bus: ValueId, message: ValueId, effects: EffectMask }
// 使用例: send %p2p_bus, %message effects=[BUS]

// Bus受信
Recv { dst: ValueId, bus: ValueId, effects: EffectMask }
// 使用例: %msg = recv %p2p_bus effects=[BUS]
```

### **6. 最適化・デバッグ（1命令）**
```mir
// GC・最適化ポイント
Safepoint
// 使用例: safepoint  # GCタイミング・デバッグブレークポイント
```

## 🔄 **Tier-1: 高度最適化（5命令）**

### **必要な場合のみ追加**
```mir
// 型変換（最適化パス用）
Cast { dst: ValueId, value: ValueId, target_type: MirType }

// 動的型チェック（安全性）
TypeCheck { dst: ValueId, value: ValueId, expected_type: String }

// weak参照（Ownership-Forest用）
WeakNew { dst: ValueId, box_val: ValueId }
WeakLoad { dst: ValueId, weak_ref: ValueId }

// 何でも逃がし（複雑操作用）
Intrinsic { dst: Option<ValueId>, name: String, args: Vec<ValueId>, effects: EffectMask }
```

## 🛠️ **intrinsic逃がし戦略**

### **現在35命令→20命令削減計画**

#### **intrinsicに移行する命令（15個削除）**
```rust
// 配列操作 → intrinsic
// 現在: ArrayGet, ArraySet
// 移行後: intrinsic("array_get", [array, index]) 
//        intrinsic("array_set", [array, index, value])

// デバッグ → intrinsic  
// 現在: Debug, Print, Nop
// 移行後: intrinsic("debug", [value, message])
//        intrinsic("print", [value])

// 例外処理 → intrinsic
// 現在: Throw, Catch
// 移行後: intrinsic("throw", [exception])
//        intrinsic("catch", [exception_type])

// 参照詳細 → intrinsic
// 現在: RefNew, RefGet, RefSet, Copy
// 移行後: intrinsic("ref_new", [box])
//        intrinsic("ref_get", [ref, field])
//        intrinsic("ref_set", [ref, field, value])

// バリア → intrinsic
// 現在: BarrierRead, BarrierWrite  
// 移行後: intrinsic("barrier_read", [ptr])
//        intrinsic("barrier_write", [ptr])

// Future → intrinsic
// 現在: FutureNew, FutureSet, Await
// 移行後: intrinsic("future_new", [value])
//        intrinsic("future_set", [future, value])  
//        intrinsic("await", [future])
```

#### **intrinsic実装例**
```rust
// src/mir/intrinsics.rs
pub fn execute_intrinsic(name: &str, args: &[ValueId], effects: EffectMask) -> Result<ValueId, String> {
    match name {
        "print" => {
            let value = get_value(args[0]);
            println!("{}", value);
            Ok(ValueId::void())
        }
        
        "array_get" => {
            let array = get_value(args[0]);
            let index = get_value(args[1]);
            Ok(array.get_element(index)?)
        }
        
        "future_new" => {
            let value = get_value(args[0]);
            let future = FutureBox::new_with_value(value);
            Ok(ValueId::from_box(future))
        }
        
        _ => Err(format!("Unknown intrinsic: {}", name))
    }
}
```

## 📊 **削減効果・期待値**

### **複雑性削減**
| 指標 | 削減前 | 削減後 | 効果 |
|------|--------|--------|------|
| **命令数** | 35個 | 20個 | 43%削減 |
| **コア実装** | 分散 | 統合 | 保守性向上 |
| **バックエンド負荷** | 35×3=105 | 20×3=60 | 43%削減 |

### **拡張性向上**
- **新機能追加**: intrinsicで実験→安定したらcore昇格
- **バックエンド追加**: core 20命令のみ実装すれば基本動作
- **最適化**: intrinsic は必要に応じて最適化・無視可能

## 🎯 **実装戦略・Phase 8.4**

### **段階1: intrinsic基盤（1週間）**
```rust
// 1. Intrinsic命令追加
Intrinsic { dst: Option<ValueId>, name: String, args: Vec<ValueId>, effects: EffectMask }

// 2. intrinsic実行エンジン
impl IntrinsicExecutor {
    fn execute(&self, name: &str, args: &[ValueId]) -> Result<ValueId, String>
}

// 3. 基本intrinsic実装
// print, debug, array_get, array_set
```

### **段階2: 命令統合（1週間）**
```rust
// 1. BinOp統合（Compare削除）
// 2. Load/Store統合（ArrayGet/ArraySet削除）
// 3. 複雑操作のintrinsic移行
```

### **段階3: Bus命令実装（1週間）**
```rust
// 1. Send/Recv命令追加
// 2. Bus-elision基盤
// 3. P2PBox統合
```

### **段階4: 検証・テスト（1週間）**
```rust
// 1. Golden dump更新
// 2. 全バックエンド互換確認  
// 3. 性能回帰チェック
```

## ✅ **Phase 8.4完了基準**

### **技術要件**
- [ ] **命令数20個以内**: ChatGPT5推奨準拠
- [ ] **intrinsic基盤**: 拡張可能な逃がし仕組み
- [ ] **Bus命令**: 分散・非同期一次市民化
- [ ] **全バックエンド動作**: interp/vm/wasm対応

### **品質要件**  
- [ ] **Golden dump更新**: 新命令セットで標準更新
- [ ] **互換テスト通過**: 全バックエンド同一出力
- [ ] **性能維持**: 280倍WASM高速化維持
- [ ] **回帰テストPASS**: 既存機能への影響なし

---

## 📚 **関連ドキュメント**

- **MIR設計思想**: [mir-reference.md](mir-reference.md)
- **互換性契約**: [portability-contract.md](portability-contract.md)
- **テスト仕様**: [golden-dump-testing.md](golden-dump-testing.md)
- **現在実装**: [../../../src/mir/instruction.rs](../../../src/mir/instruction.rs)

---

*最終更新: 2025-08-14 - ChatGPT5「太り過ぎ」対策完全設計*

*MIR最小コア = Nyash「全バックエンド統一」の技術的基盤*