# Rust VM vs Hakorune VM ギャップ分析と実装計画

**分析日**: 2025-10-10
**ソース**: mini_vm_progress.md, INSTRUCTION_SET.md, Rust VM handlers, Hakorune VM handlers

---

## 📊 Executive Summary

### 現状まとめ

| メトリクス | Rust VM | Hakorune VM | ギャップ |
|-----------|---------|-------------|---------|
| **基本命令実装** | 16/16 (100%) | 16/16 (100%) | **なし** ✅ |
| **MirCall Phase 1** | ✅ 完全実装 | ✅ 完全実装 (Global + Extern) | **なし** ✅ |
| **BoxCall** | ✅ 完全実装 | ✅ 実装済み（一部問題あり） | **ArrayBox/MapBox問題** ⚠️ |
| **Collection API** | ✅ 完全動作 | ⚠️ StringBox のみ完全動作 | **ArrayBox/MapBox参照保持** ⚠️ |
| **MirCall Phase 2** | ✅ 完全実装 | ❌ 未実装 | **Method/ModuleFunction** ❌ |

### 重要な発見

✅ **良い発見**:
- Hakorune VM は 16/16 命令（100%）を実装済み！
- Phase 1（基本演算・制御フロー・GC命令）は完全実装
- MirCall Phase 1（Global + Extern）は完全動作
- TypeOp は実装済み（簡易版）

⚠️ **既知の問題**:
1. **ArrayBox/MapBox 参照保持問題**（Phase 4 Day 11 で発見）
   - push() 後に size() が 0 を返す問題
   - Selfhost VM（Hakoruneスクリプト）⇔ Rust VM 連携の問題
   - 調査中（Task Teacher で根本原因特定中）

2. **Rust VM print() バグ**（Phase 4 Day 11 で発見）
   - `print("size=" + obj.size())` でバグ
   - 回避策: `local s = obj.size(); print("size=" + s)` で回避済み

❌ **未実装機能**:
- **MirCall Phase 2**: Method/ModuleFunction/Constructor 実装（最重要）
  - Rust VM では完全実装済み
  - Hakorune VM では未実装
  - Selfhost compiler 完全動作に必須

---

## 1. 機能比較表（詳細版）

### 1.1 基本演算（5命令）

| 命令 | Rust VM | Hakorune VM | 実装ファイル | 複雑度 | 状態 |
|------|---------|-------------|-------------|--------|------|
| **Const** | ✅ | ✅ | const_handler.hako (67行) | 低 | **完全動作** ✅ |
| **BinOp** | ✅ | ✅ | binop_handler.hako (70行) | 低 | **完全動作** ✅ |
| **UnaryOp** | ✅ | ✅ | unaryop_handler.hako (63行) | 低 | **完全動作** ✅ |
| **Compare** | ✅ | ✅ | compare_handler.hako (77行) | 低 | **完全動作** ✅ |
| **TypeOp** | ✅ | ✅ | typeop_handler.hako (60行) | **中** | **簡易実装** ⚠️ |

**TypeOp 詳細**:
- **Hakorune VM 実装**: 簡易版（Check=1固定、Cast=copy）
- **Rust VM 実装**: （要調査）
- **ギャップ**: 実行時型チェック/変換が未実装
- **影響**: 現在のテストケースでは問題なし
- **優先度**: **低**（Phase 2以降で改善）

---

### 1.2 メモリ操作（2命令）

| 命令 | Rust VM | Hakorune VM | 実装ファイル | 複雑度 | 状態 |
|------|---------|-------------|-------------|--------|------|
| **Load** | ✅ | ✅ | load_handler.hako (44行) | 低 | **完全動作** ✅ |
| **Store** | ✅ | ✅ | store_handler.hako (42行) | 低 | **完全動作** ✅ |

---

### 1.3 制御フロー（4命令）

| 命令 | Rust VM | Hakorune VM | 実装ファイル | 複雑度 | 状態 |
|------|---------|-------------|-------------|--------|------|
| **Branch** | ✅ | ✅ | terminator_handler.hako (208行) | 中 | **完全動作** ✅ |
| **Jump** | ✅ | ✅ | terminator_handler.hako | 低 | **完全動作** ✅ |
| **Phi** | ✅ | ✅ | phi_handler.hako (223行) | **高** | **完全動作** ✅ |
| **Return** | ✅ | ✅ | terminator_handler.hako | 低 | **完全動作** ✅ |

