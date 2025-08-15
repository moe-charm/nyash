# 📦 clone_box() vs share_box() 責務分離設計

## 📅 作成日: 2025-08-15
## 🎯 Phase: 9.75D - Box参照管理根本革命

## 🚨 **設計の背景**

Phase 9.75でArc<Mutex> → RwLock変換後、ArrayBoxの状態保持が機能しなくなった根本問題を解決するため、**Box参照セマンティクスの明確な責務分離**を導入する。

### 現在の問題

```nyash
// 期待される動作
arr = new ArrayBox()
arr.push("hello")     // 状態変更
arr.length()          // 1 であるべき

// 実際の動作  
arr.length()          // 0 （状態が失われる）
```

### 根本原因

```rust
// src/interpreter/expressions.rs:108
ASTNode::Variable { name, .. } => {
    let shared_var = self.resolve_variable(name)?;
    Ok((*shared_var).clone_box())  // ← 🚨 毎回新インスタンス作成！
}
```

## 🎯 **設計原則**

### 1. **責務分離の明確化**

**clone_box()**: **値セマンティクス**
- 独立した新しいインスタンスを作成
- 元のオブジェクトと完全に分離
- Rustの`Clone`トレイト慣習に準拠

**share_box()**: **参照セマンティクス**  
- 内部状態を共有する新しいハンドルを作成
- 元のオブジェクトと状態を共有
- 変数アクセス・代入で使用

### 2. **Everything is Box哲学の維持**

両メソッドとも`Box<dyn NyashBox>`を返すことで、統一インターフェースを保持。

```rust
trait NyashBox {
    /// 独立した新しいコピー（ディープコピー）を作成
    fn clone_box(&self) -> Box<dyn NyashBox>;
    
    /// 状態を共有する新しいハンドルを作成
    fn share_box(&self) -> Box<dyn NyashBox>;
}
```

### 3. **型カテゴリ別実装戦略**

#### **ステートフルBox（状態保持が重要）**
- ArrayBox, MapBox, SocketBox, P2PBox, FileBox, StreamBox
- `share_box()`: Arc<RwLock>をクローンして状態共有
- `clone_box()`: ディープコピーで独立インスタンス

#### **ステートレスBox（値のみ保持）**
- StringBox, IntegerBox, BoolBox, MathBox
- `share_box()` = `clone_box()` (同じ実装で問題なし)

## 🔧 **技術実装設計**

### **ArrayBox実装例**

```rust
// 現在の実装（問題のある構造）
pub struct ArrayBox {
    pub items: RwLock<Vec<Box<dyn NyashBox>>>,
    base: BoxBase,
}

// 新しい実装（状態共有対応）
pub struct ArrayBox {
    pub items: Arc<RwLock<Vec<Box<dyn NyashBox>>>>,  // Arc追加
    base: BoxBase,
}

impl NyashBox for ArrayBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        // ディープコピー: 既存のClone実装を使用
        Box::new(self.clone())
    }
    
    fn share_box(&self) -> Box<dyn NyashBox> {
        // 状態共有: Arcをクローンして新しいハンドル作成
        let new_instance = ArrayBox {
            items: Arc::clone(&self.items),  // 🎯 状態共有の核心
            base: BoxBase::new(),            // 新しいID
        };
        Box::new(new_instance)
    }
}

impl Clone for ArrayBox {
    fn clone(&self) -> Self {
        // ディープコピー実装
        let items_guard = self.items.read().unwrap();
        let cloned_items: Vec<Box<dyn NyashBox>> = items_guard.iter()
            .map(|item| item.clone_box())
            .collect();
        
        ArrayBox {
            items: Arc::new(RwLock::new(cloned_items)),
            base: BoxBase::new(),
        }
    }
}
```

### **StringBox実装例（ステートレス）**

```rust
impl NyashBox for StringBox {
    fn clone_box(&self) -> Box<dyn NyashBox> {
        Box::new(self.clone())
    }
    
    // ステートレスなので同じ実装で問題なし
    fn share_box(&self) -> Box<dyn NyashBox> {
        self.clone_box()
    }
}
```

### **インタープリター修正**

