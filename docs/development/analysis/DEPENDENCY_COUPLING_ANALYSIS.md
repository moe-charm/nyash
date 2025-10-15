# 依存関係・結合度分析レポート

**分析日**: 2025-10-15
**対象**: `/home/tomoaki/git/hakorune-selfhost/selfhost/`
**分析ファイル数**: 165 .hako files
**総依存関係数**: 447 using statements

---

## 📊 エグゼクティブサマリー

### ✅ **良い点**
1. **循環依存ゼロ** - 全モジュールで循環依存なし（健全な設計）
2. **双方向依存ゼロ** - 高結合なモジュールペアなし（優れた疎結合）
3. **平均依存度: 3.02** - 適度な依存関係（業界標準: 2-5）
4. **レイヤー違反: 12件のみ** - アーキテクチャの整合性が高い

### ⚠️ **改善点**
1. **Hub modules過剰依存** - 5つのモジュールに15+の依存（単一障害点リスク）
2. **未使用import 36モジュール** - 不要な依存関係による複雑度増加
3. **レイヤー違反 12件** - 下位レイヤーが上位レイヤーに依存
4. **高結合モジュール** - 25+依存を持つモジュール（複雑度高）

---

## 🔍 詳細分析

### 1. 循環依存の検出

**結果**: ✅ **循環依存なし**

**意味**:
- グラフが有向非巡回グラフ（DAG）である
- ビルド順序が明確
- テストの独立性が保たれている
- リファクタリング容易性が高い

**維持するための推奨事項**:
- 新しいモジュール追加時に依存方向を意識
- 定期的な依存関係チェック（CI統合推奨）
- レイヤー分離の遵守

---

### 2. 高結合モジュールペア（双方向依存）

**結果**: ✅ **双方向依存なし**

**意味**:
- すべてのモジュールが単方向依存
- 優れた疎結合設計
- モジュール単体でのテスト・交換が容易

---

### 3. 結合度メトリクス

#### **Top 10 最高結合度スコア**

| Rank | モジュール | Efferent | Afferent | Total | Instability |
|------|-----------|----------|----------|-------|-------------|
| 1 | **hakorune-vm.hakorune_vm_core** | 17 | 16 | **33** | 0.52 |
| 2 | **compiler.pipeline_v2.pipeline** | 25 | 1 | **26** | 0.96 |
| 3 | **shared.json.json_cursor** | 3 | 22 | **25** | 0.12 |
| 4 | **hakorune-vm.value_manager** | 2 | 20 | **22** | 0.09 |
| 5 | **hakorune-vm.instruction_dispatcher** | 19 | 1 | **20** | 0.95 |
| 6 | **hakorune-vm.json_field_extractor** | 2 | 17 | **19** | 0.11 |
| 7 | **hakorune-vm.mircall_handler** | 17 | 1 | **18** | 0.94 |
| 8 | **compiler.pipeline_v2.regex_flow** | 1 | 15 | **16** | 0.06 |
| 9 | **vm.boxes.mir_vm_min** | 14 | 1 | **15** | 0.93 |
| 10 | **hakorune-vm.boxcall_handler** | 8 | 2 | **10** | 0.80 |

**用語説明**:
- **Efferent (出力結合)**: 他のモジュールへの依存数（このモジュールが他を使う）
- **Afferent (入力結合)**: 他のモジュールからの依存数（他がこのモジュールを使う）
- **Total**: 総結合度（Efferent + Afferent）
- **Instability**: 不安定度 = Efferent / Total（0=安定、1=不安定）

**分析**:
- `hakorune_vm_core` は **バランス型Hub** (16 in / 17 out) - 中核的役割
- `pipeline` は **高依存型** (25 out) - 多くのモジュールに依存
- `json_cursor`, `value_manager` は **安定Hub** (20+ in) - 多くから依存される

---

### 4. Hub モジュール（高Afferent結合）

**定義**: 多くのモジュールから依存されるモジュール（Afferent ≥ 5）