**Phi 詳細**:
- **Hakorune VM 実装**: Rust VM のロジックを忠実に移植
- **複雑度**: 高（predecessor tracking が必要）
- **動作確認**: 5/5 テスト PASS（Phase 1 Day 3）

---

### 1.4 GC関連（3命令）

| 命令 | Rust VM | Hakorune VM | 実装ファイル | 複雑度 | 状態 |
|------|---------|-------------|-------------|--------|------|
| **Barrier** | ✅ | ✅ | barrier_handler.hako (19行) | 低 | **Nop実装** ✅ |
| **Safepoint** | ✅ | ✅ | safepoint_handler.hako (19行) | 低 | **Nop実装** ✅ |
| **Nop** | ✅ | ✅ | nop_handler.hako (19行) | 低 | **完全動作** ✅ |

**注**: GC関連命令は現状 Nop 実装（将来的に GC 対応時に拡張）

---

### 1.5 呼び出し命令（最重要）⭐

| 命令/機能 | Rust VM | Hakorune VM | 実装ファイル | 複雑度 | 状態 |
|----------|---------|-------------|-------------|--------|------|
| **MirCall Phase 1** | ✅ | ✅ | mircall_handler.hako (88行) | 中 | **Global/Extern のみ** ✅ |
| **MirCall Phase 2** | ✅ | ❌ | - | **高** | **未実装** ❌ |
| **BoxCall** | ✅ | ⚠️ | boxcall_handler.hako (145行) | 中 | **一部問題** ⚠️ |
| **NewBox** | ✅ | ✅ | newbox_handler.hako (51行) | 低 | **完全動作** ✅ |
| **Copy** | ✅ | ✅ | copy_handler.hako (29行) | 低 | **完全動作** ✅ |

---

## 2. 未実装機能の詳細分析

### 2.1 MirCall Phase 2（Method/ModuleFunction）⭐最重要

#### 📊 概要

**目的**: Selfhost compiler 完全動作に必須

**Callee 種別**:
```rust
pub enum Callee {
    Global(String),              // ✅ Phase 1 完了
    Extern(String),              // ✅ Phase 1 完了
    ModuleFunction(String),      // ❌ Phase 2 未実装
    Method {                     // ❌ Phase 2 未実装
        box_name: String,
        method: String,
        receiver: Option<ValueId>,
        certainty: TypeCertainty,
    },
    Constructor { box_type: String },  // ❌ Phase 2 未実装
    Closure { ... },             // ❌ Phase 2 未実装（優先度低）
    Value(ValueId),              // ❌ Phase 2 未実装（優先度低）
}
```

---

#### 2.1.1 ModuleFunction 実装

**Rust VM 実装**:
- **ファイル**: `src/backend/mir_interpreter/handlers/calls/function.rs`
- **関数**: `handle_callee_module_function()`
- **行数**: ~170行（284-454行）
- **複雑度**: **高**

**実装の要点**:
1. **関数テーブル検索**: `self.functions.get(&want_name)`
2. **引数値の読み込み**: `self.reg_load(*a)?` でレジスタから値取得
3. **birth() 特殊処理**:
   - 冪等性チェック（`contracts_born.contains(&key)`）
   - 再入チェック（`contracts_in_birth.insert(key)`）
   - 成功時に `lifecycle_contracts_birth()` 呼び出し
4. **Tail-based fallback**: "Class.method/N" 形式の名前解決
5. **Builtin vtable bridge**: ArrayBox/MapBox/StringBox → BoxCall へ委譲

**Hakorune VM 移植時の課題**:
1. ❌ **関数テーブルなし**: Selfhost VM には MIR 関数テーブルがない
2. ❌ **birth() 特殊処理**: 冪等性/再入チェックの実装が必要
3. ❌ **Rust VM 呼び出しブリッジ**: Selfhost VM から Rust VM 関数を呼ぶ仕組み
4. ⚠️ **名前解決の複雑さ**: Tail-based fallback の実装

**解決策**:
- **Option A（推奨）**: Rust VM への委譲
  - Selfhost VM は MirCall JSON を Rust VM に渡す
  - Rust VM が関数実行して結果を返す
  - 実装量: ~50行（ブリッジのみ）

- **Option B**: Selfhost VM 内で完全実装
  - 関数テーブルを JSON として渡す（functions: {name: MirFunction}）
  - Selfhost VM が MIR を解釈して実行
  - 実装量: ~300行（関数実行エンジン）

**工数見積もり**:
- **Option A**: 2-3人日（ブリッジ実装 + テスト）
- **Option B**: 8-12人日（完全実装 + テスト）

