# Rust → Hakorune 移行計画（詳細版）

**作成日**: 2025-10-06
**目的**: Rustで実装された Hakorune 処理系を段階的に Hakorune 言語自身で置き換える

---

## 📊 現状分析（2025-10-06）

### 🔍 コードベース規模

#### **Rust実装（src/）**
- **合計**: 約94,356行
- **ファイル数**: 636ファイル（.rs）
- **主要コンポーネント**:
  - Parser: 8ファイル（トップレベル）
  - MIR Builder: 20ファイル（トップレベル）
  - Backend: 4ファイル（トップレベル）
  - Runtime: 28ファイル（トップレベル）

**最大ファイル**（行数順）:
1. `src/backend/wasm/codegen.rs` - 899行
2. `src/runner/mir_json_emit.rs` - 867行
3. `src/config/env.rs` - 849行
4. `src/mir/builder.rs` - 732行
5. `src/boxes/p2p_box.rs` - 713行

#### **Hakorune実装（apps/）**
- **合計**: 537ファイル（.hako + .nyash）
- **主要ディレクトリ**:
  - `apps/selfhost-compiler/`: 63ファイル、5,653行
  - `apps/selfhost/`: 63ファイル
  - `apps/hakorune/vm/`: 9ファイル、582行
  - `apps/lib/`: 58ファイル

**Hakoruneコンパイラの実装状況**:
- ✅ Parser MVP（Stage-2/3サポート）
- ✅ MIR生成基本（const/binop/compare/branch/jump/ret）
- ✅ JSON v0出力
- ✅ Pipeline V2（Box-First emit-only architecture）
- ✅ UsingResolverBox/NamespaceBox（完了）

**Hakorune VM の実装状況**:
- ✅ M2/M3基盤（6命令動作実証: const/binop/compare/branch/jump/ret）
- ✅ InstructionScannerBox/OpHandlersBox
- ✅ ProgramStateBox/CfgNavigatorBox/RetResolverBox
- 🔥 残り命令（newbox/boxcall/phi/load/store/externcall）← Phase 15.7で実装中

---

## 🎯 移行戦略

### 📋 基本方針

1. **段階的移行**: 一度にすべて置き換えるのではなく、小さな単位で移行
2. **Box-First原則**: すべての機能をBoxとして分離・固定
3. **相互検証**: Rust実装とHakorune実装を並行動作させ、パリティテスト
4. **Fail-Fast**: 問題があれば即座にRust実装へロールバック
5. **完全なドキュメント化**: 移行の各ステップを記録

### 🔄 移行の3つのフェーズ

```
Phase A: コンパイラ移行（Parser + MIR Builder）
    ↓
Phase B: VM移行（MIR Interpreter）
    ↓
Phase C: ランタイム移行（Runtime + Box実装）
```

---

## 📅 Phase A: コンパイラ移行（優先度: 最高）

**目標**: `apps/selfhost-compiler/` を完全に動作させる

### ✅ **完了済み（85-90%）**

#### **A1: Parser実装**（完了）
- **Rust実装**: `src/parser/` (約8ファイル)
- **Hakorune実装**: `apps/selfhost-compiler/boxes/parser/` (約20ファイル)
- **状態**: ✅ Stage-2/3サポート完了

**実装済みファイル**:
- `parser_box.hako` - メインパーサー
- `parser_expr_box.hako` - 式解析
- `parser_stmt_box.hako` - 文解析
- `parser_control_box.hako` - 制御フロー
- `using_collector_box.hako` - using文収集

#### **A2: MIR Builder基本**（完了）
- **Rust実装**: `src/mir/builder.rs` (732行)
- **Hakorune実装**: `apps/selfhost-compiler/builder/` + `pipeline_v2/`
- **状態**: ✅ 基本命令（6命令）完了

**実装済み命令**:
- const, binop, compare, branch, jump, ret

#### **A3: Using解決**（完了）
- **Rust実装**: `src/mir/resolve/` + `src/using/`
- **Hakorune実装**:
  - `selfhost/compiler/pipeline_v2/using_resolver_box.hako`
  - `selfhost/compiler/pipeline_v2/namespace_box.hako`
- **状態**: ✅ 完了（見積もり12日 → 実績2日、**85%短縮！**）

#### **A4: Pipeline V2基盤**（完了）
- **Hakorune実装**:
  - `selfhost/compiler/pipeline_v2/execution_pipeline_box.hako`
  - `selfhost/compiler/pipeline_v2/backend_box.hako`
  - `selfhost/compiler/pipeline_v2/mir_builder_box.hako`
- **状態**: ✅ Box-First emit-only architecture完成

### 🔥 **残り10-15%（Phase 15.7で完了予定）**

#### **A5: MIR Builder拡張**（一部未完）
**残り命令（優先順位順）**:
1. 🔥 **newbox**（2日） - Box生成（最重要）
2. 🔥 **boxcall**（2日） - メソッド呼び出し（最重要）
3. **phi**（2日） - SSA合流
4. **load/store**（2日） - メモリアクセス
5. **externcall**（1日） - 外部関数呼び出し
6. **unaryop/typeop**（2日） - 単項演算・型操作

