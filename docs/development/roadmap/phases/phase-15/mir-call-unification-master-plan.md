# MIR Call命令統一 完全移行戦略

## Executive Summary

ChatGPT5 Pro A++設計によるMIR Call系命令の完全統一化プロジェクト。6種類の異なるCall命令を1つのMirCallに統一し、4つの実行器すべてで統一処理を実現。Phase 15セルフホスティングの重要な柱として、**5,200行（26%）のコード削減**を達成する。

## 現在の状況分析

### ✅ 完了済み項目（Phase 1-2）
- **MIR統一定義**: `src/mir/definitions/call_unified.rs`（297行）完成
- **Callee enum拡張**: Constructor/Closureバリアント追加済み
- **統一メソッド実装**: `emit_unified_call()`と便利メソッド3種実装済み
- **環境変数制御**: `NYASH_MIR_UNIFIED_CALL=1`で切り替え可能
- **Math関数で実使用**: builder_calls.rs:340-347で統一Call使用中

### 🔍 現在の実装状況

#### 1. Call系命令の処理箇所（4つの実行器）

| 実行器 | ファイル | 行数 | 実装状況 |
|-------|----------|------|----------|
| **MIR生成** | `src/mir/builder/*.rs` | 3,656行 | ✅ 統一Call部分実装済み |
| **VM Interpreter** | `src/backend/mir_interpreter.rs` | 712行 | ✅ Callee型対応済み |
| **Python LLVM** | `src/llvm_py/instructions/` | 804行 | ❌ 6種類別々実装 |
| **mini-vm (Nyash)** | `apps/selfhost/vm/` | 2,200行 | 🔄 新規実装（最初から統一対応） |

#### 2. 6種類のCall系命令の分布

```rust
// 現在の6種類
MirInstruction::Call { .. }           // 汎用関数呼び出し
MirInstruction::BoxCall { .. }        // Boxメソッド呼び出し
MirInstruction::PluginInvoke { .. }   // プラグイン呼び出し
MirInstruction::ExternCall { .. }     // C ABI外部呼び出し
MirInstruction::NewBox { .. }         // Box コンストラクタ
MirInstruction::NewClosure { .. }     // クロージャ生成

// 統一後の1種類
MirInstruction::MirCall(MirCall)      // すべて統一
```

## 移行戦略詳細

### Phase 3: MIR Builder完全統一（1週間）

#### 3.1 高頻度使用箇所の統一（2日）

**対象箇所**：
- `build_indirect_call_expression`（exprs_call.rs:6） - 旧Call生成箇所
- `emit_box_or_plugin_call`（utils.rs:75） - BoxCall/PluginInvoke生成
- print等の基本関数（builder_calls.rs） - 現在callee: None

**実装内容**：
```rust
// Before: 旧Call生成
self.emit_instruction(MirInstruction::Call {
    dst: Some(dst),
    func: func_val,
    callee: None,  // ← 旧式
    args,
    effects: EffectMask::IO,
});

// After: 統一Call生成
self.emit_unified_call(
    Some(dst),
    CallTarget::Value(func_val),
    args
)?;
```

**期待成果**：
- MIR生成器の統一率: 45% → 80%
- コード削減: 400行

#### 3.2 残存emit_*_call系メソッド統一（2日）

**統一対象**：
- `emit_box_or_plugin_call` → `emit_method_call`
- `emit_external_call` → `emit_extern_call`
- `emit_new_box` → `emit_constructor_call`

**実装戦略**：
1. 各メソッドを統一Call使用に書き換え
2. 既存呼び出し箇所を段階的に移行
3. `NYASH_MIR_UNIFIED_CALL=1`で切り替えテスト

#### 3.3 環境変数制御からデフォルト化（3日）

**段階的デフォルト化**：
1. テストスイート全体で統一Call有効化
2. スモークテスト + CI通過確認
3. 環境変数をデフォルトONに変更
4. 旧実装コードの削除

### Phase 4: Python LLVM統一（1.5週間）

#### 4.1 統一MirCall処理の実装（4日）