**推奨**: **Option A**（Rust VM 委譲）
- 理由: Selfhost compiler の目的は「MIR 生成」であり、「MIR 実行」は Rust VM に任せるべき
- 影響: Selfhost VM は「MIR 生成器」として機能すれば十分

---

#### 2.1.2 Method 実装

**Rust VM 実装**:
- **ファイル**: `src/backend/mir_interpreter/handlers/calls/method.rs`
- **関数**: `handle_callee_method()`
- **行数**: 要調査
- **複雑度**: **中**

**実装の要点**:
1. **receiver の型取得**: `self.reg_load(receiver)?`
2. **メソッドディスパッチ**: box_name + method でテーブル検索
3. **TypeCertainty 処理**: Known/Unknown で分岐

**Hakorune VM 移植時の課題**:
1. ❌ **メソッドテーブルなし**: Selfhost VM には Box メソッドテーブルがない
2. ⚠️ **動的型取得**: receiver の型を実行時に判定する必要

**解決策**:
- **Option A（推奨）**: Rust VM への委譲（ModuleFunction と同様）
- **Option B**: BoxCall への変換
  - Method → BoxCall に変換して既存実装を利用
  - 実装量: ~30行

**工数見積もり**:
- **Option A**: 1人日（ModuleFunction と共通化）
- **Option B**: 1-2人日（変換ロジック + テスト）

---

#### 2.1.3 Constructor 実装

**Rust VM 実装**:
- **ファイル**: `src/backend/mir_interpreter/handlers/mod.rs`
- **関数**: `handle_new_box()`（NewBox 命令と共通）
- **行数**: ~20行（17-38行）
- **複雑度**: **低**

**実装の要点**:
1. **Box インスタンス生成**: `new BoxType()`
2. **auto_birth 処理**: birth() メソッド自動呼び出し
3. **Fallback birth**: `handle_box_call(None, *dst, "birth", args)`

**Hakorune VM 移植時の課題**:
1. ✅ **既に実装済み**: NewBoxHandlerBox が同等機能を提供
2. ⚠️ **auto_birth 未実装**: birth() 自動呼び出しがない

**解決策**:
- NewBoxHandlerBox に auto_birth 処理を追加
- 実装量: ~10行

**工数見積もり**: 0.5人日（既存拡張のみ）

---

### 2.2 BoxCall 問題修正（ArrayBox/MapBox）⚠️

#### 📊 現状

**問題**: ArrayBox.push() 後に size() が 0 を返す

**発見**: Phase 4 Day 11（2025-10-09）

**影響範囲**:
- ❌ ArrayBox: push/size/isEmpty テスト失敗（5/9 テスト）
- ❌ MapBox: size/isEmpty/keys テスト失敗（5/9 テスト）
- ✅ StringBox: 全テスト成功（4/9 テスト）

**成功率**: 4/9 (44%)

---

#### 🐛 根本原因（仮説）

**仮説1**: ValueManagerBox.get() が毎回別インスタンスを返す
- **検証結果**: ❌ 却下（MapBox.set/get は正しく参照を保存）

**仮説2**: Rust VM print() バグ
- **検証結果**: ✅ 確認済み（`print("size=" + obj.size())` で失敗、外で呼ぶと成功）
- **回避策**: デバッグトレースを修正済み

**仮説3**: Selfhost VM 内部で ArrayBox インスタンスが複製される
- **検証結果**: 🔍 調査中（Task Teacher で根本原因特定中）
- **関連**: ChatGPT Legacy Removal（boxes_*.rs削除）の影響？

---

#### 🔧 解決策

**Phase 4 Day 12 計画**:
1. **Task Teacher 調査**: ArrayBox/MapBox 問題の根本原因特定（2-3時間）
2. **Rust VM 修正**: 必要に応じて boxes_*.rs 復元（1-2時間）
3. **テスト再実行**: Collection API 全テスト（1時間）

**工数見積もり**: 4-6時間（調査 2-3h + 修正 1-2h + テスト 1h）

---

## 3. 実装計画

### Phase 1: 必須実装（最優先）⭐

#### 対象機能

1. **MirCall Phase 2: ModuleFunction 実装**
2. **MirCall Phase 2: Method 実装**
3. **ArrayBox/MapBox 問題修正**

---

#### 3.1 実装順序と根拠

**Step 1: ArrayBox/MapBox 問題修正**（見積もり: 0.5-1人日）

**理由**:
- BoxCall の基盤が壊れていると MirCall Phase 2 の検証ができない
- 既知の問題（Phase 4 Day 11 で発見）を先に解決すべき

