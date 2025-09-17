# Phase 11.8: MIR命令セット究極整理 - Core‑13 で統一する

## 🎯 概要

ChatGPT5さんの深い洞察「**MIRは接着剤、Boxが世界**」を実現する究極のMIR整理。
現在の26（拡張版）→ Core‑15 → Core‑14（Phase 12）→ **Core‑13（最終決定・固定）**。

決定（2025‑09‑04）
- 目標を「Core‑13」に固定し、移行フラグを既定ONにする。
- 以降の最適化/検証/CIは Core‑13 を前提とする（旧命令は禁制）。

### 基本哲学

- **MIR = マイクロカーネル**: 最小限の制御と計算のみ
- **Box = すべての実データと操作**: Everything is Box の究極形
- **ExternCall = システムコール**: 外界との最小インターフェース

## 📊 現状分析

### 現行（移行前の参考）Core‑15（Phase 11.7）

```
基本演算(5): Const, UnaryOp, BinOp, Compare, TypeOp
メモリ(2): Load, Store
制御(4): Branch, Jump, Return, Phi
Box(3): NewBox, BoxCall, PluginInvoke
配列(2): ArrayGet, ArraySet
外部(1): ExternCall
```

### Core‑14（Phase 12の中間目標）

```
基本演算(5): Const, UnaryOp, BinOp, Compare, TypeOp
メモリ(2): Load, Store
制御(4): Branch, Jump, Return, Phi
Box(2): NewBox, BoxCall  ← PluginInvoke統合
配列(2): ArrayGet, ArraySet
外部(1): ExternCall
```

## 🚀 Core‑13（最終形）への道筋（実行計画）

### Step 1: 配列操作のBoxCall統合（Core‑14 → Core‑12）

```mir
// 現在
%val = ArrayGet %arr, %idx
ArraySet %arr, %idx, %val

// 統合後
%val = BoxCall %arr, "get", [%idx]
BoxCall %arr, "set", [%idx, %val]
```

実装方針:
- Optimizer: ArrayGet/ArraySet → BoxCall 変換
- VM: 高頻度パスは内部最適化維持
- JIT: 既知型の場合はインライン展開

### Step 2: Load/Store の再考（Core‑12 → Core‑11）

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

### Step 3: 定数統合とUnaryOp簡素化（Core‑11 → Core‑13）

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

## 🎯 最終形：Core‑13（固定セット・CI基準）

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

移行スイッチ（既定ON）と検証
- 環境変数（デフォルトON）
  - NYASH_MIR_CORE13=1（Core‑13一括）
  - 診断: NYASH_OPT_DIAG_FORBID_LEGACY=1（旧命令が最終MIRに残ったらエラー）
- ビルダー/最適化の方針
  - Builder: ArrayGet/ArraySet・RefGet/RefSet を emit せず最初から BoxCall を出す
  - Optimizer: 既存の Array/Ref→BoxCall 正規化パスを保持（保険）
  - UnaryOp→BinOp 正規化は常時ON（簡易変換）
  - Load/Store はSSA利用で極力抑止（最終MIRから排除が目標）
- VM/JIT
  - BoxCall fast‑path/vtable を維持し、get/set は型特化とWriteBarrierを維持
  - PluginInvoke はMIRから排除（必要経路は BoxCall→VM側ABI判定）

CI/テスト
- Core‑13固定の数・名前検査を `instruction_introspection.rs` に追加（Core‑15検査は保持しつつ非推奨）
- 旧命令（ArrayGet/ArraySet/RefGet/RefSet/Load/Store/UnaryOp）が最終MIRに残らないことをゲート
- 代表スモーク（配列/参照/extern/await）は VM/JIT で同値性を確認

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

## 📋 実装ロードマップ（確定版）

### ステータス（進捗メモ）
- 実装済み（トグルONで有効化）
  - Optimizer: ArrayGet/Set・RefGet/Set → BoxCall 変換（`NYASH_MIR_ARRAY_BOXCALL`, `NYASH_MIR_REF_BOXCALL`, `NYASH_MIR_CORE13`）
  - VM: BoxCall(setField)のWriteBarrier、Array/Instanceの軽量fast-path（by-name/slot併用）
  - 管理棟: 主要なMIR/GC/Optimizerフラグを `config::env` に集約
- 決定/実行（今回）
  - Core‑13を既定ON（nyash.toml [env] 推奨値）
  - 旧命令禁止の診断を既定ON
  - BuilderのArray/Ref出力をBoxCallに変更（emit抑止）
  - Unary→BinOpを常時変換
- 未了/次段
  - JIT: BoxCall fast‑path の inlining（bounds/Barrier含む）
  - ベンチとCIゲート（array/field/arithmetic_loop）
  - InstanceのgetField/setFieldのslot化（name→slotの検討）
  - 直env参照の段階移行（ログ用途は後段）

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