**対応するRustファイル**:
- `src/mir/builder/exprs.rs` - 式のMIR変換
- `src/mir/builder/calls/` - 呼び出し系命令

#### **A6: 構文サポート拡張**（Phase 16以降）
**未実装構文**:
- match式完全対応
- property（getter/setter）
- lambda式
- async/await

**対応するRustファイル**:
- `src/parser/items/functions.rs` - 関数定義
- `src/parser/statements/control_flow.rs` - 制御フロー

---

## 📅 Phase B: VM移行（優先度: 高）

**目標**: `apps/hakorune/vm/` を完全に動作させる

### ✅ **完了済み（40-50%）**

#### **B1: VM基盤**（完了）
- **Rust実装**: `src/backend/mir_interpreter/exec.rs` (約637行)
- **Hakorune実装**: `apps/hakorune/vm/boxes/hakorune_vm_min.hako`
- **状態**: ✅ M2/M3基盤完成（6命令動作実証）

**実装済みBox**:
- `InstructionScannerBox` - 命令スキャナ
- `OpHandlersBox` - 命令ハンドラ
- `ProgramStateBox` - プログラム状態管理
- `CfgNavigatorBox` - CFG操作
- `RetResolverBox` - 戻り値解決
- `DiagnosticsBox` - 診断出力

#### **B2: 基本命令**（完了）
**実装済み**:
- const, binop, compare, branch, jump, ret

### 🔥 **残り50-60%（Phase 15.7で完了予定）**

#### **B3: 残り命令実装**（未完）
**Phase 15.7で実装中**:
1. 🔥 **newbox**（2日） - Box生成
2. 🔥 **boxcall**（2日） - メソッド呼び出し
3. **phi**（2日） - SSA合流
4. **load/store**（2日） - フィールドアクセス
5. **externcall**（1日） - print等外部呼び出し
6. **unaryop/typeop**（2日） - 単項演算・型操作

**対応するRustファイル**:
- `src/backend/mir_interpreter/handlers/` - 各命令ハンドラ
- `src/backend/mir_interpreter/helpers.rs` (593行)

#### **B4: 最適化・性能改善**（Phase 16以降）
- メモリ管理最適化
- ホットパス最適化
- トレース・デバッグ機能

---

## 📅 Phase C: ランタイム移行（優先度: 中〜低）

**目標**: `src/runtime/` を Hakorune で置き換える

### 🔄 **現状（5-10%）**

#### **C1: 基本Box実装**（一部完了）
- **Rust実装**: `src/boxes/` (多数のファイル)
- **Hakorune実装**: `apps/lib/boxes/` (4ファイル)

**実装済みBox**:
- ✅ `StringBox` → `apps/lib/boxes/string_std.hako`
- ✅ `ArrayBox` → `apps/lib/boxes/array_std.hako`
- ✅ `MapBox` → `apps/lib/boxes/map_std.hako`
- ✅ `ConsoleBox` → `apps/lib/boxes/console_std.hako`

**未実装Box**（優先順位順）:
1. **IntegerBox** - 整数操作
2. **BoolBox** - ブール操作
3. **FileBox** - ファイルI/O
4. **TimerBox** - 時刻操作（一部完了: nyrt.time.now_ms）
5. **PathBox** - パス操作
6. **RegexBox** - 正規表現
7. **HttpBox** - HTTP通信
8. その他（SocketBox, P2PBox, WebBox等）

#### **C2: ランタイムシステム**（未着手）

**Phase 17以降で検討**:
- GC（ガベージコレクション）
- Plugin System
- Module System
- Type System
- Error Handling

**対応するRustファイル**:
- `src/runtime/gc.rs` (多数行)
- `src/runtime/plugin_loader_v2/` (多数ファイル)
- `src/runtime/modules_registry.rs`
- `src/runtime/type_registry.rs`

---

## 📊 移行優先度マトリックス

| コンポーネント | 優先度 | 状態 | Rust行数 | Hakorune行数 | 見積期間 | 備考 |
|--------------|-------|------|---------|------------|---------|------|
| **Parser** | 🔴 P1 | ✅ 完了 | ~8k行 | ~2k行 | - | Phase 15.7完了 |
| **MIR Builder（基本）** | 🔴 P1 | ✅ 完了 | ~5k行 | ~3k行 | - | Phase 15.7完了 |
| **Using解決** | 🔴 P1 | ✅ 完了 | ~2k行 | ~500行 | - | Phase 15.7完了 |
| **MIR Builder（拡張）** | 🟡 P2 | 🔥 進行中 | ~3k行 | ~1k行 | 2週間 | Phase 15.7で完了予定 |
| **Hakorune VM（基本）** | 🟡 P2 | ✅ 完了 | ~2k行 | ~600行 | - | Phase 15.7完了 |
| **Hakorune VM（拡張）** | 🟡 P2 | 🔥 進行中 | ~3k行 | ~500行 | 1-2週間 | Phase 15.7で完了予定 |
| **基本Box** | 🟢 P3 | 🔄 一部 | ~10k行 | ~1k行 | 4-6週間 | Phase 16-17で計画 |
| **構文拡張** | 🟢 P3 | 📋 計画 | ~5k行 | 未着手 | 4-8週間 | Phase 16以降 |
| **LLVM Backend** | 🟢 P3-P4 | 📋 計画 | ~8k行 | 未着手 | 8-12週間 | Phase 18以降 |
| **WASM Backend** | 🟢 P3-P4 | 📋 計画 | ~5k行 | 未着手 | 6-10週間 | Phase 18以降 |
| **ランタイムシステム** | ⚪ P4-P5 | 📋 計画 | ~30k行 | 未着手 | 16-24週間 | Phase 20以降 |
| **Plugin System** | ⚪ P4-P5 | 📋 計画 | ~10k行 | 未着手 | 8-12週間 | Phase 20以降 |