**依存**: なし

**成果物**:
- ArrayBox/MapBox BoxCall テスト 9/9 PASS
- Collection API 完全動作

**開始条件**: 即座に開始可能

---

**Step 2: MirCall Phase 2 - ModuleFunction 実装**（見積もり: 2-3人日）

**理由**:
- Selfhost compiler の関数呼び出しに必須
- Method よりも使用頻度が高い
- Rust VM 委譲アプローチで実装量を削減

**依存**: なし（BoxCall 問題とは独立）

**成果物**:
- ModuleFunction 呼び出し動作
- Selfhost compiler の基本機能が動作

**実装アプローチ**: **Option A（Rust VM 委譲）**
- Selfhost VM は MirCall JSON を Rust VM に渡す
- Rust VM が関数実行して結果を返す
- 実装量: ~50行（ブリッジのみ）

---

**Step 3: MirCall Phase 2 - Method 実装**（見積もり: 1-2人日）

**理由**:
- ModuleFunction の仕組みを Method にも適用
- Rust VM 委譲で共通化可能

**依存**: ModuleFunction 実装完了

**成果物**:
- Method 呼び出し動作
- Selfhost compiler の Box メソッド呼び出しが動作

**実装アプローチ**: **Option A（Rust VM 委譲）**
- ModuleFunction と共通のブリッジを利用

---

#### 3.2 マイルストーン

| マイルストーン | 期間 | 累計 | 成果 |
|--------------|------|------|------|
| **Week 1** | 0.5-1人日 | 0.5-1人日 | ArrayBox/MapBox 問題修正完了 |
| **Week 2** | 2-3人日 | 2.5-4人日 | ModuleFunction 実装完了 |
| **Week 3** | 1-2人日 | 3.5-6人日 | Method 実装完了 |
| **合計** | **3.5-6人日** | - | **MirCall Phase 2 完全実装** ✅ |

---

### Phase 2: 重要実装

#### 対象機能

1. **TypeOp 改善**（実行時型チェック/変換）
2. **Constructor 実装**（auto_birth 処理）

---

#### 実装順序

**Step 1: Constructor 実装**（見積もり: 0.5人日）

**理由**:
- NewBox 拡張のみで実装可能
- auto_birth 処理は Selfhost compiler で必要

**依存**: なし

**成果物**:
- NewBox に auto_birth 処理追加
- Constructor 呼び出し動作

---

**Step 2: TypeOp 改善**（見積もり: 1-2人日）

**理由**:
- 現状は簡易実装（Check=1固定、Cast=copy）
- 実行時型チェック/変換の完全実装

**依存**: なし（独立機能）

**成果物**:
- TypeOp 完全実装
- 実行時型安全性の向上

---

#### マイルストーン

| マイルストーン | 期間 | 累計 | 成果 |
|--------------|------|------|------|
| **Week 4** | 0.5人日 | 0.5人日 | Constructor 実装完了 |
| **Week 5** | 1-2人日 | 1.5-2.5人日 | TypeOp 改善完了 |
| **合計** | **1.5-2.5人日** | - | **Phase 2 完了** ✅ |

---

### Phase 3: オプション実装（優先度低）

#### 対象機能

1. **Closure 実装**（MirCall Phase 2）
2. **Value 実装**（MirCall Phase 2）
3. **GC 実装**（Barrier/Safepoint の実際のGC処理）

---

#### 実装順序

**将来的に実装**（Selfhost compiler Phase 1 では不要）

---

## 4. 工数見積もりサマリー

### 4.1 Phase別見積もり

| Phase | 機能 | 見積もり | 累計 | 優先度 |
|-------|------|---------|------|--------|
| **Phase 1** | ArrayBox/MapBox 修正 | 0.5-1人日 | 0.5-1人日 | **最優先** ⭐ |
| **Phase 1** | ModuleFunction | 2-3人日 | 2.5-4人日 | **最優先** ⭐ |
| **Phase 1** | Method | 1-2人日 | 3.5-6人日 | **最優先** ⭐ |
| **Phase 2** | Constructor | 0.5人日 | 4-6.5人日 | 高 |
| **Phase 2** | TypeOp 改善 | 1-2人日 | 5-8.5人日 | 中 |
| **Phase 3** | Closure/Value/GC | TBD | - | 低 |
| **合計** | **Phase 1+2** | **5-8.5人日** | - | - |

---

### 4.2 Critical Path（最短経路）

