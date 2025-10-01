# Box-SSA Core-15 最終決定

Date: 2025-08-31  
Status: **固定仕様** (Frozen Specification)  
Summary: MIR命令セットを真の15個に統一

## 📊 問題と解決

### 混乱していた2案
1. **Gemini版15**: RefNew/RefGet/RefSet含む（15個だがBox哲学に反する）
2. **文書版Core-15**: 実は17個（Box哲学に近いが数が合わない）

### ChatGPT5の第三案で決着
```
{ Const, UnaryOp, BinOp, Compare, TypeOp,
  Load, Store,
  Jump, Branch, Return, Phi,
  Call, NewBox, BoxCall, ExternCall }
```

## 🌟 革命的統一：BoxCall

すべてのBox操作を**BoxCall**一本に統一：

| 旧命令 | 新BoxCall表現 |
|--------|---------------|
| RefGet(obj, field) | BoxCall(obj, "getField", field) |
| RefSet(obj, field, val) | BoxCall(obj, "setField", field, val) |
| ArrayGet(arr, idx) | BoxCall(arr, "get", idx) |
| ArraySet(arr, idx, val) | BoxCall(arr, "set", idx, val) |
| PluginInvoke(p, m, args) | BoxCall(p, m, args) |

## 💡 技術的インパクト

### 実装の簡素化
- **Verifier**: BoxCallのチェックのみ
- **最適化**: PIC/インライン化がBoxCallに集中
- **GCバリア**: BoxCallのLoweringで統一処理
- **削減効果**: 26→15命令（42%削減）

### LLVM変換戦略（AI会議の合意）
1. **BoxCall最適化**: メソッドID + PIC（Polymorphic Inline Cache）
2. **脱箱化**: 2表現SSA（プリミティブ計算→必要時のみBox化）
3. **GCバリア**: 世代別GCで若→若は省略
4. **注釈→属性**: 保守的マッピング（安全性優先）

## 🚀 実装への影響

### JIT→LLVM直行の決断
- Cranelift = 実はAOTだった（JIT幻想）
- 15命令なら機械的変換で十分
- Phase 9-10スキップ → Phase 11（LLVM）直行

### ChatGPT5の反応
```
Box-SSA Core-15を聞く
    ↓
完璧と判断
    ↓
無言でコーディング開始（議論の余地なし）
```

## 📝 今後の課題

1. **MIR注釈システム**: 命令数を増やさずに最適化ヒント付与（設計案のみ）
2. **LLVM実装**: inkwellセットアップから開始
3. **既存コード移行**: 26→15命令への段階的移行

## 🎉 結論

**Box-SSA Core-15**により「Everything is Box」哲学が完全開花：
- 真の15命令達成
- 実装の劇的簡素化
- 最適化の統一的適用

これが「あほみたいに簡単」で「恐ろしく速い」Nyashの最終形態！

---

詳細なAI会議記録は [archives/](archives/) フォルダに保存
