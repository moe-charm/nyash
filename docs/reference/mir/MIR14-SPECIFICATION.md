# MIR14 命令セット仕様（2025-09-23現在）

## 📊 MIR14とは

**MIR14 = Core-13 + UnaryOp**

27命令→13命令→14命令という実践的な進化を経て、現在14命令で全実行形態をサポート。

## 🎯 Core-14命令

### 基本演算（5命令）
1. **Const** - 定数ロード
2. **BinOp** - 二項演算（+,-,*,/,%,&,|,^,<<,>>）
3. **UnaryOp** - 単項演算（-,!,~）← 14番目の命令
4. **Compare** - 比較演算（==,!=,<,<=,>,>=）
5. **TypeOp** - 型操作（check/cast）

### メモリ（2命令）
6. **Load** - メモリ読み込み
7. **Store** - メモリ書き込み

### 制御（4命令）
8. **Branch** - 条件分岐
9. **Jump** - 無条件ジャンプ
10. **Return** - 関数リターン
11. **Phi** - SSA合流

### Box（2命令）
12. **NewBox** - Box生成
13. **BoxCall** - Boxメソッド呼び出し

### 外部（1命令）
14. **ExternCall** - 外部関数呼び出し

## ❌ Core-14に含まれない命令

### Call系拡張（統合予定）
- **Call** - 関数呼び出し（Callee型で拡張中）
- **PluginInvoke** - プラグイン呼び出し（BoxCallと統合予定）
- **NewClosure** - クロージャ生成

### レガシー命令（段階的削除）
- **Print** - ExternCallに統合
- **Debug** - ExternCallに統合
- **Copy** - 最適化パス用
- **Nop** - 何もしない

### 高度な機能（オプション）
- **ArrayGet/ArraySet** - 配列操作
- **RefGet/RefSet/RefNew** - 参照操作
- **WeakRef/Barrier** - GC関連
- **FutureNew/FutureSet/Await** - 非同期
- **Throw/Catch/Safepoint** - 例外処理

## 📈 命令数の変遷

```
初期（27命令）: なんでも入れた状態
  ↓
MIR13（13命令）: 極限まで削減
  ↓
MIR14（14命令）: UnaryOp追加で実用的に ← 現在
  ↓
将来: Call統一でさらにシンプルに？
```

## 🎯 設計原則

1. **最小限主義**: 本当に必要な命令だけ
2. **Box中心**: すべてはBoxCallで表現
3. **段階的拡張**: Core-14を基盤に、必要に応じて拡張

## 📊 実行形態別サポート

| 命令 | Interpreter | VM | JIT | AOT | 備考 |
|------|------------|----|----|-----|------|
| Core-14 | ✅ | ✅ | ✅ | ✅ | 完全サポート |
| Call系 | ✅ | ✅ | △ | △ | 統合中 |
| レガシー | ✅ | ✅ | × | × | 削除予定 |
| 高度機能 | △ | △ | × | × | オプション |

## 🚀 今後の方向性

### Phase 15.5（現在）
- Call系6命令を1つに統一（ChatGPT5 Pro A++案）
- MirCall + Calleeで表現力向上

### 将来
- Core-14 + MirCallで完全体へ
- レガシー命令の完全削除
- 高度機能の選択的サポート

## 📝 参考文献
- [論文A: MIR14で作る万能実行系](docs/private/papers/paper-a-mir13-ir-design/)
- [Call命令現状](docs/reference/mir/call-instructions-current.md)
- [MIR命令実装](src/mir/instruction.rs)