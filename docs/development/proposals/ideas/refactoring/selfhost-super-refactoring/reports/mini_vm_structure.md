# Mini-VM 構造調査レポート

**調査日時**: 2025-10-04
**対象**: `/home/tomoaki/git/hakorune-selfhost/apps/selfhost/vm/` および関連モジュール

---

## 📊 エグゼクティブサマリー

Mini-VMは、Hakoruneのセルフホスティング実装における**JSON v0形式のMIR実行器**です。
- **総ファイル数**: 32ファイル (.hako + .nyash)
- **総行数**: 約3,400行 (.hako: 2,591行 + .nyash: 795行)
- **主要実行器**: `mir_vm_min.hako` (191行) - 最小MIR実行器
- **アーキテクチャ**: 箱化（Box-First）設計による疎結合モジュール構造

---

## 📁 ディレクトリ構造

```
apps/selfhost/
├── vm/                          # Mini-VM本体
│   ├── boxes/                   # 箱化版モジュール（.hako）
│   │   ├── mir_vm_min.hako      # 最小MIR実行器（191行）
│   │   ├── mini_vm_core.hako    # コア機能（28行）
│   │   ├── mini_vm_prints.hako  # Print命令処理（115行）
│   │   ├── step_runner.hako     # ステップ観測器（74行）
│   │   ├── op_handlers.hako     # 命令ハンドラ（148行）
│   │   ├── arithmetic.hako      # 安全算術演算（136行）
│   │   ├── compare_ops.hako     # 比較演算（24行）
│   │   ├── operator_box.hako    # 演算子統一API（36行）
│   │   ├── json_*.hako          # JSON解析ユーティリティ（計183行）
│   │   └── その他ユーティリティ
│   ├── flow_runner.hako         # フロー実行器（39行）
│   └── *.nyash                  # 旧版実装（600行）
└── common/                      # 共通モジュール
    ├── mini_vm_scan.hako        # スキャンヘルパー（206行）
    ├── mini_vm_binop.hako       # 二項演算処理（277行）
    ├── mini_vm_compare.hako     # 比較演算処理（48行）
    └── json_adapter.hako        # JSONアダプター（19行）
```

---

## 🎯 主要コンポーネント

### 1. **実行器コア** (Executor Core)

#### `mir_vm_min.hako` (191行) - 最小MIR実行器
**責務**: MIR JSON v0形式の実行
- **対応命令**: const, copy, binop, compare, branch, jump, ret
- **実行モデル**: レジスタマシン（MapBox使用）
- **最大ステップ数**: 200,000（無限ループ防止）
- **Fail-Fast原則**: エラー時は明示的に-1を返す
- **thin_mode**: 簡略化されたret解決モード

**依存関係**:
```
mir_vm_min.hako
├── op_handlers.hako      # 命令ハンドラ
├── json_frag.hako        # JSON断片抽出
├── operator_box.hako     # 演算子API
└── compare_ops.hako      # 比較演算
```

#### `flow_runner.hako` (39行) - フロー実行器
**責務**: AST JSON → MIR → 実行のパイプライン
- **Fast-path最適化**: `Return(Int v)` 直接処理
- **互換モード**: v1→v0変換経路対応
- **依存**: FlowEntryBox（pipeline_v2）+ MirVmMin

---

### 2. **命令処理** (Instruction Handlers)

#### `op_handlers.hako` (148行)
**責務**: const/binop/compare命令の実装
- **const**: 定数ロード（i64対応）
- **binop**: Add/Sub/Mul/Div/Mod（ArithmeticBox使用）
- **compare**: 比較演算（CompareOpsBox使用）
- **安全性**: _is_numeric_str()による型検証

#### `compare_ops.hako` (24行)
**責務**: 比較演算のマッピングと評価
- **シンボル→Kind変換**: `==` → `Eq`, `<` → `Lt` など
- **評価関数**: `eval(kind, a, b) → 0/1`
- **対応演算**: Eq, Ne, Lt, Le, Gt, Ge