```rust
// src/interpreter/expressions.rs
ASTNode::Variable { name, .. } => {
    let shared_var = self.resolve_variable(name)?;
    Ok((*shared_var).share_box())  // 🎯 参照共有使用
}

// 値コピーが必要な場面（例：関数引数渡し）
ASTNode::Assignment { .. } => {
    let value = self.evaluate_expression(right)?;
    // 代入時も参照共有
    self.declare_local_variable(name, value.share_box());
}
```

## 🌍 **マルチバックエンド対応**

### **インタープリター**
- `expressions.rs:108`: `share_box()`使用
- 変数アクセス・代入時の参照管理を統一

### **VM Backend**
- `vm.rs:764`: 配列要素アクセス時の`clone_box()`を検証
- スタック操作での適切なセマンティクス選択

### **WASM Backend**
- WASMメモリ管理は独自実装で影響軽微
- `clone_box()`呼び出し箇所の意図確認・修正

## 📊 **性能への影響**

### **メモリ使用量**
- **改善**: Arc<RwLock>による効率的な状態共有
- **増加**: Arcのオーバーヘッド（8バイト程度）

### **実行性能**
- **現状維持**: WASM 13.5倍、VM 20.4倍高速化を保持
- **改善期待**: 不要なディープコピーの削減

### **ロック競合**
- **リスク**: 複数ハンドルでの同時アクセス
- **対策**: RwLock読み取り中心の設計

## 🧪 **テスト戦略**

### **Unit Test追加**
```rust
#[test]
fn test_arraybox_state_sharing() {
    let arr1 = ArrayBox::new();
    arr1.push(StringBox::new("hello"));
    
    let arr2 = arr1.share_box();
    let arr2_array = arr2.as_any().downcast_ref::<ArrayBox>().unwrap();
    
    // 状態が共有されていることを確認
    assert_eq!(arr2_array.length().as_any().downcast_ref::<IntegerBox>().unwrap().value, 1);
}

#[test]
fn test_arraybox_clone_independence() {
    let arr1 = ArrayBox::new();
    arr1.push(StringBox::new("hello"));
    
    let arr2 = arr1.clone_box();
    let arr2_array = arr2.as_any().downcast_ref::<ArrayBox>().unwrap();
    
    // 独立していることを確認
    arr1.push(StringBox::new("world"));
    assert_eq!(arr2_array.length().as_any().downcast_ref::<IntegerBox>().unwrap().value, 1);
}
```

### **Integration Test**
- 15個のステートフルBox全てでの状態保持テスト
- 3バックエンドでの一貫性テスト
- 既存のNyashプログラムでのリグレッションテスト

## 🔄 **移行時の互換性**

### **後方互換性**
- 既存の`clone_box()`は変更なし
- 新しい`share_box()`を段階的に導入

### **段階的移行**
1. `NyashBox`トレイトに`share_box()`追加
2. 全Box型に仮実装（`clone_box()`と同じ）
3. ステートフルBoxから順次正しい実装に修正
4. インタープリター・VM・WASM修正
5. 全体テスト・リグレッション確認

## 🎯 **設計の利点**

### **明確性**
- 値コピー vs 参照共有の意図が明確
- コードレビュー時の理解容易性

### **保守性**
- 新しいBox型追加時のガイドライン明確
- デバッグ時の状態追跡容易

### **拡張性**
- 将来の最適化（COW等）への発展可能
- 静的解析ツールでの解析支援

### **一貫性**
- 3バックエンド全てで統一されたセマンティクス
- Everything is Box哲学の維持

## 🚨 **設計上の注意点**

### **循環参照リスク**
- Arc<RwLock>による循環参照の可能性
- 弱参照（Weak）の将来的導入検討

### **デッドロックリスク**
- 複数のRwLockを同時取得する場合
- ロック順序の統一ガイドライン必要

### **メモリリーク検証**
- 長時間実行での参照カウント監視
- デバッグ時のメモリ使用量追跡

---

## 📋 **関連ドキュメント**

- [Phase 9.75D 移行計画](phase-9-75d-migration-plan.md)
- [現在の課題](implementation-notes/current-issues.md)
- [Everything is Box 哲学](everything-is-box.md)
- [メモリ管理](memory-management.md)

---

**この設計により、Nyashの状態保持問題を根本解決し、長期的な保守性・拡張性を確保する。**