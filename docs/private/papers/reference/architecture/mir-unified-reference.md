# 🤖 Nyash MIR (Mid-level Intermediate Representation) - 統合リファレンス

*26命令削減実装中・ChatGPT5仕様準拠 - 2025年8月17日版*

## 🚨 **重要: MIR命令削減プロジェクト進行中**

**現状**: 35命令実装（175%膨張）→ **目標**: 26命令（ChatGPT5仕様）  
**Gemini評価**: 削減戦略「極めて健全」「断行推奨」

## 🎯 **MIR概要**

Nyash MIRは、Everything is Box哲学を基盤とした中間表現です。現在35命令が実装され、インタープリター・VM・WASM・AOTの全バックエンドで統一された実行を実現します。

### **🌟 主要特徴**
- **Everything is Box**: 全データがBoxオブジェクトとして統一表現
- **Effect System**: pure/mut/io/control効果による最適化基盤
- **所有権管理**: 強参照森（ownership forest）+ weak参照システム
- **非同期対応**: Future/Bus操作の言語レベル統合
- **FFI/ABI統合**: ExternCall命令による外部API統一呼び出し

## 🏗️ **命令分類 - 35命令全体系**

### **Tier-0: コア演算 (8命令)**
基本的な計算・制御フロー命令

| 命令 | 形式 | 効果 | 説明 |
|------|------|------|------|
| `Const` | `%dst = const value` | pure | 定数値生成 |
| `BinOp` | `%dst = %lhs op %rhs` | pure | 二項演算（+,-,*,/等） |
| `UnaryOp` | `%dst = op %operand` | pure | 単項演算（not, neg等） |
| `Compare` | `%dst = %lhs cmp %rhs` | pure | 比較演算（==, !=, <等） |
| `Branch` | `br %cond -> %then, %else` | control | 条件分岐 |
| `Jump` | `jmp %target` | control | 無条件ジャンプ |
| `Return` | `ret %value?` | control | 関数戻り |
| `Phi` | `%dst = phi [%val1:%bb1, %val2:%bb2]` | pure | SSA φ関数 |

### **Tier-1: メモリ・関数操作 (8命令)**
メモリアクセス・関数呼び出し・型操作

| 命令 | 形式 | 効果 | 説明 |
|------|------|------|------|
| `Load` | `%dst = load %ptr` | pure | メモリ読み取り |
| `Store` | `store %value -> %ptr` | mut | メモリ書き込み |
| `Call` | `%dst = call %func(%args...)` | context | 関数呼び出し |
| `BoxCall` | `%dst = %box.method(%args...)` | context | Boxメソッド呼び出し |
| `NewBox` | `%dst = new_box "Type"(%args...)` | mut | Box生成 |
| `TypeCheck` | `%dst = type_check %box "Type"` | pure | 型チェック |
| `Cast` | `%dst = cast %value as Type` | pure | 型変換 |
| `Copy` | `%dst = copy %src` | pure | 値コピー |

### **Tier-2: 配列・デバッグ・制御 (7命令)**
配列操作・デバッグ・例外処理

| 命令 | 形式 | 効果 | 説明 |
|------|------|------|------|
| `ArrayGet` | `%dst = %array[%index]` | pure | 配列要素取得 |
| `ArraySet` | `%array[%index] = %value` | mut | 配列要素設定 |
| `Debug` | `debug %value "message"` | io | デバッグ出力 |
| `Print` | `print %value` | io | コンソール出力 |
| `Nop` | `nop` | pure | 無操作 |
| `Throw` | `throw %exception` | control | 例外発生 |
| `Catch` | `catch %type -> %handler` | control | 例外捕捉 |

### **Tier-3: 参照・非同期・外部API (12命令)**
所有権管理・非同期処理・外部連携

| 命令 | 形式 | 効果 | 説明 |
|------|------|------|------|
| `Safepoint` | `safepoint` | io | セーフポイント |
| `RefNew` | `%dst = ref_new %box` | pure | 参照生成 |
| `RefGet` | `%dst = ref_get %ref.field` | pure | 参照経由読み取り |
| `RefSet` | `ref_set %ref.field = %value` | mut | 参照経由書き込み |
| `WeakNew` | `%dst = weak_new %box` | pure | weak参照生成 |
| `WeakLoad` | `%dst = weak_load %weak_ref` | pure | weak参照読み取り |
| `BarrierRead` | `barrier_read %ptr` | io | メモリバリア読み |
| `BarrierWrite` | `barrier_write %ptr` | io | メモリバリア書き |
| `FutureNew` | `%dst = future_new %value` | mut | Future生成 |
| `FutureSet` | `future_set %future = %value` | mut | Future値設定 |
| `Await` | `%dst = await %future` | io | Future待機 |
| `ExternCall` | `%dst = extern_call iface.method(%args...)` | context | 外部API呼び出し |

## 🎭 **Effect System - 4種類の効果**

### **効果分類と最適化ルール**