#### `arithmetic.hako` (136行)
**責務**: 安全な10進数演算
- **オーバーフロー回避**: 文字列ベース演算
- **対応演算**: add_i64, sub_i64, mul_i64
- **負数対応**: sub時に符号処理

#### `operator_box.hako` (36行)
**責務**: 演算子の統一API
- **apply2**: 二項演算（Add/Sub/Mul/Div/Mod/Bit系）
- **unary**: 単項演算（Neg/Not/BitNot）
- **compare**: 比較演算（CompareOpsBoxに委譲）

---

### 3. **JSON解析** (JSON Parsing)

#### `json_frag.hako` (51行) - JSON断片抽出
**責務**: JSON文字列からkey:intやkey:strを抽出
- **get_int(seg, key)**: 数値フィールド取得
- **get_str(seg, key)**: 文字列フィールド取得
- **block0_segment**: instructions配列抽出

#### `json_scan.hako` (71行) - 構造スキャン
**責務**: エスケープ対応のJSON構造解析
- **seek_obj_end**: オブジェクト終端検出
- **seek_array_end**: 配列終端検出
- **find_key_dual**: プレーン/エスケープ両対応

#### `json_cur.hako` (61行) - カーソルヘルパー
**責務**: 低レベルJSON走査
- **next_non_ws**: 空白スキップ
- **read_quoted_from**: 引用符付き文字列読み込み
- **read_digits_from**: 数字列読み込み

#### `string_scan.hako` (42行)
**責務**: 文字列スキャン基本機能
- **scan_string_end**: 文字列終端検出（エスケープ対応）

---

### 4. **Print処理** (Print Handler)

#### `mini_vm_prints.hako` (115行)
**責務**: Print命令の実装（AST JSONからの出力）
- **try_print_string_value_at**: 文字列リテラル出力
- **try_print_int_value_at**: 整数リテラル出力（型付き）
- **try_print_functioncall_at**: echo/itoa関数呼び出し
- **依存**: MiniVmScan, MiniVmBinOp, MiniVmCompare, MiniJsonLoader

---

### 5. **共通モジュール** (Common Modules)

#### `mini_vm_scan.hako` (206行) - apps/selfhost/common/
**責務**: スキャンと数値ヘルパー
- **index_of_from**: 部分文字列検索
- **find_balanced_array_end/object_end**: 括弧対応
- **_str_to_int / _int_to_str**: 数値変換
- **read_digits**: 数字列読み込み
- **sum_***: 数値集計関数（複数バリエーション）

#### `mini_vm_binop.hako` (277行) - apps/selfhost/common/
**責務**: BinaryOp処理（Print内の二項演算）
- **try_print_binop_at**: BinaryOp(+)の出力
- **string+string**: 文字列連結対応
- **int+int**: 整数加算対応
- **sum_***: 複数のフォールバック戦略

#### `mini_vm_compare.hako` (48行) - apps/selfhost/common/
**責務**: Compare処理（Print内の比較演算）
- **try_print_compare_at**: Compare命令の出力
- **対応演算**: <, ==, <=, >, >=, !=

---

## 🔗 依存関係マップ

