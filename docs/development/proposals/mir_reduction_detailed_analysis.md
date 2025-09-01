# MIR 35→26→15命令削減: 詳細分析・移行戦略

*実装ベース完全マッピング - 2025年8月17日版*  
**注意**: このドキュメントは35→26命令への移行計画を記録しています。現在はさらに15命令に削減済みです。

## 🔍 **過去の実装35命令 vs 26命令仕様の完全マッピング（歴史的記録）**

### **維持する命令 (35命令実装 → 26命令仕様への移行計画）**

| 現在実装 | 26命令仕様 | 効果 | 変更 | 
|----------|------------|------|------|
| `Const` | `Const` | pure | ✅ 維持 |
| `BinOp` | `BinOp` | pure | ✅ 維持 |
| `Compare` | `Compare` | pure | ✅ 維持 |
| `Branch` | `Branch` | control | ✅ 維持 |
| `Jump` | `Jump` | control | ✅ 維持 |
| `Phi` | `Phi` | pure | ✅ 維持 |
| `Call` | `Call` | context | ✅ 維持 |
| `Return` | `Return` | control | ✅ 維持 |
| `NewBox` | `NewBox` | mut | ✅ 維持 |
| `BoxCall` | `BoxCall` | context | ✅ 維持 |
| `ExternCall` | `ExternCall` | context | ✅ 維持 |
| `Safepoint` | `Safepoint` | io | ✅ 維持 |
| `RefGet` | `RefGet` | pure | ✅ 維持 |
| `RefSet` | `RefSet` | mut | ✅ 維持 |
| `WeakNew` | `WeakNew` | pure | ✅ 維持 |
| `WeakLoad` | `WeakLoad` | pure | ✅ 維持 |

**小計**: 16命令維持

### **削除する命令 (17命令)**

#### **グループ1: BinOp統合 (1命令)**

| 削除命令 | 置換方法 | 実装例 |
|----------|----------|--------|
| `UnaryOp` | `BinOp`統合 | `not %a` → `%a xor true`<br>`neg %a` → `0 sub %a` |

#### **グループ2: BoxField操作統合 (4命令)**

| 削除命令 | 置換方法 | 実装例 |
|----------|----------|--------|
| `Load` | `BoxFieldLoad` | `load %ptr` → `%ptr.value` |
| `Store` | `BoxFieldStore` | `store %val -> %ptr` → `%ptr.value = %val` |
| `ArrayGet` | `BoxFieldLoad` | `%arr[%idx]` → `%arr.elements[%idx]` |
| `ArraySet` | `BoxFieldStore` | `%arr[%idx] = %val` → `%arr.elements[%idx] = %val` |

#### **グループ3: intrinsic化 (6命令)**

| 削除命令 | intrinsic名 | 実装例 |
|----------|-------------|--------|
| `Print` | `@print` | `print %val` → `call @print, %val` |
| `Debug` | `@debug` | `debug %val "msg"` → `call @debug, %val, "msg"` |
| `TypeCheck` | `@type_check` | `type_check %val "Type"` → `call @type_check, %val, "Type"` |
| `Cast` | `@cast` | `cast %val Type` → `call @cast, %val, Type` |
| `Throw` | `@throw` | `throw %exc` → `call @throw, %exc` |
| `Catch` | `@catch` | `catch Type -> %bb` → `call @catch, Type, %bb` |

#### **グループ4: 完全削除 (3命令)**

| 削除命令 | 削除理由 | 代替方法 |
|----------|----------|----------|
| `Copy` | 最適化パス専用 | 最適化段階でのみ使用 |
| `Nop` | 不要 | 削除（プレースホルダー不要） |
| `RefNew` | 冗長 | `RefGet`で代用可能 |

#### **グループ5: 統合・置換 (3命令)**

| 削除命令 | 統合先 | 実装例 |
|----------|--------|--------|
| `BarrierRead` | `AtomicFence` | `barrier_read %ptr` → `atomic_fence acquire` |
| `BarrierWrite` | `AtomicFence` | `barrier_write %ptr` → `atomic_fence release` |
| `FutureNew` | `NewBox + BoxCall` | `future_new %val` → `%f = new_box "Future"(%val)` |
| `FutureSet` | `BoxCall` | `future_set %f = %val` → `%f.set(%val)` |
| `Await` | `BoxCall` | `await %f` → `%f.await()` |

