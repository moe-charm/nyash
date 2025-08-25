# MIR/VMでの統一Box処理の簡素化
Status: Pending
Created: 2025-08-25
Priority: High
Related-Code: src/backend/vm_instructions.rs::execute_boxcall()

## 現状の問題

VMのBoxCall実装で、InstanceBox（ユーザー定義Box）だけ特別扱いされている：

```rust
// 現在の実装
if let Some(inst) = arc_box.as_any().downcast_ref::<InstanceBox>() {
    // ユーザー定義Boxは関数呼び出しに変換
    let func_name = format!("{}.{}/{}", inst.class_name, method, args.len());
    // ...
} else {
    // ビルトイン/プラグインBoxは直接メソッド呼び出し
    self.call_box_method(cloned_box, method, nyash_args)?
}
```

## 理想的な統一実装

すべてのBoxが統一的にデリゲーションできるなら、実装も統一できるはず：

### 案1: すべてのBoxでメソッドディスパッチ統一
```rust
// すべてのBoxで同じインターフェース
pub trait UnifiedBox: NyashBox {
    fn dispatch_method(&self, method: &str, args: Vec<Box<dyn NyashBox>>) -> Result<Box<dyn NyashBox>, String>;
}

// VMでの統一処理
let result = match &recv {
    VMValue::BoxRef(arc_box) => {
        arc_box.dispatch_method(method, nyash_args)?
    }
    _ => {
        let box_value = recv.to_nyash_box();
        box_value.dispatch_method(method, nyash_args)?
    }
};
```

### 案2: メソッドレジストリの統合
```rust
struct UnifiedMethodRegistry {
    // Box型名 → メソッド名 → 実装
    methods: HashMap<String, HashMap<String, MethodImpl>>,
}

enum MethodImpl {
    Native(fn(&dyn NyashBox, Vec<Box<dyn NyashBox>>) -> Result<Box<dyn NyashBox>, String>),
    MirFunction(String), // MIR関数名
    Plugin(PluginMethodId),
}
```

## メリット

1. **コードの簡素化**
   - Box型による分岐が不要
   - 統一的なメソッドディスパッチ

2. **保守性の向上**
   - 新しいBox型追加が容易
   - ビルトイン/プラグイン/ユーザー定義の区別が不要

3. **パフォーマンス**
   - 統一的な最適化が可能
   - メソッドキャッシュの効率化

## 実装タイミング
- [ ] Phase 10での大規模リファクタリング時
- [ ] 新しいBox型追加時
- [ ] パフォーマンス最適化時

## 関連する改善案
- InstanceBoxの廃止（すべてのBoxを統一表現）
- MIRでのBox型情報の統一
- プラグインシステムの簡素化