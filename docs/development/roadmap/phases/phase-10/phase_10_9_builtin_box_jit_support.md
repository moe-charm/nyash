# Phase 10.9 - ビルトインBox JITサポート

## 🎯 目的
ビルトインBoxをJITで使えるようにし、Python統合（Phase 10.1）への道を開く。

## 📦 対象Box（優先順位順）

### 第1段階：読み取り専用メソッド
```nyash
// StringBox
str.length()      // → i64
str.isEmpty()     // → bool
str.charAt(idx)   // → String（新Box生成）

// ArrayBox  
arr.length()      // → i64
arr.isEmpty()     // → bool
arr.get(idx)      // → Box（既存参照）

// IntegerBox/FloatBox
int.toFloat()     // → f64
float.toInt()     // → i64
```

### 第2段階：Box生成
```nyash
// new演算子のJIT化
new StringBox("hello")     // → Handle
new IntegerBox(42)         // → Handle（または直接i64）
new ArrayBox()             // → Handle
```

### 第3段階：書き込みメソッド
```nyash
// 状態変更を伴う操作
arr.push(item)             // Mutex操作必要
arr.set(idx, value)        // 境界チェック必要
map.set(key, value)        // ハッシュ操作
```

## 🔧 実装戦略

### 1. HandleRegistry活用
```rust
// 既存のHandleRegistry（80%実装済み）を拡張
pub fn jit_get_box_method(handle: u64, method: &str) -> Option<MethodPtr> {
    // ハンドル → Box → メソッドポインタ
}
```

### 2. HostCall拡張
```rust
// 現在の限定的なHostCallを段階的に拡張
enum HostCallKind {
    // 既存
    ArrayIsEmpty,
    StringLength,
    
    // Phase 10.9で追加
    StringIsEmpty,
    StringCharAt,
    ArrayGet,
    IntToFloat,
    FloatToInt,
    
    // new演算子サポート
    NewStringBox,
    NewIntegerBox,
    NewArrayBox,
}
```

### 3. 型安全性の確保
```rust
// JIT時の型チェック
match method {
    "length" => {
        // StringBox/ArrayBoxのみ許可
        verify_box_type(handle, &[BoxType::String, BoxType::Array])?
    }
    "isEmpty" => {
        // より多くのBoxで使用可能
        verify_box_type(handle, &[BoxType::String, BoxType::Array, BoxType::Map])?
    }
}
```

## 📊 成功指標

### 機能面
- [ ] StringBox.length() がJITで実行可能
- [ ] ArrayBox.isEmpty() がJITで実行可能
- [ ] new StringBox() がJITで生成可能
- [ ] 型チェックが正しく動作

### 性能面
- [ ] HostCall経由でも10倍以上高速化
- [ ] Handle解決のオーバーヘッド最小化
- [ ] Mutex競合の回避（読み取り専用）

### Python統合への貢献
- [ ] PythonParserBoxの基本メソッドが使用可能
- [ ] MirBuilderBoxへのデータ受け渡し可能
- [ ] 最小限のPython→Nyash変換が動作

## 🚧 技術的課題

### 1. Arc<Mutex>パターンとの整合性
```rust
// 読み取り専用でもMutexロックが必要？
// → 読み取り専用APIを別途用意？
```

### 2. Box生成時のメモリ管理
```rust
// JIT内でのArc生成
// → HandleRegistryで一元管理
```

### 3. エラーハンドリング
```rust
// パニックしない設計
// → Result型での丁寧なエラー伝播
```

## 📈 実装ロードマップ

### Week 1：基盤整備
- HandleRegistry拡張
- HostCallインターフェース設計
- 型チェック機構

### Week 2：読み取りメソッド実装
- StringBox：length, isEmpty, charAt
- ArrayBox：length, isEmpty, get
- 数値変換：toInt, toFloat

### Week 3：Box生成サポート
- new演算子のMIR→JIT変換
- コンストラクタ呼び出し
- HandleRegistry登録

### Week 4：テストと最適化
- E2Eテストスイート
- パフォーマンス測定
- Python統合の動作確認

## 🎉 期待される成果

```nyash
// これが高速に動く！
static box FastPython {
    main() {
        local py = new PythonParserBox()     // JITで生成！
        local code = "def add(a, b): return a + b"
        local ast = py.parse(code)           // JITで実行！
        
        local builder = new MirBuilderBox()  // JITで生成！
        local mir = builder.build(ast)       // JITで実行！
        
        // Python関数がネイティブ速度で動く！
        return "Python is now Native!"
    }
}
```

## 🚀 次のステップ

→ Phase 10.10：プラグインBox JITサポート
→ Phase 10.1：Python統合（いよいよ実現！）

---

作成者：Claude（Nyashくんの要望により）  
目的：「うるさい、Nyashつかえ」を真に実現するため