**最優先パス**: Phase 1（MirCall Phase 2 実装）

**理由**:
- Selfhost compiler 完全動作に必須
- ArrayBox/MapBox 問題を先に解決
- ModuleFunction → Method の順で実装

**最短期間**: **3.5人日**（楽観的見積もり）
**現実的期間**: **6人日**（保守的見積もり）

---

## 5. リスク評価

### 5.1 技術的リスク

#### 高リスク

**リスク1: ArrayBox/MapBox 問題の根本原因が不明**

**詳細**:
- Selfhost VM（Hakoruneスクリプト）⇔ Rust VM 連携の問題
- ChatGPT Legacy Removal（boxes_*.rs削除）の影響？
- 2レイヤーVM連携の複雑さ

**影響**: MirCall Phase 2 の検証ができない

**対策**:
- Task Teacher で根本原因特定（2-3時間）
- 最悪の場合、boxes_*.rs 復元（1-2時間）
- Rust VM print() バグは回避策あり（既に対応済み）

**確率**: 中（50%）

---

**リスク2: ModuleFunction 実装の複雑さ**

**詳細**:
- Rust VM への委譲ブリッジの実装
- 関数テーブルの JSON 形式渡し
- birth() 特殊処理の実装

**影響**: 見積もりの 2-3 倍の時間がかかる可能性

**対策**:
- Option A（Rust VM 委譲）で実装量を削減
- 段階的実装（最小機能 → 拡張）
- 早期にプロトタイプ実装してリスクを確認

**確率**: 中（40%）

---

#### 中リスク

**リスク3: テストカバレッジ不足**

**詳細**:
- MirCall Phase 2 のテストケースが不足
- Edge case の見落とし

**影響**: 本番環境でバグ発見

**対策**:
- 各機能で 10 件以上のテストケース作成
- Rust VM のテストケースを参照
- 段階的にテストを追加

**確率**: 低（20%）

---

#### 低リスク

**リスク4: TypeOp 改善の優先度誤認**

**詳細**:
- TypeOp は現状の簡易実装で十分な可能性
- 実行時型チェック/変換の必要性が不明

**影響**: 無駄な実装工数

**対策**:
- Selfhost compiler Phase 1 で TypeOp の使用状況を確認
- 必要になってから実装（Phase 2）

**確率**: 低（10%）

---

### 5.2 スケジュールリスク

**リスク**: ModuleFunction + Method が予想より複雑

**シナリオ**:
- 楽観的: 3.5人日で完了
- 現実的: 6人日で完了
- 悲観的: 10人日（見積もりの 1.7 倍）

**対策**:
- 早期にプロトタイプ実装（Week 1）
- 段階的にテストケースを追加
- 問題発見時は Option B（完全実装）への切り替えを検討

---

### 5.3 品質リスク

**リスク**: 既存実装との整合性

**詳細**:
- Rust VM と Hakorune VM の動作が異なる
- Selfhost compiler が想定外の MIR を生成

**影響**: デバッグが困難、品質低下

**対策**:
- Rust VM の実装を忠実に移植（Option A）
- テストケースで動作を検証（各機能 10 件以上）
- 差分があれば Issue として記録

---

## 6. 推奨アクション

### 6.1 即座に開始すべき

**アクション1: ArrayBox/MapBox 問題修正**（Phase 1, Week 1）

**内容**:
- Task Teacher で根本原因特定
- Rust VM 修正（必要に応じて）
- Collection API 全テスト再実行

**期間**: 0.5-1人日

**成果**:
- BoxCall 完全動作
- Collection API 9/9 テスト PASS

---

**アクション2: ModuleFunction 実装開始**（Phase 1, Week 2）

**内容**:
- Rust VM 委譲ブリッジの設計
- 最小プロトタイプ実装
- テストケース 10 件作成

**期間**: 2-3人日

**成果**:
- ModuleFunction 呼び出し動作
- Selfhost compiler の基本機能が動作

---

### 6.2 次に実施

**アクション3: Method 実装**（Phase 1, Week 3）

**内容**:
- ModuleFunction ブリッジを Method に拡張
- テストケース 10 件作成

**期間**: 1-2人日

**成果**:
- Method 呼び出し動作
- MirCall Phase 2 完全実装

---

### 6.3 その後

**アクション4: Constructor 実装**（Phase 2, Week 4）

**内容**:
- NewBox に auto_birth 処理追加

**期間**: 0.5人日

---

**アクション5: TypeOp 改善**（Phase 2, Week 5）

