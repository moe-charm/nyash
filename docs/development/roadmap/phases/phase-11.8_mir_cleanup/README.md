# Phase 11.8: MIR命令セット究極整理 - Core-13への道

## 🎯 概要

ChatGPT5さんの深い洞察「**MIRは接着剤、Boxが世界**」を実現する究極のMIR整理。
現在の26命令 → Core-15 → Core-14（Phase 12）→ **Core-13（最終目標）**への段階的削減。

### 基本哲学

- **MIR = マイクロカーネル**: 最小限の制御と計算のみ
- **Box = すべての実データと操作**: Everything is Box の究極形
- **ExternCall = システムコール**: 外界との最小インターフェース

## 📊 現状分析

### 現在のCore-15（Phase 11.7）

```
基本演算(5): Const, UnaryOp, BinOp, Compare, TypeOp
メモリ(2): Load, Store
制御(4): Branch, Jump, Return, Phi
Box(3): NewBox, BoxCall, PluginInvoke
配列(2): ArrayGet, ArraySet
外部(1): ExternCall
```

### Core-14（Phase 12予定）

```
基本演算(5): Const, UnaryOp, BinOp, Compare, TypeOp
メモリ(2): Load, Store
制御(4): Branch, Jump, Return, Phi
Box(2): NewBox, BoxCall  ← PluginInvoke統合
配列(2): ArrayGet, ArraySet
外部(1): ExternCall
```

## 🚀 Core-13への道筋

### Step 1: 配列操作のBoxCall統合（Core-14 → Core-12）

```mir
// 現在
%val = ArrayGet %arr, %idx
ArraySet %arr, %idx, %val

// 統合後
%val = BoxCall %arr, "get", [%idx]
BoxCall %arr, "set", [%idx, %val]
```

**実装方針**:
- Optimizer: ArrayGet/ArraySet → BoxCall 変換
- VM: 高頻度パスは内部最適化維持
- JIT: 既知型の場合はインライン展開

### Step 2: Load/Store の再考（Core-12 → Core-11）

**SSAの威力を活かす**:
- ローカル変数のLoad/Store → SSA変数で代替
- 真に必要なのはBoxフィールドアクセスのみ
- それもBoxCall("getField"/"setField")で統合可能

```mir
// 現在
Store %slot, %value
%val = Load %slot

// SSA化
%val = %value  // 直接参照（Copyも実質不要）
```

### Step 3: 定数統合とUnaryOp簡素化（Core-11 → Core-13）

**Const統合案**:
```mir
// 現在
Const::Integer(i64)
Const::Float(f64)
Const::Bool(bool)
Const::String(String)
Const::Null

// 統合後
Const { type: Type, value: u64 }  // 全て64bitに収める
```

**UnaryOp削減**:
- Neg → BinOp(Sub, 0, x)
- Not → BinOp(Xor, x, 1)
- BitNot → BinOp(Xor, x, -1)

## 🎯 最終形：Core-13

```yaml
定数(1):
  - Const（統合型：i64/f64/bool/null/handle）

計算(2):
  - BinOp（Add/Sub/Mul/Div/Mod/And/Or/Xor/Shl/Shr）
  - Compare（Eq/Ne/Lt/Le/Gt/Ge）

制御(4):
  - Branch（条件分岐）
  - Jump（無条件ジャンプ）  
  - Phi（SSA合流）
  - Return（関数終了）

呼出(3):
  - Call（Nyash関数呼び出し）
  - BoxCall（Box操作統一）
  - ExternCall（環境アクセス）

メタ(3):
  - TypeOp（型チェック/キャスト）
  - Safepoint（GCセーフポイント）
  - Barrier（メモリバリア）

合計: 13命令
```

## 💡 なぜCore-13で十分なのか

### 1. チューリング完全性の保証

最小限必要なもの:
- 定数
- 算術演算
- 条件分岐
- ループ（Jump + Branch）
- 関数呼び出し

これらはすべてCore-13に含まれる。

### 2. Everything is Box の威力