### **追加する命令 (10命令)**

| 新命令 | 効果 | 目的 | 実装必要度 |
|--------|------|------|------------|
| `BoxFieldLoad` | pure | Everything is Box核心 | 🔥 Critical |
| `BoxFieldStore` | mut | Everything is Box核心 | 🔥 Critical |
| `WeakCheck` | pure | weak参照完全対応 | ⚡ High |
| `Send` | io | Bus操作一次市民化 | ⚡ High |
| `Recv` | io | Bus操作一次市民化 | ⚡ High |
| `TailCall` | control | JIT最適化基盤 | 📝 Medium |
| `Adopt` | mut | 所有権移管明示 | 📝 Medium |
| `Release` | mut | 所有権移管明示 | 📝 Medium |
| `MemCopy` | mut | 最適化基盤 | 📝 Medium |
| `AtomicFence` | io | 並行制御統一 | 📝 Medium |

## 🛠️ **具体的実装戦略**

### **Phase 1: 新命令実装**

#### **BoxFieldLoad/BoxFieldStore実装**
```rust
// src/mir/instruction.rs
pub enum MirInstruction {
    // 新規追加
    BoxFieldLoad {
        dst: ValueId,
        box_val: ValueId,
        field: String,
    },
    BoxFieldStore {
        box_val: ValueId,
        field: String,
        value: ValueId,
    },
    // ...
}
```

#### **WeakCheck実装**
```rust
WeakCheck {
    dst: ValueId,
    weak_ref: ValueId,
}
```

#### **Send/Recv実装**
```rust
Send {
    data: ValueId,
    target: ValueId,
},
Recv {
    dst: ValueId,
    source: ValueId,
},
```

### **Phase 2: intrinsic関数システム実装**

#### **intrinsic レジストリ**
```rust
// src/interpreter/intrinsics.rs
pub struct IntrinsicRegistry {
    functions: HashMap<String, IntrinsicFunction>,
}

impl IntrinsicRegistry {
    pub fn new() -> Self {
        let mut registry = Self { functions: HashMap::new() };
        registry.register("@print", intrinsic_print);
        registry.register("@debug", intrinsic_debug);
        registry.register("@type_check", intrinsic_type_check);
        registry.register("@cast", intrinsic_cast);
        registry.register("@array_get", intrinsic_array_get);
        registry.register("@array_set", intrinsic_array_set);
        registry
    }
}
```

#### **intrinsic関数実装例**
```rust
fn intrinsic_print(args: &[Value]) -> Result<Value, RuntimeError> {
    println!("{}", args[0]);
    Ok(Value::Void)
}

fn intrinsic_array_get(args: &[Value]) -> Result<Value, RuntimeError> {
    let array = args[0].as_array_box()?;
    let index = args[1].as_integer()?;
    array.get_element(index as usize)
}

fn intrinsic_array_set(args: &[Value]) -> Result<Value, RuntimeError> {
    let array = args[0].as_array_box_mut()?;
    let index = args[1].as_integer()?;
    let value = args[2].clone();
    array.set_element(index as usize, value)
}
```

### **Phase 3: AST→MIR生成更新**

#### **Load/Store → BoxFieldLoad/BoxFieldStore変換**
```rust
// src/mir/builder.rs
impl MirBuilder {
    fn visit_field_access(&mut self, node: &FieldAccessNode) -> Result<ValueId, BuildError> {
        let box_val = self.visit_expression(&node.object)?;
        let dst = self.new_temp_var();
        
        // 旧: Load命令生成
        // self.emit(MirInstruction::Load { dst, ptr: box_val });
        
        // 新: BoxFieldLoad命令生成
        self.emit(MirInstruction::BoxFieldLoad {
            dst,
            box_val,
            field: node.field.clone(),
        });
        
        Ok(dst)
    }
    
    fn visit_field_assignment(&mut self, node: &FieldAssignmentNode) -> Result<(), BuildError> {
        let box_val = self.visit_expression(&node.object)?;
        let value = self.visit_expression(&node.value)?;
        
        // 旧: Store命令生成
        // self.emit(MirInstruction::Store { value, ptr: box_val });
        
        // 新: BoxFieldStore命令生成
        self.emit(MirInstruction::BoxFieldStore {
            box_val,
            field: node.field.clone(),
            value,
        });
        
        Ok(())
    }
}
```