```
【実行器層】
mir_vm_min.hako (191行)
  ├─→ op_handlers.hako (148行)
  │     ├─→ arithmetic.hako (136行)
  │     └─→ compare_ops.hako (24行)
  ├─→ json_frag.hako (51行)
  │     ├─→ string_scan.hako (42行)
  │     └─→ json_scan.hako (71行)
  ├─→ operator_box.hako (36行)
  │     ├─→ arithmetic.hako
  │     └─→ compare_ops.hako
  └─→ compare_ops.hako

flow_runner.hako (39行)
  ├─→ FlowEntryBox (pipeline_v2)
  └─→ mir_vm_min.hako

【ヘルパー層】
mini_vm_core.hako (28行)
  ├─→ json_cur.hako (61行) as MiniJson
  ├─→ mini_vm_scan.hako (206行) [common]
  ├─→ mini_vm_binop.hako (277行) [common]
  ├─→ mini_vm_compare.hako (48行) [common]
  └─→ mini_vm_prints.hako (115行)

mini_vm_prints.hako (115行)
  ├─→ mini_vm_scan.hako [common]
  ├─→ mini_vm_binop.hako [common]
  ├─→ mini_vm_compare.hako [common]
  └─→ json_cur.hako as MiniJsonLoader

【JSON層】
json_frag.hako → string_scan.hako + json_scan.hako
json_scan.hako → string_scan.hako

【演算層】
op_handlers.hako → arithmetic.hako + compare_ops.hako
operator_box.hako → arithmetic.hako + compare_ops.hako
```

---

## 📈 コード規模

### .hako ファイル（箱化版・現行）

| ファイル | 行数 | 責務 |
|---------|------|------|
| `mini_vm_binop.hako` | 277 | BinaryOp処理（common） |
| `mini_vm_scan.hako` | 206 | スキャンヘルパー（common） |
| `seam_inspector.hako` | 202 | 継ぎ目検査器 |
| `mir_vm_min.hako` | 191 | **最小MIR実行器** |
| `op_handlers.hako` | 148 | 命令ハンドラ |
| `arithmetic.hako` | 136 | 安全算術演算 |
| `mini_vm_prints.hako` | 115 | Print命令処理 |
| `instruction_scanner.hako` | 112 | 命令スキャナ |
| `flow_debugger.hako` | 95 | フロー診断器 |
| `step_runner.hako` | 74 | ステップ観測器 |
| `json_scan.hako` | 71 | JSON構造スキャン |
| `json_cur.hako` | 61 | JSONカーソル |
| `minivm_probe.hako` | 57 | Mini-VMプローブ |
| `json_frag.hako` | 51 | JSON断片抽出 |
| `mini_vm_compare.hako` | 48 | Compare処理（common） |
| `string_scan.hako` | 42 | 文字列スキャン |
| `flow_runner.hako` | 39 | フロー実行器 |
| `operator_box.hako` | 36 | 演算子API |
| `mini_vm_core.hako` | 28 | コア機能 |
| `compare_ops.hako` | 24 | 比較演算 |
| `json_adapter.hako` | 19 | JSONアダプター（common） |
| **合計** | **2,591** | |

### .nyash ファイル（旧版）

| ファイル | 行数 | 状態 |
|---------|------|------|
| `vm/boxes/mir_vm_m2.nyash` | 不明 | 旧実装 |
| `vm/boxes/vm_kernel_box.nyash` | 不明 | 旧カーネル |
| `vm/boxes/step_runner.nyash` | 不明 | 旧ステップ実行器 |
| `vm/mini_vm_lib.nyash` | 34 | 初期実装 |
| その他 | 約600 | テスト・ローダー等 |
| **合計** | **約795** | |

---

## 🆚 新旧バージョン比較

### .nyash版（旧）vs .hako版（新）

| 側面 | .nyash版 | .hako版 |
|------|---------|---------|
| **ファイル数** | 約11ファイル | 23ファイル |
| **総行数** | 約795行 | 2,591行 |
| **モジュール化** | 粗い（機能混在） | 細かい（箱化設計） |
| **using構文** | 限定的 | 完全活用 |
| **責務分離** | 不明瞭 | 明確（1箱1責務） |
| **テスト容易性** | 低い | 高い |
| **再利用性** | 低い | 高い |
| **保守性** | 低い | 高い |

**移行状況**:
- ✅ `mini_vm_lib.nyash` (34行) → `mini_vm_core.hako` (28行) + 各種箱
- ✅ `step_runner.nyash` → `step_runner.hako` (74行)
- ✅ `mir_vm_m2.nyash` → `mir_vm_min.hako` (191行)
- 🔄 `.nyash`ファイルは主にテストとローダーとして残存

