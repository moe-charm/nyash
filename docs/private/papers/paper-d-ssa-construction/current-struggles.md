# 現在のSSA構築における苦闘記録

*2025-09-12時点の生々しい記録*

## 🔥 現在直面している問題

### 1. PHI配線の複雑さ
```llvm
; エラー例
PHINode should have one entry for each predecessor!
  %phi_30 = phi i64 [ %74, %Main_esc_json_1_bb27 ]
```

**原因**:
- MIRのPHI入力と実際の前任ブロックの不一致
- 動的な制御フロー変更によるpred/succ関係のずれ
- sealed/unsealed両モードでの配線タイミングの混乱

### 2. 型の不整合
```llvm
; よくある問題
%str = phi i8* [ %handle_as_i64, %bb1 ], [ %ptr, %bb2 ]
; → i64 handleとi8* ptrの混在でverifyエラー
```

**Nyash特有の問題**:
- Everything is Box → すべてi64 handle
- でもLLVMは型付き → ptr型が必要な場面
- 動的な型変換の必要性

### 3. Dominance違反
```llvm
Instruction does not dominate all uses!
  %val = ...
  %use = ... %val ...  ; valの定義より前に使用？
```

**複雑な要因**:
- 文字列定数のhoisting（entry blockへ移動）
- ループ内での値の循環参照
- BuilderCursorの位置管理ミス

## 💡 試行錯誤の記録

### Phase 1: 素朴なアプローチ（失敗）
```rust
// 最初の試み：単純にPHIを作って配線
let phi = builder.build_phi(ty, "phi");
phi.add_incoming(&[(&val, pred_bb)]);  // → 爆発
```

### Phase 2: Sealed SSA導入（部分的成功）
```rust
// block終了時点でのスナップショット
let block_end_values: HashMap<BlockId, HashMap<ValueId, Value>> = ...;
// seal時にスナップショットから配線
```

**改善点**:
- pred終了時点の値を確実に取得
- でも完全ではない（未定義値の扱い）

### Phase 3: BuilderCursor（現在）
```rust
// 位置管理を構造化
cursor.with_block(bid, bb, |c| {
    // この中でのみ命令挿入可能
    c.emit_term(bid, |b| { /* terminator */ });
});
```

**狙い**:
- 終端後の命令挿入を構造的に防止
- でもまだPHI問題は解決せず...

## 📊 デバッグから得た洞察

### 1. PHIトレースログの分析
```
[PHI:new] fn=Main_esc_json_1 bb=30 dst=30 ty=i64 inputs=(23->7),(27->30)
[PHI] incoming add pred_bb=27 val=30 ty=i64
; → bb=23からの配線が欠落！
```

### 2. ゼロ合成の功罪
```rust
// 未定義値への対処として0を合成
match phi_ty {
    IntType => int_ty.const_zero(),
    PtrType => ptr_ty.const_null(),
    // ...
}
```

**利点**: verifyエラーを回避して前進
**欠点**: 意味的に正しくない可能性

### 3. LoopFormへの期待
```
現在: 複雑なCFG → 複雑なPHI配置
LoopForm: 定型dispatch → PHI配置が自明に！
```

## 🚀 次の一手

### 短期（今すぐ）
1. preds_by_blockの完全な正規化
2. PHI配線の網羅的チェック
3. 型変換の統一ポリシー

### 中期（LoopForm）
1. whileループのみLoopForm化
2. dispatch blockでのPHI集約
3. 段階的な適用拡大

### 長期（論文化）
1. この苦闘を体系化
2. アルゴリズムとして定式化
3. 他言語への適用可能性

## 🎓 学術的価値

**なぜこの苦闘が論文になるか**:

1. **実践的課題**: 理論と実装のギャップ
2. **Box言語特有**: 動的型付けとSSAの相性
3. **解決手法**: BuilderCursor、sealed SSA、LoopForm
4. **再現可能性**: 具体的なプログラムとログ

**想定読者**:
- 言語実装者
- コンパイラ研究者
- LLVM利用者

---

*この記録は、生々しい実装の苦闘を論文の素材として残すものである。*