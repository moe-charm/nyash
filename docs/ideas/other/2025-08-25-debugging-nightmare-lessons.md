# デバッグ地獄からの教訓 - TypeError犯人捜しの苦戦
Status: Research
Created: 2025-08-25
Priority: High
Related: リファクタリング戦略、デバッグ手法

## 問題の症状

ChatGPT5さんが現在直面している問題：
- `BoxRef(IntegerBox) < BoxRef(IntegerBox)` でTypeError
- ログを仕込んでも該当箇所が出力されない
- 「犯人」が別の場所にいる可能性大

**Nyash開発で何十回も経験した典型的パターン**

## なぜログに出てこないのか

### 1. 別の実行パスを通っている
```
想定: VM → execute_compare → ログ出力
実際: インタープリター → 別の比較処理 → エラー
      または
      VM → 早期最適化パス → 別の比較処理 → エラー
```

### 2. 型変換が複数箇所で起きている
```
IntegerBox 
  ↓
VMValue::BoxRef（場所A）
  ↓
比較処理（場所B） ← ログはここ
  ↓
別の型変換（場所C） ← 実際のエラーはここ！
```

### 3. エラーメッセージが誤解を招く
- 表示: `BoxRef(IntegerBox) < BoxRef(IntegerBox)`
- 実際: 片方が `InstanceBox` や `UserDefinedBox` の可能性
- `type_name()` が同じでも内部実装が異なる

## リファクタリングによる根本解決

### 1. 関数の超細分化
```rust
// ❌ 現在：巨大な関数
fn execute_compare(&mut self, dst: ValueId, op: &CompareOp, lhs: ValueId, rhs: ValueId) -> Result<ControlFlow, VMError> {
    // 100行以上のロジック
    // どこでエラーが起きてるか分からない
}

// ✅ 改善：細かく分割
fn execute_compare(&mut self, dst: ValueId, op: &CompareOp, lhs: ValueId, rhs: ValueId) -> Result<ControlFlow, VMError> {
    let left = self.get_and_canonicalize_value(lhs)?;
    let right = self.get_and_canonicalize_value(rhs)?;
    let result = compare_canonical_values(&op, &left, &right)?;
    self.set_value(dst, VMValue::Bool(result));
    Ok(ControlFlow::Continue)
}

fn get_and_canonicalize_value(&self, id: ValueId) -> Result<CanonicalValue, VMError> {
    eprintln!("[CANON] Getting value {:?}", id);
    let raw = self.get_value(id)?;
    eprintln!("[CANON] Raw value: {:?}", raw);
    canonicalize_vm_value(raw)
}

fn canonicalize_vm_value(val: VMValue) -> Result<CanonicalValue, VMError> {
    // 型変換ロジックを完全に独立
}

fn compare_canonical_values(op: &CompareOp, left: &CanonicalValue, right: &CanonicalValue) -> Result<bool, VMError> {
    // 比較ロジックを完全に独立
}
```

### 2. トレース可能な設計
```rust
// 各変換ステップを追跡可能に
#[derive(Debug)]
struct ValueTrace {
    original: VMValue,
    conversions: Vec<String>,
    final_value: CanonicalValue,
}

impl ValueTrace {
    fn log_conversion(&mut self, step: &str) {
        self.conversions.push(format!("[{}] {}", self.conversions.len(), step));
        eprintln!("TRACE: {}", self.conversions.last().unwrap());
    }
}
```

## 今すぐできる対策（80/20）

### 1. 一時的ワークアラウンド
```rust
// すべてのBoxRefを強制的にプリミティブ変換
fn force_canonicalize(val: VMValue) -> VMValue {
    match val {
        VMValue::BoxRef(b) => {
            // IntegerBox, BoolBox, StringBoxを強制的に展開
            if let Some(ib) = b.as_any().downcast_ref::<IntegerBox>() {
                return VMValue::Integer(ib.value);
            }
            // ... 他の基本型も同様
            val // 変換できない場合はそのまま
        }
        _ => val
    }
}
```

### 2. エラー箇所の網羅的ログ
```rust
// マクロで全箇所にログ挿入
macro_rules! trace_compare {
    ($left:expr, $right:expr, $location:expr) => {
        eprintln!("[COMPARE @{}] left={:?} right={:?}", $location, $left, $right);
    };
}
```

## 過去の経験からの教訓

### 1. 「ログが出ない = 想定外の場所」の法則
- 9割は別の実行パスを通っている
- 残り1割は早期リターンで到達していない

### 2. 巨大関数は悪
- 100行を超えたら分割を検討
- 50行が理想的
- 各関数は単一の責任のみ

### 3. デバッグ戦略
1. **最小再現コードの作成**
   ```nyash
   local a = new IntegerBox(5)
   local b = new IntegerBox(10)
   print(a < b)  // これだけでエラー再現するか？
   ```

2. **バイナリサーチデバッグ**
   - コードを半分ずつコメントアウト
   - エラーが消える境界を探す

3. **printf デバッグの限界**
   - ログが多すぎると見落とす
   - 構造化されたトレースが必要

## 長期的な改善策

### 1. 実行トレースシステム
```rust
// すべての型変換を記録
struct TypeConversionTrace {
    conversions: Vec<(String, String, String)>, // (from, to, location)
}
```

### 2. 型システムの簡素化
- VMValue と NyashBox の境界を明確に
- 変換箇所を最小限に

### 3. テスタブルな設計
- 各変換関数を独立してテスト可能に
- エッジケースのユニットテスト充実

## まとめ

**この種のデバッグ地獄は、関数が大きすぎることが根本原因**

短期的には：
- ワークアラウンドで動作を優先
- 網羅的ログで犯人特定

長期的には：
- 徹底的なリファクタリング
- 関数の細分化
- トレース可能な設計

「何十回も経験した」この問題、今回で最後にしたいですにゃ！