---

## 🏗️ アーキテクチャ設計原則

### 1. **箱理論（Box-First）実践**

各モジュールが独立した箱として設計されています:

```hakorune
// ✅ 明確な責務分離
static box CompareOpsBox {     // 比較演算のみ
  map_symbol(sym) { ... }
  eval(kind, a, b) { ... }
}

static box ArithmeticBox {     // 算術演算のみ
  add_i64(a, b) { ... }
  sub_i64(a, b) { ... }
  mul_i64(a, b) { ... }
}

static box OperatorBox {       // 統一API
  apply2(kind, a, b) { ... }
  unary(kind, a) { ... }
  compare(kind, a, b) { ... }  // 委譲
}
```

### 2. **Fail-Fast原則**

エラーは即座に明示的に失敗:

```hakorune
// mir_vm_min.hakoより
if op == "ret" {
  local v = JsonFragBox.get_int(seg, "value")
  if v == null {
    me._tprint("[ERROR] Undefined ret value field")
    return -1  // ❌ Fail-Fast: フォールバックしない
  }
  // ...
  if me._is_numeric_str(sval) == 1 {
    return me._load_reg(regs, v)
  }
  me._tprint("[ERROR] Undefined register ret: r"+me._int_to_str(v))
  return -1  // ❌ Fail-Fast
}
```

### 3. **不変条件の保証**

各箱が明確な入出力契約を持つ:

```hakorune
// compare_ops.hako
eval(kind, a, b) {
  // 入力: kind (Eq/Ne/Lt/Le/Gt/Ge), a, b (integers)
  // 出力: 0 または 1（必ず真偽値）
  if kind == "Eq" { if a == b { return 1 } else { return 0 } }
  // ...
  return 0  // デフォルトは0（false）
}
```

### 4. **学習効果（Self-Documentation）**

コメントで責務と非責務を明示:

```hakorune
// json_frag.hako
// 責務: 文字列JSONから key:int / key:str を簡便に取り出す。
// 非責務: 実行・評価（構造検査やVM実行は他箱に委譲）。
```

---

## 🔍 問題点と改善提案

### 1. **重複コード**

#### 問題: _str_to_int() の重複実装
**場所**:
- `arithmetic.hako` (115-130行)
- `op_handlers.hako` (18-33行)
- `json_frag.hako` (12行)
- `json_scan.hako` (8行)
- `step_runner.hako` (8行)

**提案**: 共通ユーティリティ箱に統一
```hakorune
// 新規: apps/selfhost/common/string_utils.hako
static box StringUtilsBox {
  str_to_int(s) { ... }
  int_to_str(n) { ... }
  is_numeric_str(s) { ... }
}
```

#### 問題: index_of_from() の重複実装
**場所**:
- `mini_vm_scan.hako` (4-19行)
- `json_frag.hako` (10行)
- `flow_runner.hako` (7行)
- `mir_vm_min.hako` (41行)
- `step_runner.hako` (6行)

**提案**: StringUtilsBoxに統合

### 2. **命名の不一致**

#### 問題: 同じ機能に異なる名前
- `json_cur.hako` → `MiniJsonCur` / `MiniJson` / `MiniJsonLoader`
- `vm/scan.hako` → `MiniVmScan` (using先が不明確)

**提案**: 統一された命名規則
```
mini_vm_scan.hako → MiniVmScanBox
json_cur.hako     → JsonCursorBox
json_frag.hako    → JsonFragBox (現状維持)
```

### 3. **依存関係の複雑化**

#### 問題: common/ ↔ vm/boxes/ の双方向依存
- `mini_vm_binop.hako` (common) が `selfhost.vm.scan` を使用
- `mini_vm_core.hako` (vm/boxes) が `selfhost.common.*` を使用

