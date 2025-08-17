# 🤖 Nyash MIR 26命令仕様書 (ChatGPT5設計版)

*Everything is Box哲学・完璧なIR化実現 - 2025年8月17日版*

## 🎯 **概要**

Nyash MIR 26命令は、ChatGPT5 + AI大会議により設計された、「化け物に伸びる余白」と「実装の現実」の最適バランスを実現する中間表現です。

### **🌟 設計思想**
- **RISC原則**: 直交性・シンプル性重視
- **階層化設計**: Tier-0/1/2による段階的実装
- **Everything is Box**: Box中心のセマンティクス
- **Effect System**: 最適化安全性の確保
- **所有権森**: メモリ安全性の言語レベル保証

## 🏗️ **26命令完全仕様**

### **Tier-0: 普遍的コア (8命令)**
コンパイラ・仮想マシンの基盤となる必須命令

| 命令 | 形式 | 効果 | 説明 |
|------|------|------|------|
| **Const** | `%dst = const value` | pure | 定数値生成 |
| **BinOp** | `%dst = %lhs op %rhs` | pure | 二項演算（+,-,*,/,==,!=,<,>,and,or等） |
| **Compare** | `%dst = %lhs cmp %rhs` | pure | 比較演算（専用最適化用） |
| **Branch** | `br %cond -> %then, %else` | control | 条件分岐 |
| **Jump** | `jmp %target` | control | 無条件ジャンプ |
| **Phi** | `%dst = phi [%val1:%bb1, %val2:%bb2]` | pure | SSA φ関数 |
| **Call** | `%dst = call %func(%args...)` | context | 関数呼び出し |
| **Return** | `ret %value?` | control | 関数戻り |

### **Tier-1: Nyashセマンティクス (13命令)**
Everything is Box哲学の核心実装

| 命令 | 形式 | 効果 | 説明 |
|------|------|------|------|
| **NewBox** | `%dst = new_box "Type"(%args...)` | mut | 強所有Box生成（所有森ノード） |
| **BoxFieldLoad** | `%dst = %box.field` | pure | Boxフィールド読み取り |
| **BoxFieldStore** | `%box.field = %value` | mut | Boxフィールド書き込み |
| **BoxCall** | `%dst = %box.method(%args...)` | context | Boxメソッド呼び出し |
| **ExternCall** | `%dst = extern %iface.method(%args...)` | context | 外部ライブラリ呼び出し |
| **Safepoint** | `safepoint` | io | 分割fini・割込み許可ポイント |
| **RefGet** | `%dst = ref_get %ref` | pure | 参照から値取得 |
| **RefSet** | `ref_set %ref = %value` | mut | 参照先差し替え（所有規則検証付き） |
| **WeakNew** | `%dst = weak_new %box` | pure | weak参照生成 |
| **WeakLoad** | `%dst = weak_load %weak` | pure | weak参照から値取得（失効時null） |
| **WeakCheck** | `%dst = weak_check %weak` | pure | weak参照生存確認 |
| **Send** | `send %data -> %target` | io | Bus送信 |
| **Recv** | `%dst = recv %source` | io | Bus受信 |

### **Tier-2: 実装補助・最適化友好 (5命令)**
JIT/AOT最適化の基盤

| 命令 | 形式 | 効果 | 説明 |
|------|------|------|------|
| **TailCall** | `tail_call %func(%args...)` | control | 末尾呼び出し最適化 |
| **Adopt** | `adopt %parent <- %child` | mut | 所有権移管（親が子を取り込み） |
| **Release** | `release %ref` | mut | 強所有解除（weak化/null化） |
| **MemCopy** | `memcopy %dst <- %src, %size` | mut | 小規模メモリ移動最適化 |
| **AtomicFence** | `atomic_fence %ordering` | io | 並行時順序保証 |

## 🎭 **Effect System - 最適化基盤**

### **効果分類と最適化ルール**

#### **Pure効果 (8命令)**
```
Const, BinOp, Compare, Phi, BoxFieldLoad, RefGet, WeakNew, WeakLoad, WeakCheck
```
- ✅ **再順序化可能**: 副作用なし
- ✅ **共通部分式除去**: 同一計算結果再利用
- ✅ **不変コード移動**: ループ外移動可能

#### **Mut効果 (5命令)**
```
NewBox, BoxFieldStore, RefSet, Adopt, Release, MemCopy
```
- ⚠️ **同一リソース順序保持**: 同じBox/同じFieldで依存関係維持
- ✅ **異なるリソース並列化**: 別Box操作は並列実行可能

#### **Io効果 (4命令)**
```
Safepoint, Send, Recv, AtomicFence
```
- 🔒 **順序保持必須**: プログラム順序で実行
- ❌ **再順序化禁止**: 副作用の整合性確保

#### **Control効果 (4命令)**
```
Branch, Jump, Return, TailCall
```
- 🌊 **制御フロー変更**: 基本ブロック境界制御
- 📊 **静的解析対象**: CFG構築・到達可能性解析

#### **Context依存効果 (3命令)**
```
Call, BoxCall, ExternCall
```
- 🔄 **呼び出し先依存**: 関数・メソッドの効果を継承
- 📝 **BID/型情報**: ExternCallはBID仕様から効果決定

## 🔧 **所有権森システム**

### **強参照森 (Ownership Forest)**
```rust
// 基本原則: strong in-degree ≤ 1
%parent = NewBox "Parent"()
%child = NewBox "Child"()
Adopt %parent <- %child    // 子を親の強所有に移管
```

#### **検証ルール**
- ✅ **DAG構造保証**: 強参照による循環禁止
- ✅ **単一所有**: 各Boxは最大1つの強参照のみ
- ✅ **所有移管**: Adopt/Releaseによる安全な移転