**凡例**:
- 🔴 P1: 最優先（セルフホスティング必須）
- 🟡 P2: 高優先度（基本機能完成）
- 🟢 P3: 中優先度（機能拡張）
- ⚪ P4-P5: 低優先度（長期計画）

---

## 🎯 マイルストーン

### ✅ **Milestone 1: Hakoruneコンパイラ MVP**（Phase 15.7完了予定）
- **目標**: Hakorune言語で Hakorune をコンパイル可能に
- **期限**: 2025-10-20（予想）
- **進捗**: **85-90%完了**
- **残タスク**:
  - Hakorune VM命令拡張（newbox/boxcall/phi/load/store/externcall）
  - セルフホストループE2E検証

### 🔥 **Milestone 2: 完全なセルフホスティング**（Phase 15.7完了時）
- **目標**: c0（Rustコンパイラ）→c1（Hakoruneコンパイラ）→c1'（自己コンパイル）成功
- **期限**: 2025-10-27（予想）
- **進捗**: 85-90%完了
- **達成基準**:
  1. UsingResolverBox/NamespaceBox動作
  2. Hakorune VM 14命令すべて実装
  3. .hakoソース→MIR JSON→Hakorune VM実行成功
  4. c0→c1動作
  5. c1→c1'成功
  6. Quick smokes 全PASS維持

### 📋 **Milestone 3: 基本Box完全実装**（Phase 16-17）
- **目標**: String/Array/Map/Integer/Bool/File等の基本Boxを Hakorune で実装
- **期限**: 2025-12-31（目標）
- **進捗**: 5-10%完了
- **残タスク**:
  - IntegerBox, BoolBox実装
  - FileBox, PathBox実装
  - TimerBox完全実装

### 🌟 **Milestone 4: フル機能コンパイラ**（Phase 18-19）
- **目標**: match式/property/lambda/async-await等の全構文サポート
- **期限**: 2026-03-31（目標）
- **進捗**: 未着手

### 🚀 **Milestone 5: Rust依存完全排除**（Phase 20+）
- **目標**: ランタイムシステム含めてすべてHakorune実装
- **期限**: 2026-12-31（目標）
- **進捗**: 未着手

---

## 🎓 教訓・ベストプラクティス

### ✅ **Phase 15.7からの学び**

1. **Box-First設計の威力**
   - 新機能追加が予想の**9倍速**で完了
   - 小さな単位（Box）に分割することで、並行開発・テストが容易
   - 例: UsingResolver/Namespace実装は1日ずつで完了（見積もり12日 → 実績2日）

2. **見積もりの精度**
   - 初期見積もりは慎重すぎた
   - 基盤が成熟すると、上位レイヤーの実装は加速する
   - 教訓: 「基盤整備に時間をかけるべき」

3. **順次開発 vs 並行開発**
   - Using解決がモジュールシステムの基盤
   - コンパイラー完成 → Hakorune VM拡張の順が合理的
   - 教訓: 「依存関係を正しく理解し、順序を守る」

4. **品質ファーストの重要性**
   - 計画外の成果（SignatureVerifier/MethodRegistry/JsonCursorBox）が開発を加速
   - Fail-Fast文化の確立が重要
   - 教訓: 「品質に投資すると、結果的に速度が上がる」

### 🚨 **移行時の注意点**

1. **パリティテスト必須**
   - Rust実装とHakorune実装を並行実行し、出力を比較
   - 差分があれば即座に調査・修正

2. **ロールバック経路の確保**
   - 環境変数でRust実装/Hakorune実装を切り替え可能に
   - 問題があれば即座にRust実装へロールバック

3. **ドキュメントの充実**
   - 移行の各ステップを詳細に記録
   - 問題点・失敗・学びを必ず文書化

4. **スモークテストの拡充**
   - 新機能追加時は必ずテスト追加
   - Quick smokes常緑維持

---

## 📚 関連ドキュメント

- [Phase 15.7 README](phases/phase-15.7/README.md) - セルフホスティング実現への道筋
- [00_MASTER_ROADMAP](phases/00_MASTER_ROADMAP.md) - 開発マスタープラン
- [CURRENT_TASK.md](../../../CURRENT_TASK.md) - 現在のタスク

---

**最終更新**: 2025-10-06
**作成者**: Claude（調査・分析・文書化）