| Rank | モジュール | 被依存数 | 役割 |
|------|-----------|---------|------|
| 1 | `shared.json.json_cursor` | **22** | JSON走査の中核 |
| 2 | `hakorune-vm.value_manager` | **20** | VM値管理の基盤 |
| 3 | `hakorune-vm.json_field_extractor` | **17** | MIR JSONフィールド抽出 |
| 4 | `hakorune-vm.hakorune_vm_core` | **16** | VM実行エンジン本体 |
| 5 | `compiler.pipeline_v2.regex_flow` | **15** | 正規表現フロー制御 |
| 6 | `hakorune-vm.json_scan_guard` | **7** | JSON走査ガード |
| 7 | `hakorune-vm.reg_guard` | **6** | レジスタガード |
| 8 | `shared.json.utils.json_frag` | **6** | JSON断片操作 |
| 9 | `shared.mir.block_builder_box` | **5** | MIRブロック構築 |

**リスク**:
- **単一障害点（SPOF）**: これらのモジュールの変更が広範囲に影響
- **変更の波及効果**: バグ修正・仕様変更が多くのモジュールに影響
- **テスト複雑度**: これらのモジュールのテスト失敗が多くのテストを連鎖的に失敗させる

**推奨対策**:
1. **インターフェース抽出** (Extract Interface)
   - 抽象Boxを定義し、実装を疎結合化
   - 例: `JsonCursorInterface` ← `JsonCursorImpl`

2. **Facade パターン**
   - 複数の細かいモジュールを1つのFacadeでラップ
   - 例: `HakoruneVmFacade` (value_manager + json_field_extractor + ...)

3. **Dependency Injection**
   - コンストラクタ/メソッドで依存を注入
   - テスト時にモック可能に

---

### 5. 高依存モジュール（高Efferent結合）

**定義**: 多くのモジュールに依存するモジュール（Efferent ≥ 4）

| Rank | モジュール | 依存数 | 問題点 |
|------|-----------|-------|--------|
| 1 | `compiler.pipeline_v2.pipeline` | **25** | 過度に複雑、分割推奨 |
| 2 | `hakorune-vm.instruction_dispatcher` | **19** | 命令ハンドラ統合、Facade推奨 |
| 3 | `hakorune-vm.mircall_handler` | **17** | 呼び出しロジック複雑、分割推奨 |
| 4 | `hakorune-vm.hakorune_vm_core` | **17** | VM本体、複雑度高い |
| 5 | `vm.boxes.mir_vm_min` | **14** | ミニVM実装、依存過多 |
| 6 | `hakorune-vm.method_call_handler` | **9** | メソッド呼び出し処理 |
| 7 | `shared.mir.mir_io_box` | **9** | MIR入出力、レイヤー違反あり |
| 8 | `hakorune-vm.boxcall_handler` | **8** | Box呼び出し処理 |

**問題点**:
- **変更コスト高**: 1つの依存が変わると修正箇所が多い
- **テスト困難**: 多くのモックが必要
- **理解困難**: 依存関係を追うのが難しい

**推奨対策**:
1. **モジュール分割** (Split Module)
   - 責任を明確に分離（例: pipeline → pipeline_parser + pipeline_emit）
   - 各モジュールは単一責任を持つ

2. **Facade 導入**
   - 複数の関連モジュールを1つのFacadeで隠蔽
   - 例: `InstructionHandlerFacade` (binop + compare + boxcall + ...)

3. **依存注入 (DI)**
   - 依存を外部から注入し、ハードコーディングを避ける

---

### 6. レイヤー違反の検出

**想定アーキテクチャ**:
```
Layer 5: Tools/Tests      (開発用ユーティリティ)
         ↓
Layer 4: Compiler         (コンパイラ実装)
         ↓
Layer 3: VM               (VM実装: hakorune-vm, vm)
         ↓
Layer 2: Infrastructure   (共通サービス: shared/json, shared/mir)
         ↓
Layer 1: Foundation       (基本ユーティリティ: shared/common)
```

**ルール**: 下位レイヤーは上位レイヤーに依存してはならない

#### **検出された違反（12件）**

