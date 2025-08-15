# 🚀 Phase 9.75D 段階的移行計画

## 📅 移行期間: 2025-08-15 〜 2025-08-22 (7日間)
## 🎯 目標: clone_box() vs share_box() 責務分離完全実装

## 📋 **移行フェーズ概要**

| フェーズ | 期間 | 内容 | リスク |
|---------|------|------|-------|
| **Phase A** | 1日 | 基盤整備・トレイト拡張 | 低 |
| **Phase B** | 2日 | ArrayBox修正・コアテスト | 中 |
| **Phase C** | 2日 | 主要ステートフルBox展開 | 中 |
| **Phase D** | 1日 | バックエンド横展開 | 高 |
| **Phase E** | 1日 | 残りBox・最終検証 | 低 |

## 🔧 **Phase A: 基盤整備 (Day 1)**

### **目標**: コンパイル可能な基盤構築

### **A1: NyashBoxトレイト拡張**
**ファイル**: `src/boxes/traits.rs`

```rust
// 追加するメソッド
trait NyashBox: Send + Sync + BoxCore + DynClone + Any {
    // ... 既存メソッド ...
    
    /// 状態を共有する新しいハンドルを作成
    /// 変数アクセス・代入時に使用
    fn share_box(&self) -> Box<dyn NyashBox>;
}
```

### **A2: 全Box型への仮実装追加**
**対象ファイル**: 以下の20個のBox実装
```
src/boxes/array/mod.rs       ← 最重要
src/boxes/map_box.rs
src/boxes/string_box.rs
src/boxes/integer_box.rs
src/boxes/bool_box.rs
src/boxes/socket_box.rs
src/boxes/p2p_box.rs
src/boxes/file/mod.rs
src/boxes/stream/mod.rs
src/boxes/http_server_box.rs
src/boxes/simple_intent_box.rs
src/boxes/intent_box.rs
src/boxes/egui_box.rs
src/boxes/random_box.rs
src/boxes/debug_box.rs
src/boxes/future/mod.rs
src/boxes/json/mod.rs
src/boxes/http/mod.rs
src/boxes/regex/mod.rs
src/boxes/buffer/mod.rs
```

**仮実装コード**:
```rust
impl NyashBox for XxxBox {
    // ... 既存メソッド ...
    
    /// 仮実装: clone_boxと同じ（後で正しく修正）
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}
```

### **A3: コンパイル確認**
```bash
cargo check --lib
cargo build --lib -j32
```

**完了条件**: 全ての型チェックエラーが解消され、コンパイル成功

---

## 🎯 **Phase B: ArrayBox修正・コアテスト (Day 2-3)**

### **目標**: 状態保持問題の直接解決

### **B1: ArrayBox構造体修正**
**ファイル**: `src/boxes/array/mod.rs`

```rust
// 現在の構造体
pub struct ArrayBox {
    pub items: RwLock<Vec<Box<dyn NyashBox>>>,
    base: BoxBase,
}

// 修正後の構造体
pub struct ArrayBox {
    pub items: Arc<RwLock<Vec<Box<dyn NyashBox>>>>,  // Arc追加
    base: BoxBase,
}
```

### **B2: ArrayBox::new()修正**
```rust
impl ArrayBox {
    pub fn new() -> Self {
        ArrayBox { 
            items: Arc::new(RwLock::new(Vec::new())),  // Arc::new追加
            base: BoxBase::new(),
        }
    }
    
    pub fn new_with_elements(elements: Vec<Box<dyn NyashBox>>) -> Self {
        ArrayBox { 
            items: Arc::new(RwLock::new(elements)),    // Arc::new追加
            base: BoxBase::new(),
        }
    }
}
```

### **B3: ArrayBox::share_box()正しい実装**
```rust
impl NyashBox for ArrayBox {
    fn share_box(&self) -> Box<dyn NyashBox> {
        let new_instance = ArrayBox {
            items: Arc::clone(&self.items),  // 🎯 状態共有
            base: BoxBase::new(),            // 新しいID
        };
        Box::new(new_instance)
    }
}
```