**新ファイル作成**：
```python
# src/llvm_py/instructions/mir_call.py
def lower_mir_call(builder, module, mir_call, dst_vid, vmap, resolver):
    """統一MirCall処理 - 6種類の命令を1箇所で処理"""
    match mir_call.callee:
        case Global(name):
            # 旧call.pyロジック流用
        case Method(box_name, method, receiver):
            # 旧boxcall.pyロジック流用
        case Extern(name):
            # 旧externcall.pyロジック流用
        case Constructor(box_type):
            # 旧newbox.pyロジック流用
        case Closure(params, captures):
            # 新規実装
        case Value(vid):
            # 動的呼び出し実装
```

#### 4.2 instruction_lower.py統合（2日）

**dispatch統一**：
```python
# Before: 6つの分岐
elif op == "call":
    lower_call(...)
elif op == "boxcall":
    lower_boxcall(...)
elif op == "externcall":
    lower_externcall(...)
# ... 他3種類

# After: 1つの統一分岐
elif op == "mir_call":
    lower_mir_call(owner, builder, inst["mir_call"], inst.get("dst"), func)
```

#### 4.3 既存ファイル削除とリファクタ（4日）

**削除対象**：
- `instructions/call.py` (172行)
- `instructions/boxcall.py` (425行)
- `instructions/externcall.py` (207行)
- **合計**: 804行削除

**期待成果**：
- Python LLVM実装: 804行 → 300行（63%削減）
- 処理統一による最適化機会増加

### Phase 5: VM Interpreter最適化（1週間）

#### 5.1 統一execute_mir_call実装（3日）

現在のVMはCallee型対応済みだが、さらなる統一と最適化を実施：

```rust
// 統一実行器
fn execute_mir_call(&mut self, mir_call: &MirCall) -> Result<VMValue, VMError> {
    match &mir_call.callee {
        Callee::Global(name) => self.execute_global_function(name, &mir_call.args),
        Callee::Method { receiver: Some(recv), method, .. } => {
            let recv_val = self.reg_load(*recv)?;
            self.execute_method_call(&recv_val, method, &mir_call.args)
        },
        Callee::Constructor { box_type } => {
            self.execute_constructor_call(box_type, &mir_call.args)
        },
        Callee::Extern(name) => self.execute_extern_call(name, &mir_call.args),
        // ... 他のパターン
    }
}
```

#### 5.2 既存分岐削除（2日）

**削除対象**：
- `execute_callee_call`と`execute_legacy_call`の分岐
- 6種類のCall命令別処理ロジック

#### 5.3 エラーハンドリング改善（2日）

統一された呼び出し処理により、エラー処理も統一化。

### Phase 6: mini-vm統一対応（5日）

mini-vmは新規実装のため、最初から統一MirCall対応で実装：

```nyash
// apps/selfhost/vm/call_executor.nyash
static box CallExecutor {
    execute(mir_call: MirCallBox) {
        local callee_type = mir_call.getCalleeType()
        match callee_type {
            "Global" => me.executeGlobal(mir_call)
            "Method" => me.executeMethod(mir_call)
            "Constructor" => me.executeConstructor(mir_call)
            "Extern" => me.executeExtern(mir_call)
            _ => panic("Unknown callee type: " + callee_type)
        }
    }
}
```

## コード削減見込み

### 削減内訳

| フェーズ | 対象領域 | 現在行数 | 削減行数 | 削減率 |
|---------|----------|----------|----------|--------|
| **Phase 3** | MIR Builder | 3,656行 | 800行 | 22% |
| **Phase 4** | Python LLVM | 804行 | 504行 | 63% |
| **Phase 5** | VM Interpreter | 712行 | 200行 | 28% |
| **Phase 6** | mini-vm | 2,200行 | 400行* | 18% |
| **共通** | 統一定義活用 | - | +200行 | - |

**総計**: **5,372行 → 4,472行** = **900行削減（17%減）**

\* mini-vmは最初から統一実装のため、削減ではなく最適実装

### Phase 15目標への寄与

- **Phase 15目標**: 80k行 → 20k行（75%削減）
- **MirCall統一寄与**: 900行削減 = **全体の4.5%**
- **複合効果**: 統一による他システムへの波及効果で追加2-3%

## スケジュール

### ✅ 完了済み（2025-09-24）
- ✅ Phase 3.1: build_indirect_call_expression統一移行（完了）
- ✅ Phase 3.2: print等基本関数のCallee型適用（完了）

### 🔧 進行中（今週）
- 🔄 Phase 3.3: emit_box_or_plugin_call統一化（1-2日）

### 📅 実装優先順位（戦略的判断済み）