**内容**:
- 実行時型チェック/変換の完全実装
- Selfhost compiler での必要性を確認してから実施

**期間**: 1-2人日

---

### 6.4 将来的に

**アクション6: Closure/Value/GC 実装**（Phase 3）

**内容**:
- Selfhost compiler Phase 2 以降で必要になったら実装

**期間**: TBD

---

## 7. 成功の定義

### Phase 1 完了時（最重要）⭐

✅ **ArrayBox/MapBox 問題修正完了**:
- Collection API 全テスト 9/9 PASS
- BoxCall 完全動作

✅ **ModuleFunction 実装完了**:
- テスト 10 件以上、成功率 100%
- Selfhost compiler の基本機能が動作

✅ **Method 実装完了**:
- テスト 10 件以上、成功率 100%
- Selfhost compiler の Box メソッド呼び出しが動作

✅ **MirCall Phase 2 完全実装**:
- Global/Extern/ModuleFunction/Method すべて動作
- Selfhost compiler 完全動作に必須機能が揃う

---

### Phase 2 完了時

✅ **Constructor 実装完了**:
- NewBox に auto_birth 処理追加
- Constructor 呼び出し動作

✅ **TypeOp 改善完了**:
- 実行時型チェック/変換の完全実装
- テスト 5 件以上、成功率 100%

---

### Phase 3 完了時（将来）

✅ **Closure/Value/GC 実装完了**:
- すべての MirCall Callee が動作
- GC 関連命令が実際の GC 処理を実行
- Rust VM と完全同等

---

## 8. 次のステップ

### 今すぐ実施 ⚡

**Step 1: ArrayBox/MapBox 問題修正**
- Task Teacher 調査開始
- 根本原因特定
- 修正実装

**開始しますか？** → **YES** ✅

---

**Step 2: ModuleFunction 実装準備**
- Rust VM の handle_callee_module_function() 詳細調査
- ブリッジ設計書作成
- テストケース設計

**開始しますか？** → **ArrayBox/MapBox 修正後** ⏳

---

### 質問・確認事項

**Q1**: ArrayBox/MapBox 問題の優先度は？
**A1**: **最優先**（BoxCall の基盤が壊れているため）

**Q2**: ModuleFunction 実装アプローチは？
**A2**: **Option A（Rust VM 委譲）**を推奨（実装量 ~50行、2-3人日）

**Q3**: Selfhost compiler Phase 1 で必須な機能は？
**A3**: **MirCall Phase 2（ModuleFunction + Method）**のみ

**Q4**: Phase 1 完了までの期間は？
**A4**: **楽観的 3.5人日、現実的 6人日**

---

## 9. まとめ

### 現状まとめ

✅ **良い点**:
- Hakorune VM は 16/16 命令（100%）を実装済み
- Phase 1（基本演算・制御フロー・GC）は完全動作
- MirCall Phase 1（Global + Extern）は完全動作
- 箱化モジュール化により保守性が高い

⚠️ **既知の問題**:
- ArrayBox/MapBox 参照保持問題（調査中）
- Rust VM print() バグ（回避策あり）

❌ **未実装機能**:
- MirCall Phase 2（ModuleFunction + Method）
- Selfhost compiler 完全動作に必須

---

### 推奨実装順序

1. **ArrayBox/MapBox 問題修正**（0.5-1人日）⭐最優先
2. **ModuleFunction 実装**（2-3人日）⭐最優先
3. **Method 実装**（1-2人日）⭐最優先
4. **Constructor 実装**（0.5人日）
5. **TypeOp 改善**（1-2人日）

**合計**: **5-8.5人日**（Phase 1+2）

---

### Critical Path

**最優先パス**: Phase 1（MirCall Phase 2）
**最短期間**: **3.5人日**（楽観的）
**現実的期間**: **6人日**（保守的）

---

### 次のアクション

**今すぐ**: ArrayBox/MapBox 問題修正（0.5-1人日）
**次**: ModuleFunction 実装（2-3人日）
**その後**: Method 実装（1-2人日）

**開始しますか？** → **YES** ✅

---

## 付録: 参考資料

- **mini_vm_progress.md**: Hakorune VM 開発進捗（Phase 1-4 Day 11）
- **INSTRUCTION_SET.md**: MIR 命令セット仕様
- **Rust VM handlers**: `src/backend/mir_interpreter/handlers/`
- **Hakorune VM handlers**: `apps/selfhost/hakorune-vm/*.hako`

---

**分析完了日**: 2025-10-10
**次回更新**: Phase 1 完了時