### **B4: ArrayBox::Clone修正**
```rust
impl Clone for ArrayBox {
    fn clone(&self) -> Self {
        let items_guard = self.items.read().unwrap();
        let cloned_items: Vec<Box<dyn NyashBox>> = items_guard.iter()
            .map(|item| item.clone_box())
            .collect();
        
        ArrayBox {
            items: Arc::new(RwLock::new(cloned_items)),  // 新しいArc
            base: BoxBase::new(),
        }
    }
}
```

### **B5: インタープリター修正**
**ファイル**: `src/interpreter/expressions.rs`

```rust
// Line 108周辺
ASTNode::Variable { name, .. } => {
    let shared_var = self.resolve_variable(name)?;
    Ok((*shared_var).share_box())  // clone_box() → share_box()
}

// 他のclone_box()呼び出し箇所も確認・修正
```

### **B6: 状態保持テスト追加**
**ファイル**: `tests/array_state_sharing_test.rs` (新規作成)

```rust
#[test]
fn test_arraybox_state_sharing_after_push() {
    // 問題再現テスト
    let mut interpreter = Interpreter::new();
    let program = r#"
        arr = new ArrayBox()
        arr.push("hello")
        result = arr.length()
    "#;
    
    let result = interpreter.execute_program(program).unwrap();
    // 1を返すことを確認（0ではない）
    assert_eq!(extract_integer(result), 1);
}

#[test] 
fn test_arraybox_share_vs_clone() {
    let arr1 = ArrayBox::new();
    arr1.push(StringBox::new("hello"));
    
    // share_box: 状態共有
    let arr2 = arr1.share_box();
    let arr2_array = arr2.as_any().downcast_ref::<ArrayBox>().unwrap();
    assert_eq!(arr2_array.len(), 1);
    
    // clone_box: 独立
    let arr3 = arr1.clone_box();
    let arr3_array = arr3.as_any().downcast_ref::<ArrayBox>().unwrap();
    arr1.push(StringBox::new("world"));
    assert_eq!(arr3_array.len(), 1);  // 影響なし
}
```

### **B7: テスト実行・修正**
```bash
cargo test array_state_sharing_test
./target/debug/nyash tests/array_debug.nyash
```

**完了条件**: ArrayBoxの状態保持が正常に動作することを確認

---

## 📈 **Phase C: 主要ステートフルBox展開 (Day 4-5)**

### **目標**: 利用頻度の高いステートフルBox修正

### **C1: 優先順位リスト**
1. **MapBox** - コレクション系、使用頻度大
2. **SocketBox** - 既知の状態保持問題
3. **P2PBox** - 複雑な状態管理
4. **FileBox** - I/O状態管理
5. **StreamBox** - バッファ状態

### **C2: MapBox修正**
**ファイル**: `src/boxes/map_box.rs`

現在の構造確認→Arc追加→share_box()実装→テスト

### **C3: SocketBox修正**
**ファイル**: `src/boxes/socket_box.rs`

既知の状態保持問題（is_server）を根本解決

### **C4: 各Box修正パターン**
```rust
// 共通パターン
pub struct XxxBox {
    pub state_field: Arc<RwLock<StateType>>,  // Arc追加
    base: BoxBase,
}

impl NyashBox for XxxBox {
    fn share_box(&self) -> Box<dyn NyashBox> {
        let new_instance = XxxBox {
            state_field: Arc::clone(&self.state_field),
            base: BoxBase::new(),
        };
        Box::new(new_instance)
    }
}
```

### **C5: 段階的テスト**
各Box修正後に個別テスト実行

**完了条件**: 主要5個のステートフルBoxで状態保持が正常動作

---

## 🌐 **Phase D: バックエンド横展開 (Day 6)**

### **目標**: VM・WASMでの一貫性確保

### **D1: VM Backend確認**
**ファイル**: `src/backend/vm.rs`

```bash
# clone_box()呼び出し箇所を検索
grep -n "clone_box" src/backend/vm.rs
```