#### **第1優先: Python LLVM（来週）** - 最大削減効果
- **Phase 4.1**: Python LLVM dispatch統一（2-3日）
  - `src/llvm_py/llvm_builder.py`の6種類分岐→1つに統一
  - 環境変数で段階移行
- **Phase 4.2**: Python LLVM統一処理実装（3-4日）
  - 804行→300行（**63%削減**）
  - 統一dispatch_unified_call()実装

#### **第2優先: PyVM/VM（再来週）** - 実行器中核
- **Phase 5**: VM Interpreter統一execute実装（4-5日）
  - `src/backend/mir_interpreter.rs`（Rust VM）
  - `pyvm/vm.py`（Python VM）
  - 712行→512行（28%削減）

#### **第3優先: mini-vm（その後）** - 新規実装
- **Phase 6**: mini-vm統一Call実装（5日）
  - 最初から統一実装なので削減ではなく最適実装
  - セルフホスティング検証

### 長期（完成後）
- 📅 Phase 7: 旧命令完全削除（3日）
- 📅 Phase 8: 最適化とクリーンアップ（1週間）

## リスク管理

### 🚨 主要リスク

1. **パフォーマンス影響**
   - **対策**: ベンチマーク測定、最適化パス追加
   - **閾値**: 5%以上の性能低下で要改善

2. **既存コードの破壊的変更**
   - **対策**: 段階的移行、環境変数による切り替え
   - **ロールバック**: 旧実装を環境変数で復活可能

3. **テストの複雑性**
   - **対策**: 統一後に統合テスト追加
   - **CI継続**: 各フェーズでCI通過を確認

### 🎯 成功指標

#### Phase別チェックポイント

| Phase | 成功指標 |
|-------|----------|
| **Phase 3** | MIR生成器でemit_unified_call使用率80%以上 |
| **Phase 4** | Python LLVM実装の命令数6→1への削減完了 |
| **Phase 5** | VM実行器のCall統一処理性能5%以内 |
| **Phase 6** | mini-vm統一実装完了、セルフホスティング可能 |

#### パフォーマンス目標

- **コンパイル時間**: 現状維持（±5%以内）
- **実行時間**: 現状維持または向上（統一最適化効果）
- **メモリ使用量**: 10%以上削減（重複コード除去効果）

## 実装サンプルコード

### 1. MIR Builder統一

```rust
// src/mir/builder/builder_calls.rs
impl MirBuilder {
    /// 段階的移行メソッド - 旧emit_box_or_plugin_callを置き換え
    pub fn emit_method_call_unified(
        &mut self,
        dst: Option<ValueId>,
        receiver: ValueId,
        method: String,
        args: Vec<ValueId>,
    ) -> Result<Option<ValueId>, String> {
        // 環境変数チェック
        if std::env::var("NYASH_MIR_UNIFIED_CALL").unwrap_or("0") == "1" {
            // 統一Call使用
            self.emit_unified_call(
                dst,
                CallTarget::Method {
                    receiver,
                    method,
                    box_name: "InferredBox".to_string(), // 型推論で解決
                },
                args
            )
        } else {
            // 従来のBoxCall使用
            self.emit_box_or_plugin_call(dst, receiver, &method, None, args, EffectMask::IO)
        }
    }
}
```

### 2. Python LLVM統一

```python
# src/llvm_py/instructions/mir_call.py
def lower_mir_call(owner, builder, mir_call_dict, dst_vid, func):
    """統一MirCall処理 - ChatGPT5 Pro A++設計の実装"""
    callee = mir_call_dict["callee"]
    args = mir_call_dict["args"]

    match callee["type"]:
        case "Global":
            # 旧call.pyロジックを統合
            return _lower_global_call(
                builder, owner.module, callee["name"],
                args, dst_vid, owner.vmap, owner.resolver
            )
        case "Method":
            # 旧boxcall.pyロジックを統合
            return _lower_method_call(
                builder, owner.module, callee["receiver"],
                callee["method"], args, dst_vid, owner.vmap
            )
        case "Constructor":
            # NewBox相当の実装
            return _lower_constructor_call(
                builder, owner.module, callee["box_type"],
                args, dst_vid, owner.vmap
            )
        case _:
            raise NotImplementedError(f"Callee type {callee['type']} not supported")
```

### 3. VM Interpreter最適化