**提案**: レイヤー分離
```
Layer 1: common/utils/     # 基本ユーティリティ
Layer 2: common/json/      # JSON処理
Layer 3: common/ops/       # 演算処理
Layer 4: vm/boxes/         # VM実装
```

### 4. **テストコードの欠如**

#### 問題: 単体テストファイルが見当たらない
各箱に対応するテストが存在しない。

**提案**: テスト箱の追加
```
apps/selfhost/vm/tests/
├── test_compare_ops.hako
├── test_arithmetic.hako
├── test_json_frag.hako
└── test_mir_vm_min.hako
```

### 5. **ドキュメント不足**

#### 問題: README.mdなどの説明文書が存在しない

**提案**: 以下のドキュメント追加
```
apps/selfhost/vm/README.md              # Mini-VM概要
apps/selfhost/vm/boxes/README.md        # 箱の説明
apps/selfhost/common/README.md          # 共通モジュール説明
docs/development/mini-vm-architecture.md  # アーキテクチャ設計書
```

---

## 🎯 推奨される改善ロードマップ

### Phase 1: コード重複削除（優先度: 高）
1. **StringUtilsBox新設** (apps/selfhost/common/string_utils.hako)
   - str_to_int(), int_to_str(), is_numeric_str()を統合
   - 全箱をStringUtilsBox使用に移行
   - **効果**: 約150行削減

2. **ScanUtilsBox新設** (apps/selfhost/common/scan_utils.hako)
   - index_of_from(), read_digits()を統合
   - **効果**: 約100行削減

### Phase 2: 命名統一化（優先度: 中）
1. **命名規則確定**
   - すべての箱名を `*Box` 形式に統一
   - using aliasを明示的に

2. **ファイル名とBox名の一致**
   - `json_cur.hako` → `json_cursor.hako` (JsonCursorBox)

### Phase 3: 依存関係整理（優先度: 中）
1. **レイヤー分離**
   - common/utils/, common/json/, common/ops/, vm/boxes/ に再編成
   - 上位層から下位層への一方向依存のみ許可

2. **循環依存の解消**
   - common → vm/boxes の依存を削除
   - 共通機能はすべてcommonに移動

### Phase 4: テスト整備（優先度: 中）
1. **単体テスト追加**
   - 各箱に対応するtest_*.hakoを作成
   - smoke testの体系化

2. **統合テスト追加**
   - MIR JSON v0形式の実行テスト
   - エッジケースのテスト

### Phase 5: ドキュメント整備（優先度: 低）
1. **README追加**
   - 各ディレクトリにREADME.md追加
   - 使用例とAPI仕様を記載

2. **アーキテクチャ図作成**
   - 依存関係の可視化
   - データフロー図の作成

---

## 📝 結論

Mini-VMは**箱理論（Box-First）**に基づいた優れた設計を持つMIR実行器です。

**強み**:
- ✅ 明確な責務分離（1箱1責務）
- ✅ Fail-Fast原則の徹底
- ✅ 不変条件の保証
- ✅ 疎結合なモジュール構造

**改善余地**:
- 🔄 コード重複の削減（約250行削減可能）
- 🔄 命名規則の統一
- 🔄 依存関係の整理
- 🔄 テストとドキュメントの追加

**推奨アクション**:
1. **即時**: StringUtilsBox/ScanUtilsBox新設で重複削減
2. **短期**: 命名統一とレイヤー分離
3. **中期**: テスト整備
4. **長期**: ドキュメント整備

現状でも**十分に動作する品質**ですが、上記改善により**保守性と拡張性が大幅に向上**します。

---

**レポート作成者**: Claude (Sonnet 4.5)
**調査対象コミット**: 51f7e9f1 (feat(mir): PhiMergeHelper箱化)
**ファイル総数**: 52 (.hako: 23, .nyash: 29)
**総行数**: 約3,400行