### **weak参照システム**
```rust
%weak = WeakNew %box       // weak参照生成
%alive = WeakCheck %weak   // 生存確認 (bool)
%value = WeakLoad %weak    // 値取得 (失効時null)
```

#### **決定的挙動**
- 🎯 **失効時null**: WeakLoadは例外なしでnull返却
- 🎯 **失効時false**: WeakCheckは例外なしでfalse返却
- 🔒 **所有権なし**: weakは削除を阻止しない

## 🚀 **削減戦略 - 35命令からの移行**

### **削除対象命令 (17命令)**

#### **BinOpに統合**
- `UnaryOp` → `BinOp`（not %a → %a xor true）

#### **BoxField操作に統合**
- `Load/Store` → `BoxFieldLoad/BoxFieldStore`
- `ArrayGet/ArraySet` → `BoxFieldLoad/BoxFieldStore`（配列もBoxのフィールド）

#### **intrinsic化**
```rust
// 削除前
Print %value
Debug %value "message"

// 削除後（intrinsic化）
Call @print, %value
Call @debug, %value, "message"
```

#### **完全削除**
- `Copy, Nop` → 最適化パス専用（MIRから除外）
- `TypeCheck, Cast` → 型システム・最適化で処理
- `Throw/Catch` → Call経由例外ハンドリング

#### **統合・置換**
- `RefNew` → 削除（RefGetで代用）
- `BarrierRead/BarrierWrite` → `AtomicFence`統合
- `FutureNew/FutureSet/Await` → `NewBox + BoxCall`実装

### **新規追加命令 (10命令)**

#### **Box操作の明示化**
- `BoxFieldLoad/BoxFieldStore` → Everything is Box核心
- `Adopt/Release` → 所有権移管の明示

#### **弱参照完全対応**
- `WeakCheck` → 生存確認の明示
- `Send/Recv` → Bus操作の一次市民化

#### **最適化基盤**
- `TailCall, MemCopy, AtomicFence` → JIT/AOT準備

## 📊 **段階的移行戦略**

### **Phase 1: 共存実装 (1週間)**
- 新旧命令両対応のMIRパーサー実装
- `BoxFieldLoad/BoxFieldStore`等の新命令追加
- 既存命令は保持したまま新形式も受け入れ

### **Phase 2: フロントエンド移行 (1週間)**
- AST→MIR生成を新形式のみに変更
- `Load/Store`の代わりに`BoxFieldLoad/BoxFieldStore`生成
- intrinsic化対象は`Call @intrinsic_name`形式で生成

### **Phase 3: 最適化パス移行 (1週間)**
- 全最適化パスを新命令対応に修正
- Effect分類の正確な実装
- 所有権森検証ルール実装

### **Phase 4: バックエンド移行 (1週間)**
- Interpreter/VM/WASMの新命令対応
- intrinsic関数の実装（@print, @debug等）
- 削除予定命令の段階的無効化

### **Phase 5: 旧命令削除 (1週間)**
- 削除対象17命令の完全除去
- テストスイート更新
- ドキュメント整備

## 🧪 **検証・テスト戦略**

### **Golden MIR テスト**
```bash
# 全バックエンドでMIR出力一致確認
./target/release/nyash --dump-mir-26 program.nyash > golden.mir
./target/release/nyash --backend vm --dump-mir-26 program.nyash > vm.mir
./target/release/nyash --backend wasm --dump-mir-26 program.nyash > wasm.mir
diff golden.mir vm.mir && diff vm.mir wasm.mir
```

### **所有権森検証**
```rust
// 検証項目
fn verify_ownership_forest(mir: &MirModule) -> Result<(), VerifyError> {
    // 1. strong in-degree ≤ 1
    verify_strong_indegree_constraint()?;
    // 2. 強循環禁止（DAG構造）
    verify_no_strong_cycles()?;
    // 3. WeakLoad/WeakCheck決定的挙動
    verify_weak_determinism()?;
}
```

### **Effect System検証**
```rust
// 最適化安全性確認
fn verify_effect_system(mir: &MirModule) -> Result<(), VerifyError> {
    // Pure命令の再順序化安全性
    verify_pure_reordering_safety()?;
    // Mut命令の依存関係保持
    verify_mut_dependency_preservation()?;
    // Io命令の順序保証
    verify_io_order_preservation()?;
}
```

## 🎯 **成功基準**

### **必須基準**
- [ ] **26命令完全実装**: 全命令が仕様通り動作
- [ ] **Effect System動作**: 4種効果の正確な分類・最適化
- [ ] **所有権森検証**: 強参照森・weak参照の安全性保証
- [ ] **Golden MIRテスト**: 全バックエンドでMIR一致
- [ ] **性能維持**: 削減後も性能劣化なし

### **理想基準**
- [ ] **最適化効果実証**: pure再順序化・CSE/LICM動作確認
- [ ] **所有権森活用**: Adopt/Releaseによる効率的メモリ管理
- [ ] **JIT準備完了**: TailCall/MemCopyの最適化基盤確立

## 📚 **関連ドキュメント**

- **ChatGPT5仕様**: `docs/予定/native-plan/copilot_issues_phase0_to_94.txt`
- **実装移行計画**: `docs/予定/native-plan/issues/phase_8_5_mir_35_to_26_reduction.md`
- **Effect System詳細**: `docs/nyir/effect-system-specification.md`

---

**策定**: ChatGPT5 + AI大会議  
**Gemini評価**: 「極めて健全」「断行推奨」  
**実装目標**: 2025年9月完了予定