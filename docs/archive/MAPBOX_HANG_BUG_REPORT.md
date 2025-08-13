# MapBox 3引数メソッド呼び出しハングバグ レポート

## 🐛 バグ概要

**問題**: MapBoxオブジェクトを作成した後、3つ以上の引数を持つメソッド呼び出しでプログラムが無限ハングする

**影響範囲**: MapBox作成後の複雑なメソッド呼び出しチェーン全般

## 🔍 根本原因

### 問題のコード
`src/interpreter/methods/collection_methods.rs:131-134`

```rust
// 引数を評価
let mut arg_values = Vec::new();
for arg in arguments {
    arg_values.push(self.execute_expression(arg)?);  // ← 全引数を事前評価
}
```

### 正常動作する他のBox（例：StringBox）
`src/interpreter/methods/basic_methods.rs:27`

```rust
let delimiter_value = self.execute_expression(&arguments[0])?;  // ← 必要時に1つずつ評価
```

## 📊 調査結果

### ハングするケース
```nyash
box MessageHub {
    init { handlers }
    
    setup() {
        me.handlers = new MapBox()  // ← MapBox作成
    }
    
    deliver(messageType, data, from) {
        // 3引数メソッド呼び出し → ハング
        print("Message: " + from + " -> " + messageType + " = " + data)
    }
}
```

### 正常動作するケース
```nyash
// MapBoxを使用しない場合 → 正常
// 2引数以下の場合 → 正常  
// MapBox作成前の3引数呼び出し → 正常
```

## 🛠️ 修正方法

### 推奨修正内容
`src/interpreter/methods/collection_methods.rs:128-145`を以下に変更：

```rust
pub(in crate::interpreter) fn execute_map_method(&mut self, map_box: &MapBox, method: &str, arguments: &[ASTNode]) 
    -> Result<Box<dyn NyashBox>, RuntimeError> {
    
    match method {
        "set" => {
            if arguments.len() != 2 {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("set() expects 2 arguments, got {}", arguments.len()),
                });
            }
            // 必要時評価
            let key_value = self.execute_expression(&arguments[0])?;
            let val_value = self.execute_expression(&arguments[1])?;
            Ok(map_box.set(key_value, val_value))
        }
        "get" => {
            if arguments.len() != 1 {
                return Err(RuntimeError::InvalidOperation {
                    message: format!("get() expects 1 argument, got {}", arguments.len()),
                });
            }
            // 必要時評価
            let key_value = self.execute_expression(&arguments[0])?;
            Ok(map_box.get(key_value))
        }
        // 他のメソッドも同様に修正...
    }
}
```

## ✅ 期待効果

1. **ハング問題完全解決**: MapBox+3引数の組み合わせが正常動作
2. **性能向上**: 不要な引数評価の排除  
3. **一貫性向上**: 他のBox型と同じ評価方式に統一

## 🧪 テスト計画

修正後、以下のテストケースで動作確認：

```nyash
// テスト1: MapBox + 3引数メソッド呼び出し
local hub = new MessageHub()
hub.setup()  // MapBox作成
alice.send("hello", "Hi there!")  // 3引数チェーン → 正常動作期待

// テスト2: 複雑なフィールドアクセス
me.messageHub.deliver(messageType, data, me.nodeId)  // 正常動作期待
```

## 📝 補足

- **緊急度**: 高（基本的なMapBox機能が使用不能）
- **回避策**: 2引数+Messageオブジェクト方式で一時対応可能
- **互換性**: 修正は既存コードに影響なし（内部実装のみ変更）

---

**作成日**: 2025-01-09  
**調査者**: Claude Code Assistant  
**検証環境**: Nyash Rust Implementation