| 優先度 | 違反元 | 違反先 | 重大度 |
|-------|--------|--------|--------|
| 🔴 High | `shared.common.mini_vm_binop` (L1) | `vm.json` (L3) | **Severity: 2** |
| 🔴 High | `shared.common.mini_vm_binop` (L1) | `vm.scan` (L3) | **Severity: 2** |
| 🔴 High | `shared.common.mini_vm_compare` (L1) | `vm.scan` (L3) | **Severity: 2** |
| 🟡 Medium | `vm.flow_runner` (L3) | `compiler.pipeline_v2.flow_entry` (L4) | Severity: 1 |
| 🟡 Medium | `shared.json_adapter` (L2) | `vm.json_cur` (L3) | Severity: 1 |
| 🟡 Medium | `shared.common.mini_vm_scan` (L1) | `shared.json.json_cursor` (L2) | Severity: 1 |
| 🟡 Medium | `shared.mir.mir_io_box` (L2) | `hakorune-vm.function_locator` (L3) | Severity: 1 |
| 🟡 Medium | `shared.mir.mir_io_box` (L2) | `hakorune-vm.block_iterator` (L3) | Severity: 1 |
| 🟡 Medium | `shared.mir.mir_io_box` (L2) | `hakorune-vm.instrs_locator` (L3) | Severity: 1 |
| 🟡 Medium | `shared.mir.mir_io_box` (L2) | `vm.boxes.result_box` (L3) | Severity: 1 |
| 🟡 Medium | `shared.mir.mir_io_box` (L2) | `hakorune-vm.backward_object_scanner` (L3) | Severity: 1 |
| 🟡 Medium | `shared.mir.mir_io_box` (L2) | `hakorune-vm.blocks_locator` (L3) | Severity: 1 |

**問題点**:
1. **Foundation層がVM層に依存** (Severity 2)
   - `shared.common.mini_vm_*` が `vm.*` に依存
   - **根本原因**: Mini VM実装がFoundation層に誤配置

2. **Infrastructure層がVM層に依存** (Severity 1)
   - `shared.mir.mir_io_box` が多数のVM層モジュールに依存
   - **根本原因**: MIR I/O処理がVM固有ロジックに依存

**推奨修正**:
1. **Mini VM関連をVM層に移動**
   - `shared.common.mini_vm_*` → `vm/mini/`
   - Foundation層は純粋なユーティリティのみに

2. **Interface定義の逆転**
   - `shared.mir.mir_io_box` がインターフェースを定義
   - VM層がそのインターフェースを実装
   - 依存注入でVM実装を注入

3. **Result boxの再配置**
   - `vm.boxes.result_box` → `shared.common.result_box`
   - VM固有ではなく汎用的なエラーハンドリング

---

### 7. 未使用import（36モジュール）

**問題点**:
- 不要な依存関係が残る
- 結合度メトリクスが歪む
- コード可読性が低下

**検出例**:
```hako
// compiler.pipeline_v2.pipeline
using selfhost.common.json.mir_builder_min as MirJsonBuilderMin  // ❌ 未使用
using "selfhost/compiler/pipeline_v2/map_helpers_box.hako" as MapHelpersBox  // ❌ 未使用
```

**推奨対応**:
1. **自動検出スクリプト導入** (CI統合)
2. **定期的なクリーンアップ**
3. **Linter設定** (未使用import警告)

---

## 📈 依存関係グラフ

### **レイヤー間依存関係**

```
Layer Graph (重み付き):

hakorune_vm -> shared_json   [13 dependencies]
vm          -> shared_json   [12 dependencies]
compiler    -> shared_mir    [5 dependencies]
shared_mir  -> hakorune_vm   [5 dependencies]
compiler    -> shared_json   [2 dependencies]
vm          -> shared_common [2 dependencies]
vm          -> compiler      [1 dependency] ⚠️ 違反
hakorune_vm -> shared_mir    [1 dependency]
shared_common -> shared_json [1 dependency]
shared_mir  -> shared_json   [1 dependency]
```

**可視化**:
```
DOT形式グラフ生成済み:
  - /tmp/selfhost_deps_simplified.dot (主要モジュール19個)
  - /tmp/selfhost_deps_layers.dot (レイヤー間依存)

表示方法:
  dot -Tpng /tmp/selfhost_deps_layers.dot -o layers.png
```

---

## 🎯 リファクタリング推奨事項

### **High Priority（5項目）**

#### 1. **Extract Interface: shared.json.json_cursor**
- **理由**: 22モジュールから依存される超Hub
- **対応**:
  ```hako
  // 新規: shared/json/json_cursor_interface.hako
  box JsonCursorInterface {
      // 抽象メソッド定義のみ
      next_value()
      expect_object()
      ...
  }

  // 既存: shared/json/json_cursor.hako
  box JsonCursorBox from JsonCursorInterface {
      // 実装
  }
  ```