```nyash
// すべてがBoxCall経由
arr[0]           → BoxCall(arr, "get", [0])
arr[0] = 42      → BoxCall(arr, "set", [0, 42])
obj.field        → BoxCall(obj, "getField", ["field"])
obj.field = val  → BoxCall(obj, "setField", ["field", val])
weak.get()       → BoxCall(weak, "get", [])
```

### 3. SSAによるメモリ命令の削減

- 一時変数 → SSA変数（Load/Store不要）
- フィールド → BoxCall
- 配列要素 → BoxCall
- 真のメモリアクセスはBoxの中に隠蔽

## 📋 実装ロードマップ

### ステータス（進捗メモ）
- 実装済み（トグルONで有効化）
  - Optimizer: ArrayGet/Set・RefGet/Set → BoxCall 変換（`NYASH_MIR_ARRAY_BOXCALL`, `NYASH_MIR_REF_BOXCALL`, `NYASH_MIR_CORE13`）
  - VM: BoxCall(setField)のWriteBarrier、Array/Instanceの軽量fast-path（by-name/slot併用）
  - 管理棟: 主要なMIR/GC/Optimizerフラグを `config::env` に集約
- 未了/次段
  - JIT: BoxCall fast-path の inlining（bounds/Barrier含む）
  - ベンチ追加とCIゲート（array/field/arithmetic_loop）
  - フィールドfast-pathのslot化（name→slot化の検討）
  - 直env参照の残りの段階移行（ログ用途は後段）

### Phase 11.8.1: 準備と分析（1週間）

- [ ] 現在のMIR使用状況の詳細分析
- [ ] ArrayGet/ArraySet → BoxCall 変換の影響調査
- [ ] Load/Store 削除可能性の検証
- [ ] パフォーマンスベンチマーク基準値測定

### Phase 11.8.2: ArrayGet/ArraySet統合（2週間）

- [ ] Optimizer: ArrayGet/ArraySet → BoxCall 変換パス
- [ ] VM: BoxCall("get"/"set") の最適化パス
- [ ] JIT: 既知ArrayBoxの特殊化維持
- [ ] テスト: 既存配列操作の回帰テスト

### Phase 11.8.3: Load/Store削減（3週間）

- [ ] Builder: SSA最大活用でLoad/Store削減
- [ ] フィールドアクセス → BoxCall 変換
- [ ] VM/JIT: 最適化パスの調整
- [ ] ベンチマーク: パフォーマンス影響測定

### Phase 11.8.4: 最終統合（2週間）

- [ ] Const型統合実装
- [ ] UnaryOp → BinOp 変換
- [ ] Core-13命令セット確定
- [ ] ドキュメント最終更新

## ⚠️ リスクと緩和策

### パフォーマンスリスク

**リスク**: BoxCall統合によるオーバーヘッド
**緩和策**: 
- VM層での型別最適化維持
- JIT時の積極的インライン展開
- 高頻度パスのNyRTシム化

### 互換性リスク

**リスク**: 既存MIRコードの非互換
**緩和策**:
- Rewriteパスで自動変換
- 段階的移行（警告→エラー）
- 環境変数でレガシーモード

### 複雑性リスク

**リスク**: BoxCallの過度な多重化
**緩和策**:
- 明確な命名規約（get/set/getField等）
- 型情報による静的検証強化
- デバッグ情報の充実

## 🎯 成功指標

1. **命令数**: 26 → 13（50%削減）
2. **パフォーマンス**: ベンチマークで±5%以内
3. **コードサイズ**: MIRダンプサイズ20%削減
4. **保守性**: 新Box追加時のMIR変更不要

## 📚 関連ドキュメント

- [MIR Instruction Set](../../../reference/mir/INSTRUCTION_SET.md)
- [Phase 12: PluginInvoke統合](../phase-12/README.md)
- [Everything is Box哲学](../../../philosophy/everything-is-box.md)

---

*「少ないほど豊かである」- MIRは最小の接着剤、Boxが無限の世界を創る*