```rust
pub enum Effect {
    Pure,     // 再順序化可能、共通部分式除去可能
    Mut,      // 同一リソースで順序保持必要
    Io,       // 全順序保持必要（副作用あり）
    Control,  // 制御フロー変更
}
```

### **効果別命令分類**

#### **Pure命令 (15命令)**
```
Const, BinOp, UnaryOp, Compare, Phi, Load, TypeCheck, Cast, Copy,
ArrayGet, Nop, RefNew, RefGet, WeakNew, WeakLoad
```

#### **Mut命令 (7命令)**
```
Store, NewBox, ArraySet, RefSet, FutureNew, FutureSet
```

#### **Io命令 (6命令)**
```
Debug, Print, Safepoint, BarrierRead, BarrierWrite, Await
```

#### **Control命令 (4命令)**
```
Branch, Jump, Return, Throw, Catch
```

#### **Context依存命令 (3命令)**
```
Call, BoxCall, ExternCall
```
*効果は呼び出し先に依存*

## 🔧 **重要なMIR実装詳細**

### **ExternCall命令 - FFI/ABI統合**

```rust
ExternCall {
    dst: Option<ValueId>,
    iface_name: String,         // "env.console", "nyash.math"等
    method_name: String,        // "log", "sqrt"等
    args: Vec<ValueId>,
    effects: EffectMask,        // BID仕様から決定
}
```

**用途**: ブラウザーAPI・ネイティブライブラリ・プラグインの統一呼び出し

### **所有権管理システム**

#### **強参照森（Ownership Forest）**
- 各Boxは最大1つの強参照を持つ（in-degree ≤ 1）
- 強参照による循環は禁止（DAG構造保証）
- `NewBox`, `RefSet`で所有権移転

#### **weak参照システム**
- 所有権を持たない軽量参照
- `WeakNew`で生成、`WeakLoad`で安全アクセス
- 参照先削除時は自動的にnull化

### **非同期処理 - Future操作**

```mir
%future = FutureNew %initial_value  // Future生成
FutureSet %future = %result         // 結果設定
%value = Await %future              // 結果取得（ブロッキング）
```

## 🚀 **バックエンド別対応状況**

### **実装済みバックエンド**

| バックエンド | 対応命令数 | 主要用途 | 特徴 |
|-------------|-----------|----------|------|
| **Interpreter** | 35/35 | デバッグ・開発 | 全命令完全対応 |
| **VM** | 35/35 | 高速実行 | レジスタベース |
| **WASM** | 30/35 | Web配布 | ExternCall→import対応 |
| **AOT準備** | 計画中 | ネイティブ | LLVM IR生成予定 |

### **バックエンド固有の最適化**

#### **VM バックエンド**
- レジスタベース実行
- 局所最適化（ローカルBus elision）
- 直接スレッド化

#### **WASM バックエンド**
- メモリ線形化（文字列は (ptr,len)）
- ExternCall → import宣言自動生成
- ホスト側JavaScript連携

## 📊 **MIR最適化パス**

### **Pure命令最適化**
- **共通部分式除去 (CSE)**: 同一pure計算の除去
- **不変コード移動 (LICM)**: ループ外移動
- **定数畳み込み**: コンパイル時計算

### **Effect-aware最適化**
- **Mut順序保持**: 同一リソースアクセス順序維持
- **Io順序保持**: 全Io命令の順序保証
- **Bus elision**: ローカル通信の直接アクセス化

## 🧪 **テスト・検証**

### **MIR検証項目**
- [ ] **所有権森検証**: strong in-degree ≤ 1
- [ ] **強循環禁止**: 強参照のDAG構造保証
- [ ] **weak参照安全性**: 失効時null化
- [ ] **効果注釈正確性**: 各命令の効果分類
- [ ] **型安全性**: Box型システム整合性

### **バックエンド互換性テスト**
```bash
# 全バックエンドMIR一致テスト
./target/release/nyash --dump-mir program.nyash > interpreter.mir
./target/release/nyash --backend vm --dump-mir program.nyash > vm.mir
./target/release/nyash --backend wasm --dump-mir program.nyash > wasm.mir
diff interpreter.mir vm.mir && diff vm.mir wasm.mir
```

## 🔮 **将来計画**

### **Phase 10: AOT/JIT対応**
- LLVM IR生成バックエンド
- ExternCall → ネイティブ関数呼び出し
- 高度な最適化パス統合

### **Phase 11: MIR v2設計**
- 命令数最適化（35 → 25命令目標）
- BID統合（Box Interface Definition）
- リソース所有権拡張（own<T>, borrow<T>）

## 📚 **関連ドキュメント**

- **FFI/ABI仕様**: `docs/説明書/reference/box-design/ffi-abi-specification.md`
- **実装詳細**: `src/mir/instruction.rs`
- **Phase計画**: `docs/予定/native-plan/copilot_issues.txt`

---

**最終更新**: 2025年8月17日  
**実装ベース**: 35命令完全対応  
**次期計画**: BID統合プラグインシステム（Phase 9.75f-BID）