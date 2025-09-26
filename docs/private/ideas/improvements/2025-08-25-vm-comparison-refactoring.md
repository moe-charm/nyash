# VM比較処理のリファクタリング
Status: Pending (80%実装済み)
Created: 2025-08-25
Priority: Medium
Related-Code: src/backend/vm_instructions.rs::execute_compare(), src/backend/vm_values.rs::execute_compare_op()

## 現状（80%実装）
- BoxRef比較のTypeErrorは解決済み
- 正規化処理が2箇所に分散（execute_compare, execute_compare_op）
- デバッグログが複数の環境変数に依存
- 基本動作は完全に正常

## 問題点
1. **重複した正規化ロジック**
   - vm_instructions.rs: BoxRef → Integer変換
   - vm_values.rs: 同様の変換処理
   
2. **デバッグの複雑さ**
   - NYASH_VM_DEBUG
   - NYASH_VM_DEBUG_CMP
   - 複数箇所でのログ出力

3. **エラーパスの複雑さ**
   - "[BoxRef-BoxRef]", "[BoxRef-Integer]", "[Integer-BoxRef]", "[Default]"
   - どこでエラーが出るか予測困難

## 改善案（残り20%）

### 1. 正規化を単一関数に統一
```rust
fn canonicalize_for_comparison(value: VMValue) -> VMValue {
    match value {
        VMValue::BoxRef(b) => {
            // IntegerBox → Integer
            if let Some(ib) = b.as_any().downcast_ref::<IntegerBox>() {
                return VMValue::Integer(ib.value);
            }
            // String parse fallback
            if let Ok(n) = b.to_string_box().value.trim().parse::<i64>() {
                return VMValue::Integer(n);
            }
            VMValue::BoxRef(b)
        }
        other => other,
    }
}
```

### 2. 比較処理の階層化
```rust
impl VM {
    // エントリーポイント
    pub fn execute_compare(...) {
        let left = self.canonicalize_value(lhs)?;
        let right = self.canonicalize_value(rhs)?;
        let result = self.compare_canonical(op, &left, &right)?;
        self.set_value(dst, VMValue::Bool(result));
    }
    
    // 正規化
    fn canonicalize_value(&self, id: ValueId) -> Result<VMValue, VMError> {
        let raw = self.get_value(id)?;
        Ok(canonicalize_for_comparison(raw))
    }
    
    // 比較実行
    fn compare_canonical(&self, op: &CompareOp, left: &VMValue, right: &VMValue) -> Result<bool, VMError> {
        // シンプルな比較ロジックのみ
    }
}
```

### 3. デバッグトレースの統一
```rust
struct ComparisonTrace {
    original_left: VMValue,
    original_right: VMValue,
    canonical_left: VMValue,
    canonical_right: VMValue,
    result: Result<bool, VMError>,
}
```

## 実装タイミング
- [ ] 次の大きなバグが出たとき
- [ ] Phase 10の最適化フェーズ
- [ ] 新しい比較演算子追加時

## メリット
- デバッグが容易になる
- 新しい型の比較追加が簡単
- テストが書きやすくなる