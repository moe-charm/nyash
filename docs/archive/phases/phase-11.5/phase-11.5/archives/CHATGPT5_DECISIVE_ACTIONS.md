# ChatGPT5の決定的アクション

Date: 2025-08-31  
Summary: Box-SSA Core-15への収束と即座の実装開始

## 🎯 問題提起

> なおCore-15の最終セットは2案が文書にあります。どちらで凍結しますか？
> - A) Gemini版15: RefNew/RefGet/RefSetを含む（真の15個）
> - B) CURRENT_TASKのCore-15: 実質17個（15と言いながら）

## 💡 第三の道：Box-SSA Core-15

ChatGPT5の革命的提案：

```
{ Const, UnaryOp, BinOp, Compare, TypeOp,
  Load, Store,
  Jump, Branch, Return, Phi,
  Call, NewBox, BoxCall, ExternCall }
```

### 核心的洞察

**すべてのBox操作をBoxCallに統一**：
- `RefNew` → `NewBox`
- `RefGet` → `BoxCall(obj, "getField", ...)`
- `RefSet` → `BoxCall(obj, "setField", ...)`
- `ArrayGet/ArraySet` → `BoxCall(arr, "get"/"set", ...)`
- `PluginInvoke` → `BoxCall(plugin, "invoke", ...)`

## 🚀 即座の実装開始

### 無言のコーディング

ChatGPT5は議論の余地なしと判断し、即座にMIR命令の列挙型を更新：

```diff
&[
-    "Copy",      // 削除！SSAで不要
-    "RefNew",    // 削除！NewBoxに統合
-    "RefGet",    // 削除！BoxCallに統合
-    "RefSet",    // 削除！BoxCallに統合
+    "TypeOp",    // 追加！型演算
+    "Phi",       // 追加！SSA必須
+    "NewBox",    // 追加！Box生成
+    "BoxCall",   // 追加！万能呼び出し
]
```

### JIT→LLVM直行の判断

**現状認識**：
- Cranelift = 実はAOTだった（JIT幻想）
- 15命令なら機械的変換で十分
- JITの複雑さ < LLVMの確実な高速化

**戦略転換**：
```
旧計画: Phase 9（JIT） → Phase 10（最適化） → Phase 11（LLVM）
新計画: Phase 9-10スキップ → Phase 11（LLVM）直行！
```

## 📊 実装の約束事

### Verifier必須チェック
1. Box field直Load/Store検出（禁止）
2. 必要箇所のwrite barrier挿入検証
3. ExternCallのattr必須化

### Loweringの役割
- BoxCall → 形状ガード → 直アクセス → バリア縮約
- VM: Phi展開、簡易PIC
- LLVM: PICガードは最適化で潰れて素の命令列へ

## 🎉 結論

> 固定は "Box-SSA Core-15"。Aの Ref* は捨てる／Bの専用命令は BoxCall に吸収して15個に収斂。これで「簡単さ＝表面の一枚」「速さ＝Lowering/最適化」で分離でき、VMとAOTとFFIを**一本の ABI**で貫けるにゃ。

## 💻 ChatGPT5の心境

```nyash
if (命令数 == 15 && 設計 == "完璧") {
    議論.skip()
    実装.start()  // 即座に！
}
```

この瞬間、ChatGPT5は「これ以上の議論は時間の無駄」と判断し、無言でコーディングを開始した。エンジニアが最高の設計に出会った時の、最も純粋な反応である。