#### **配列操作 → BoxField + intrinsic変換**
```rust
fn visit_array_access(&mut self, node: &ArrayAccessNode) -> Result<ValueId, BuildError> {
    let array = self.visit_expression(&node.array)?;
    let index = self.visit_expression(&node.index)?;
    let dst = self.new_temp_var();
    
    // intrinsic化
    self.emit(MirInstruction::Call {
        dst: Some(dst),
        func: self.get_intrinsic_id("@array_get"),
        args: vec![array, index],
        effects: EffectMask::PURE,
    });
    
    Ok(dst)
}
```

### **Phase 4: バックエンド対応**

#### **Interpreter実装**
```rust
// src/backend/interpreter.rs
impl Interpreter {
    fn execute_box_field_load(&mut self, dst: ValueId, box_val: ValueId, field: &str) -> Result<(), RuntimeError> {
        let box_obj = self.get_value(box_val)?;
        let field_value = box_obj.get_field(field)?;
        self.set_value(dst, field_value);
        Ok(())
    }
    
    fn execute_box_field_store(&mut self, box_val: ValueId, field: &str, value: ValueId) -> Result<(), RuntimeError> {
        let mut box_obj = self.get_value_mut(box_val)?;
        let field_value = self.get_value(value)?;
        box_obj.set_field(field, field_value)?;
        Ok(())
    }
}
```

#### **VM実装**
```rust
// src/backend/vm.rs
impl VM {
    fn exec_box_field_load(&mut self, dst: RegId, box_val: RegId, field_id: FieldId) -> VMResult<()> {
        let box_ptr = self.registers[box_val as usize];
        let field_value = unsafe { 
            self.load_field(box_ptr, field_id)
        };
        self.registers[dst as usize] = field_value;
        Ok(())
    }
}
```

#### **WASM実装**
```rust
// src/backend/wasm/codegen.rs
impl WasmCodegen {
    fn generate_box_field_load(&mut self, dst: ValueId, box_val: ValueId, field: &str) -> Result<(), CodegenError> {
        let box_addr = self.get_value_address(box_val)?;
        let field_offset = self.get_field_offset(field)?;
        
        // WASM: i32.load offset=field_offset
        self.emit_wasm(&format!("i32.load offset={}", field_offset));
        self.set_value_register(dst);
        Ok(())
    }
}
```

## 📊 **移行スケジュール詳細**

### **Week 1: 基盤実装 (8/18-8/24)**
- [ ] 新命令構造体定義
- [ ] intrinsicレジストリ実装
- [ ] パーサー拡張（新旧両対応）

### **Week 2: フロントエンド移行 (8/25-8/31)**
- [ ] AST→MIR変換更新
- [ ] 配列操作intrinsic化
- [ ] Load/Store→BoxField変換

### **Week 3: 最適化パス移行 (9/1-9/7)**
- [ ] Effect分類実装
- [ ] 所有権森検証
- [ ] BoxFieldLoad/Store最適化

### **Week 4: バックエンド移行 (9/8-9/14)**
- [ ] Interpreter新命令実装
- [ ] VM新命令実装
- [ ] WASM新命令実装

### **Week 5: クリーンアップ (9/15-9/21)**
- [ ] 旧命令完全削除
- [ ] テスト更新
- [ ] ドキュメント整備

## 🧪 **テスト・検証計画**

### **段階的テスト**
```bash
# Week 1終了時
./scripts/test_mir_parsing_26.sh

# Week 2終了時  
./scripts/test_frontend_migration.sh

# Week 3終了時
./scripts/test_optimization_passes.sh

# Week 4終了時
./scripts/test_all_backends.sh

# Week 5終了時
./scripts/test_golden_mir_final.sh
```

### **性能回帰テスト**
```bash
# 削減前後性能比較
./scripts/benchmark_mir_reduction.sh
```

## 🎯 **リスク対策**

### **高リスク箇所**
1. **配列操作intrinsic化**: パフォーマンス影響大
2. **BoxField統合**: Box型システムとの整合性
3. **Effect分類変更**: 最適化ロジック全面見直し

### **対策**
- **プロトタイプ実装**: 高リスク箇所の事前検証
- **性能測定**: 各段階での性能チェック
- **ロールバック**: 問題発生時の迅速復旧

---

**分析完了**: 2025年8月17日  
**実装開始**: 2025年8月18日  
**完了予定**: 2025年9月21日