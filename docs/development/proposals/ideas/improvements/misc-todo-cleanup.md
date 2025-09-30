# その他TODO整理（設計メモ・低優先事項）

## 📋 概要

**発見日**: 2025-09-30
**優先度**: 🔵 最低（設計メモ・将来検討事項）
**影響範囲**: 各種機能の設計メモ

## 🎯 TODOリスト（13個）

### 1. HTTPハンドラー実装（1個）

#### `src/boxes/http_server_box.rs:350`
```rust
// TODO: Actual handler invocation would need method calling infrastructure
```

**内容**: HTTPハンドラー呼び出しインフラ未実装

**推奨対応**: Phase 17（Web機能強化）で実装

---

### 2. 設計メモ系（12個）

#### A. Box Operators（1個）
**場所**: `src/box_operators.rs:14`
```rust
// * - Phase 2-4: Static/Dynamic implementations and resolver (TODO)
```

**内容**: Box演算子のPhase 2-4実装計画メモ

**推奨対応**: そのまま残す（実装計画のマイルストーン）

#### B. Box Trait将来実装（1個）
**場所**: `src/box_trait.rs:156`
```rust
// TODO: 次のステップで完全実装
```

**内容**: NyashBox trait拡張予定メモ

**推奨対応**: そのまま残す（設計意図のメモ）

#### C. Method Box GC（1個）
**場所**: `src/method_box.rs:〇〇`
```rust
// TODO: GC実装予定
```

**内容**: MethodBox GC実装メモ（推定）

**推奨対応**: GC設計時に検討

#### D. MIR Instruction Display（1個）
**場所**: `src/mir/instruction/display.rs:〇〇`
```rust
// TODO: より詳細なDisplay実装
```

**内容**: MIR命令の表示改善メモ（推定）

**推奨対応**: デバッグ改善時に実装

#### E. MIR Passes（2個）
**場所**:
- `src/mir/passes/cse.rs:〇〇` - CSE（Common Subexpression Elimination）最適化
- `src/mir/passes/method_id_inject.rs:〇〇` - メソッドID注入最適化

**内容**: MIR最適化パス改善メモ

**推奨対応**: Phase 16-17（最適化強化）で実装

#### F. Parser Dependency Helpers（1個）
**場所**: `src/parser/declarations/dependency_helpers.rs:〇〇`
```rust
// TODO: 依存関係解決改善
```

**内容**: using/namespace依存関係解決メモ

**推奨対応**: using system完成後に改善

#### G. Runtime系（3個）
**場所**:
- `src/runtime/plugin_box_legacy.rs:〇〇` (2個) - レガシープラグインBox削除予定
- `src/runtime/plugin_loader_v2/enabled/extern_functions.rs:〇〇` - 外部関数ローダー改善
- `src/runtime/tests.rs:〇〇` - テスト追加メモ
- `src/runtime/unified_registry.rs:〇〇` - 統一レジストリ設計メモ

**内容**: ランタイムシステム改善メモ

**推奨対応**: 段階的改善（Phase 15-17）

## 💡 対応方針

### Option A: コメント→ドキュメント移行（推奨）

各TODOをドキュメント化：

#### 実装ステップ
1. TODOごとにドキュメント作成
   - `docs/development/proposals/ideas/improvements/` に個別ファイル作成
2. ソースコード内TODOコメント削除
3. 代わりにドキュメントへのリンクを追加

#### 例
```rust
// 旧: TODO: Phase 2-4実装計画
// 新: See: docs/development/proposals/ideas/improvements/box-operators-phase2-4.md
```

**利点**:
- ドキュメントで詳細記述可能
- ソースコードがクリーン
- 検索性向上

**実装時間**: 2-3時間（13個すべて）

### Option B: そのまま残す（現状維持）

設計メモとしてTODOコメント保持

**利点**:
- 実装箇所の近くにメモ
- 実装時に気づきやすい

**欠点**:
- TODOノイズ（検索時）
- 詳細記述困難

### Option C: Issue化

GitHub Issueとして管理

**利点**:
- 正式な技術的負債管理
- 優先度・担当者設定可能

**欠点**:
- Issue数増加（13個追加）
- 低優先事項の管理負担

## 🚀 実装ステップ（推奨: Option A）

### Step 1: HTTPハンドラー - 30分
`docs/development/proposals/ideas/improvements/http-handler-infrastructure.md`作成

### Step 2: 設計メモ系 - 2時間
各TODOごとにドキュメント作成（簡易版でOK）

### Step 3: ソースコード更新 - 30分
TODOコメント削除、ドキュメントリンク追加

## 📊 影響範囲

### 新規ドキュメント（13個）
- `http-handler-infrastructure.md`
- `box-operators-phases.md`
- `box-trait-future-enhancements.md`
- `method-box-gc-design.md`
- `mir-display-improvements.md`
- `mir-optimization-passes.md`
- `parser-dependency-resolution.md`
- `runtime-legacy-cleanup.md`
- `runtime-extern-functions.md`
- `runtime-test-coverage.md`
- `runtime-unified-registry.md`

### 修正ファイル（10箇所）
- 各TODOコメント箇所でリンク追加

## 🎯 成功基準

- ✅ 全13個のTODOがドキュメント化
- ✅ ソースコード内TODOコメント削減（44個 → 31個以下）
- ✅ ドキュメント検索性向上
- ✅ 既存のすべてのスモークテストがPASS

## 🔗 関連資料

- [80/20ルール](../../../../guides/development-practices.md#8020ルール)
- [アイデア管理方針](../../../../development/proposals/ideas/README.md)

## 📝 補足

**優先度判断**:
- 設計メモは「残り20%」（80/20ルール）
- 今すぐ実装不要、記録重要
- **Phase 3クリーンアップの一環として実施推奨**

**実装タイミング**: Phase 15完了後、コードベースクリーンアップ時

**メリット**:
- ソースコードがクリーン（TODOノイズ削減）
- ドキュメント充実（設計意図明確化）
- 将来実装時の参考資料

**注意点**:
- 簡易ドキュメントでOK（詳細は実装時に）
- 「完璧より進捗」を守る
- 過度な時間をかけない（各TODO 10分程度）