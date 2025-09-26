# VM and/or 短絡評価の実装
Status: Pending (80%実装済み)
Created: 2025-08-25
Priority: Low
Related-Code: src/backend/vm_instructions.rs::execute_binop()

## 現状（80%実装）
- `as_bool()`で両オペランドを評価してから論理演算を実行
- 基本動作は完全に正常、テストもすべて通過
- コード：
  ```rust
  BinOp::And => {
      let left_bool = left_val.as_bool();
      let right_bool = right_val.as_bool();
      Ok(self.allocate_value(VMValue::Bool(left_bool && right_bool)))
  }
  ```

## 改善案（残り20%）
### 1. 短絡評価の実装
- `And`: 左辺がfalseなら右辺の評価をスキップ
- `Or`: 左辺がtrueなら右辺の評価をスキップ
- 効果: 不要な計算の削減、副作用のある式での正しい動作

### 2. 実装スケッチ
```rust
BinOp::And => {
    let left_val = self.get_value(left)?;
    if !left_val.as_bool() {
        // 左辺がfalseなら即座にfalseを返す
        return Ok(self.allocate_value(VMValue::Bool(false)));
    }
    // 左辺がtrueの場合のみ右辺を評価
    let right_val = self.get_value(right)?;
    Ok(self.allocate_value(VMValue::Bool(right_val.as_bool())))
}
```

### 3. 考慮事項
- MIRレベルでの最適化との整合性
- デバッグ時のステップ実行への影響
- パフォーマンステストの必要性

## 実装タイミング
- [ ] パフォーマンス問題が報告されたら
- [ ] 副作用のある式（関数呼び出し等）で問題が発生したら
- [ ] Phase 10（最適化フェーズ）で一括対応

## メモ
- 現在の実装でも機能的には問題ない
- Pythonも初期は短絡評価なしだった（後から追加）
- まずは動くことを優先する80/20ルールの良い例