```rust
// src/backend/mir_interpreter.rs
impl MirInterpreter {
    fn execute_mir_call_unified(&mut self, mir_call: &MirCall) -> Result<VMValue, VMError> {
        // エフェクト検証
        if mir_call.flags.no_return {
            // no_return系の処理（panic, exit等）
            return self.execute_no_return_call(&mir_call.callee, &mir_call.args);
        }

        // 型安全な呼び出し
        let result = match &mir_call.callee {
            Callee::Global(name) => {
                self.execute_builtin_function(name, &mir_call.args)?
            }
            Callee::Method { receiver: Some(recv), method, box_name } => {
                let recv_val = self.reg_load(*recv)?;
                self.execute_typed_method(&recv_val, box_name, method, &mir_call.args)?
            }
            Callee::Constructor { box_type } => {
                self.execute_box_constructor(box_type, &mir_call.args)?
            }
            Callee::Extern(name) => {
                self.execute_c_abi_call(name, &mir_call.args)?
            }
            _ => return Err(VMError::InvalidInstruction("Unsupported callee type".into())),
        };

        // 結果格納
        if let Some(dst) = mir_call.dst {
            self.regs.insert(dst, result.clone());
        }

        Ok(result)
    }
}
```

## 新しいTodoリスト案

基づいて実装するべきタスク（優先順位付き）：

### 🔥 Urgent（1週間以内）

1. **build_indirect_call_expression統一移行**
   - `exprs_call.rs:6`を`emit_unified_call`使用に変更
   - テスト: 関数呼び出し動作確認

2. **print等基本関数のCallee型適用**
   - `builder_calls.rs`でcallee: None箇所を修正
   - Global("print")等の明示的Callee指定

3. **emit_box_or_plugin_call統一化**
   - `utils.rs:75`の処理を`emit_method_call`に集約
   - BoxCall/PluginInvokeの生成統一

### ⚡ High（2-3週間以内）

4. **Python LLVM dispatch統一**
   - `instruction_lower.py`で6分岐→1分岐に統合
   - `mir_call.py`新規作成

5. **Python LLVM統一処理実装**
   - call/boxcall/externcallロジックをmix
   - 804行→300行の大幅削減

6. **VM Interpreter統一execute実装**
   - `execute_callee_call`と`execute_legacy_call`統合
   - エラーハンドリング改善

### 📅 Medium（1ヶ月以内）

7. **mini-vm統一Call実装**
   - `apps/selfhost/vm/call_executor.nyash`作成
   - Nyashでの統一処理実装

8. **環境変数デフォルト化**
   - `NYASH_MIR_UNIFIED_CALL=1`をデフォルトに
   - CI/テスト全体での統一Call使用

9. **旧実装コード削除**
   - Python LLVM旧ファイル3種削除
   - MIR Builder旧メソッド削除

### 🧹 Low（継続的）

10. **パフォーマンス測定とベンチマーク**
    - 統一Call前後の性能比較
    - 最適化機会の特定

11. **統合テスト追加**
    - 4実行器での統一動作テスト
    - エラーケース検証

12. **ドキュメント更新**
    - MIR仕様書の統一Call反映
    - 開発者ガイド更新

## 期待される成果

### 📊 定量的成果

- **コード削減**: 900行（17%）
- **命令種類**: 6種類 → 1種類（83%削減）
- **メンテナンス負荷**: 4箇所 × 6種類 = 24パターン → 4箇所 × 1種類 = 4パターン（83%削減）

### 🚀 定性的成果

- **開発体験向上**: 新Call実装時の工数大幅削減
- **バグ削減**: 統一処理によるエッジケース減少
- **最適化機会**: 統一されたCall処理による最適化効果
- **AI協働開発**: ChatGPT5 Pro設計の実証完了

### 🎯 Phase 15への戦略的寄与

MirCall統一は単なるコード削減を超えて：

1. **セルフホスティング加速**: mini-vmの統一実装による開発効率化
2. **AI設計実証**: ChatGPT5 Pro A++設計の実用性証明
3. **拡張性確保**: 新しいCall種類の追加が極めて容易に
4. **保守性向上**: 4実行器×1統一処理による保守負荷激減

この包括的移行により、Phase 15の80k→20k行革命において重要な役割を果たし、Nyashセルフホスティングの技術的基盤を確立する。