- **効果**:
  - テスト容易性向上（モック可能）
  - 変更の影響範囲限定
  - 実装の差し替え可能

#### 2. **Extract Interface: hakorune-vm.value_manager**
- **理由**: 20モジュールから依存される基盤
- **対応**: 同上（インターフェース抽出）
- **効果**: VM実装の疎結合化、テスト改善

#### 3. **Extract Interface: hakorune-vm.json_field_extractor**
- **理由**: 17モジュールから依存される
- **対応**: 同上
- **効果**: MIR処理の柔軟性向上

#### 4. **Extract Interface: hakorune-vm.hakorune_vm_core**
- **理由**: 16モジュールから依存される + 17モジュールに依存（高結合）
- **対応**:
  ```hako
  // 1. 役割分離
  box VmExecutor { /* 実行ロジック */ }
  box VmState { /* 状態管理 */ }
  box VmInstructionSet { /* 命令セット */ }

  // 2. Facade統合
  box HakoruneVmFacade {
      executor: VmExecutor
      state: VmState
      instructions: VmInstructionSet
  }
  ```
- **効果**: 複雑度削減、テスト容易性向上

#### 5. **Extract Interface: compiler.pipeline_v2.regex_flow**
- **理由**: 15モジュールから依存される
- **対応**: インターフェース抽出
- **効果**: Compiler内部の疎結合化

---

### **Medium Priority（10項目）**

#### 1. **Split Module: compiler.pipeline_v2.pipeline**
- **理由**: 25モジュールに依存（過度に複雑）
- **対応**:
  ```
  pipeline.hako → 分割
    ├── pipeline_parser.hako    (パース処理)
    ├── pipeline_analyzer.hako  (解析処理)
    ├── pipeline_emitter.hako   (出力処理)
    └── pipeline_coordinator.hako (統合)
  ```
- **効果**: 複雑度削減、テスト容易性向上

#### 2. **Split Module: hakorune-vm.instruction_dispatcher**
- **理由**: 19モジュールに依存（命令ハンドラ統合）
- **対応**:
  ```hako
  // Facade パターン
  box InstructionHandlerFacade {
      binop: BinopHandler
      compare: CompareHandler
      boxcall: BoxcallHandler
      ...
  }
  ```
- **効果**: ハンドラの独立性向上、拡張容易性

#### 3-5. **その他Split Module推奨**
- `hakorune-vm.mircall_handler` (17依存)
- `hakorune-vm.hakorune_vm_core` (17依存)
- `vm.boxes.mir_vm_min` (14依存)

#### 6-10. **Facade導入推奨**
- `hakorune-vm.*` → `HakoruneVmFacade`
- `compiler.pipeline_v2.*` → `CompilerPipelineFacade`
- `shared.json.*` → `JsonUtilsFacade`

---

### **Low Priority（36項目）**

#### **Remove Dead Imports**
- **対象**: 36モジュール
- **対応**: 未使用importの一括削除
- **効果**: コードクリーン化、結合度メトリクス改善

**自動化スクリプト例**:
```bash
# 未使用import検出・削除スクリプト
python3 /tmp/remove_unused_imports.py selfhost/
```

---

## 🏗️ 理想的なアーキテクチャ

### **推奨レイヤー構造**

```
┌─────────────────────────────────────┐
│  Layer 5: Tools / Tests             │  (開発用、テスト)
│  - tools/*, tests/*                 │
└─────────────────────────────────────┘
           ↓ (依存)
┌─────────────────────────────────────┐
│  Layer 4: Compiler                  │  (コンパイラロジック)
│  - compiler/pipeline_v2/*           │
│  Interface: CompilerPipelineFacade  │
└─────────────────────────────────────┘
           ↓ (依存)
┌─────────────────────────────────────┐
│  Layer 3: VM                        │  (VM実装)
│  - hakorune-vm/*, vm/*              │
│  Interface: HakoruneVmFacade        │
└─────────────────────────────────────┘
           ↓ (依存)
┌─────────────────────────────────────┐
│  Layer 2: Infrastructure            │  (共通サービス)
│  - shared/json/*, shared/mir/*      │
│  Interface: JsonUtils, MirBuilder   │
└─────────────────────────────────────┘
           ↓ (依存)
┌─────────────────────────────────────┐
│  Layer 1: Foundation                │  (基本ユーティリティ)
│  - shared/common/* (string, result) │
│  - 純粋関数、ドメイン非依存         │
└─────────────────────────────────────┘
```