**Line 764周辺**: 配列要素アクセスの意図確認
- 値コピーが必要→`clone_box()`維持
- 参照共有が適切→`share_box()`に修正

### **D2: WASM Backend確認**
**ファイル**: `src/backend/wasm/`

WASMの独自メモリ管理での`clone_box()`使用状況確認

### **D3: バックエンド別テスト**
```bash
# VM実行テスト
./target/release/nyash --backend vm tests/array_debug.nyash

# WASM実行テスト  
./target/release/nyash --backend wasm tests/array_debug.nyash
```

**完了条件**: 3バックエンド全てで一貫した動作確認

---

## 🎯 **Phase E: 残りBox・最終検証 (Day 7)**

### **目標**: 完全修正・リグレッション確認

### **E1: 残りステートフルBox修正**
- HTTPServerBox, IntentBox, SimpleIntentBox
- EguiBox, RandomBox, DebugBox
- FutureBox, JSONBox, BufferBox

### **E2: 全体テスト実行**
```bash
# 基本機能テスト
cargo test

# 実用アプリテスト
./target/release/nyash app_dice_rpg.nyash
./target/release/nyash app_statistics.nyash

# 性能テスト
./target/release/nyash --benchmark --iterations 100
```

### **E3: 性能確認**
- WASM: 13.5倍高速化維持
- VM: 20.4倍高速化維持
- インタープリター: 状態保持正常化

### **E4: ドキュメント更新**
- `CURRENT_TASK.md`: Phase 9.75D完了報告
- `clone-box-vs-share-box-design.md`: 実装結果反映

**完了条件**: 全テスト通過・性能維持・ドキュメント完備

---

## 🚨 **リスク管理**

### **Phase A リスク (低)**
- **コンパイルエラー**: 仮実装で対応済み
- **対策**: 段階的なトレイト追加

### **Phase B リスク (中)**
- **ArrayBox破壊**: 既存機能への影響
- **対策**: 詳細なunit test、段階的修正

### **Phase C リスク (中)**
- **複数Box同時破壊**: 相互依存の問題
- **対策**: 1個ずつ修正・テスト

### **Phase D リスク (高)**
- **バックエンド非互換**: VM・WASMでの動作不一致
- **対策**: 各バックエンドでの詳細テスト

### **Phase E リスク (低)**
- **パフォーマンス劣化**: Arc<RwLock>オーバーヘッド
- **対策**: ベンチマークでの詳細測定

---

## 📊 **進捗追跡**

### **Daily Check List**

**Day 1 (Phase A)**:
- [ ] NyashBoxトレイト拡張
- [ ] 20個のBox型仮実装追加
- [ ] cargo check成功

**Day 2-3 (Phase B)**:  
- [ ] ArrayBox構造体修正
- [ ] share_box()正しい実装
- [ ] インタープリター修正
- [ ] 状態保持テスト追加・通過

**Day 4-5 (Phase C)**:
- [ ] MapBox修正完了
- [ ] SocketBox修正完了
- [ ] P2PBox, FileBox, StreamBox修正完了

**Day 6 (Phase D)**:
- [ ] VM Backend確認・修正
- [ ] WASM Backend確認・修正  
- [ ] 3バックエンド一貫性確認

**Day 7 (Phase E)**:
- [ ] 残り10個のBox修正完了
- [ ] 全テスト通過
- [ ] 性能ベンチマーク確認
- [ ] ドキュメント更新

---

## 🎉 **成功条件**

1. **機能正常性**: ArrayBoxの状態保持問題が完全解決
2. **一貫性**: 3バックエンド全てで同じセマンティクス
3. **性能維持**: WASM 13.5倍、VM 20.4倍高速化維持
4. **互換性**: 既存のNyashプログラムが正常動作
5. **拡張性**: 新しいBox型追加時のガイドライン確立

**Phase 9.75D完了により、Nyashの状態管理が根本的に安定化し、Phase 9.5以降の開発が安心して進行可能になる。**