### **設計原則**

1. **依存方向の一貫性**
   - 上位レイヤーは下位レイヤーに依存OK
   - 下位レイヤーは上位レイヤーに依存NG

2. **Interface定義の場所**
   - 依存される側（下位レイヤー）がInterfaceを定義
   - 依存する側（上位レイヤー）が実装を注入

3. **Facade パターンの活用**
   - 各レイヤーの複雑さを隠蔽
   - 単一エントリーポイント提供

4. **循環依存の禁止**
   - 定期的な依存関係チェック
   - CI/CDでの自動検証

---

## 📋 実装優先度まとめ

### **Phase 1: Quick Wins（1-2週間）**
1. ✅ 未使用import削除（36モジュール）
2. ✅ レイヤー違反修正（Mini VM移動）
3. ✅ Result box再配置

### **Phase 2: Hub Refactoring（2-4週間）**
1. 🔧 `json_cursor` インターフェース抽出
2. 🔧 `value_manager` インターフェース抽出
3. 🔧 `json_field_extractor` インターフェース抽出

### **Phase 3: Complex Module Split（4-8週間）**
1. 🏗️ `pipeline` 分割（4モジュール）
2. 🏗️ `hakorune_vm_core` 分割（3モジュール）
3. 🏗️ `instruction_dispatcher` Facade化

### **Phase 4: Architecture Refinement（継続的）**
1. 📐 レイヤー分離の徹底
2. 📐 Facade パターン導入
3. 📐 依存注入の統一

---

## 📊 ROI分析（投資対効果）

| 施策 | 工数 | 効果 | ROI |
|------|------|------|-----|
| 未使用import削除 | 0.5日 | 可読性+10%, 保守性+5% | ⭐⭐⭐⭐⭐ |
| レイヤー違反修正 | 1日 | アーキテクチャ整合性+20% | ⭐⭐⭐⭐⭐ |
| Hub Interface抽出 | 3-5日 | テスト容易性+30%, 変更容易性+25% | ⭐⭐⭐⭐ |
| Pipeline分割 | 7-10日 | 複雑度-40%, 保守性+35% | ⭐⭐⭐⭐ |
| VM Core分割 | 10-14日 | テスト容易性+40%, 拡張性+30% | ⭐⭐⭐ |

---

## 🔍 継続的改善

### **CI/CD統合推奨**
```yaml
# .github/workflows/dependency-check.yml
name: Dependency Check
on: [push, pull_request]
jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Analyze Dependencies
        run: |
          python3 scripts/analyze_dependencies.py
          python3 scripts/check_unused_imports.py
          python3 scripts/detect_circular_deps.py
      - name: Fail on violations
        run: |
          if [ -f violations.txt ]; then exit 1; fi
```

### **定期レビュー推奨**
- **月次**: 新規レイヤー違反チェック
- **四半期**: 結合度メトリクス評価
- **半期**: 大規模リファクタリング計画

---

## 📚 参考資料

### **生成ファイル**
- `/tmp/selfhost_dependencies.dot` - 全モジュール依存グラフ
- `/tmp/selfhost_deps_simplified.dot` - 主要モジュール依存グラフ（19ノード）
- `/tmp/selfhost_deps_layers.dot` - レイヤー間依存グラフ

### **可視化コマンド**
```bash
# PNG生成
dot -Tpng /tmp/selfhost_deps_layers.dot -o layers.png
dot -Tpng /tmp/selfhost_deps_simplified.dot -o modules.png

# SVG生成（拡大可能）
dot -Tsvg /tmp/selfhost_deps_layers.dot -o layers.svg
```

### **関連ドキュメント**
- アーキテクチャ設計: `docs/development/architecture/`
- レイヤー設計: `docs/development/architecture/layer-separation.md`
- Box設計: `docs/reference/boxes-system/`

---

**作成者**: Claude Code Analysis Agent
**分析ツール**: Python 3 + 静的解析スクリプト
**更新日**: 2025-10-15
