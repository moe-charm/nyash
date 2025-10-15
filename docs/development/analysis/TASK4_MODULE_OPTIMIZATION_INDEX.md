# Task 4: モジュール化・ディレクトリ構造最適化 - 完全レポート集

**作成日**: 2025-10-15
**タスク**: selfhost/ のモジュール構造を最適化する計画を立案

---

## 📋 レポート一覧

| ドキュメント | サイズ | 内容 | 読者 |
|-------------|--------|------|------|
| **[module_optimization_summary.md](./module_optimization_summary.md)** | 7.6KB | ⭐エグゼクティブサマリー | マネージャー、意思決定者 |
| **[module_structure_optimization_plan.md](./module_structure_optimization_plan.md)** | 31KB | 完全計画書 | 実装者、レビュワー |
| **[module_dependency_diagram.md](./module_dependency_diagram.md)** | 14KB | 依存関係図 | アーキテクト、開発者 |
| **[migration_quick_reference.md](./migration_quick_reference.md)** | 12KB | 実装チェックリスト | 実装者 |

**合計**: 64.6KB (約15,000語)

---

## 🎯 5秒でわかる結論

selfhost/ には **2つのVM実装が並存** (vm/ と hakorune-vm/) しており、統一が必要。

**推奨**: 5週間の段階的移行計画 (Phase 1-3) で、以下を実現：
1. **基盤統合** (core/ モジュール確立)
2. **VM統一** (runtime/ に一本化、旧VM削除)
3. **MIR集約** (mir/ モジュール確立)

---

## 📖 読み方ガイド

### まずこれを読む (5分)
👉 **[module_optimization_summary.md](./module_optimization_summary.md)** (エグゼクティブサマリー)
- 現状の問題点 (3つ)
- 提案構造 (ビフォー/アフター)
- 5段階マイグレーション計画
- リスク評価

### 詳細を知りたい (30分)
👉 **[module_structure_optimization_plan.md](./module_structure_optimization_plan.md)** (完全計画書)
- 現状分析 (ファイル配置、命名規則、依存関係)
- 推奨ディレクトリ構造 (詳細)
- モジュール分割提案 (5モジュール)
- 移動・リネーム候補 (全リスト)
- hako_module.toml 最適化
- マイグレーション計画 (Phase 1-5)
- リスク分析と緩和策
- 成功基準

### 依存関係を理解したい (15分)
👉 **[module_dependency_diagram.md](./module_dependency_diagram.md)** (依存関係図)
- 現状の依存関係 (混乱状態)
- 提案後の依存関係 (3層アーキテクチャ)
- レイヤー別詳細
- 依存関係マトリックス
- ファイル移動マップ
- 循環依存の検証方法

### 実装を開始したい (実装中に常に参照)
👉 **[migration_quick_reference.md](./migration_quick_reference.md)** (実装チェックリスト)
- Phase 1-5 のチェックリスト
- using 文の変換ルール
- 便利なスクリプト
- トラブルシューティング
- 完了確認チェックリスト
- ロールバック手順

---

## 🔍 重要な発見 (Key Findings)

### 発見1: 2つのVM実装が並存 (🔴 深刻度: 高)

**現状**:
```
selfhost/
├── vm/              # 旧Mini-VM (30ファイル、1,600行)
│   ├── boxes/mini_vm_core.hako
│   └── boxes/mir_vm_min.hako
└── hakorune-vm/    # 新Hakorune VM (67ファイル、3,446行、22命令ハンドラー)
    ├── hakorune_vm_core.hako
    ├── instruction_dispatcher.hako
    └── tests/ (22テストファイル)
```

**問題**:
- 開発者が混乱: どちらを使うべきか不明確
- テストが分散: hakorune-vm/tests/ のみに22テスト
- using 構文が混在: `selfhost.vm.*` vs `"selfhost/hakorune-vm/*"`

**推奨**: 新Hakorune VMを正式VMとし、旧VM (vm/) を削除。

---

### 発見2: 命名の不統一 (🔴 深刻度: 高)

| パターン | 例 | 出現数 | 場所 |
|---------|-----|--------|------|
| `*_box.hako` | `mir_builder_box.hako` | 38個 | compiler, shared |
| `*_handler.hako` | `binop_handler.hako` | 17個 | hakorune-vm |
| `*_guard.hako` | `args_guard.hako` | 3個 | hakorune-vm |
| `*_locator.hako` | `function_locator.hako` | 3個 | hakorune-vm |

**問題**: compiler と hakorune-vm で異なる命名規則 → 統一性なし

**推奨**: `*_box` サフィックスを削減、役割別サフィックス (`*_handler`, `*_guard`) を維持。

---

### 発見3: モジュール境界の不明確 (🟡 深刻度: 中)

**shared/ の役割混乱**:
```
shared/ (23ファイル、8サブディレクトリ)
├── common/
│   ├── mini_vm_scan.hako        # 旧VM依存
│   ├── mini_vm_binop.hako       # 旧VM依存
│   ├── mini_vm_compare.hako     # 旧VM依存
│   ├── string_helpers.hako      # 汎用 (26箇所で使用)
│   └── string_ops.hako          # 汎用 (15箇所で使用)
├── json/ (7ファイル)
├── mir/ (3ファイル)
└── [他5サブディレクトリ]
```

**問題**:
- `mini_vm_*` は旧VMに依存しているが `shared/` にある
- `string_helpers` は汎用だが `common/` に埋もれている
- JSON処理が3箇所に分散

**推奨**: 役割別に再編成 (core, runtime, mir, compiler, backend)。

---

## 📊 統計データ

### ファイル配置 (現状)

| ディレクトリ | .hako数 | 主な内容 |
|-------------|---------|---------|
| `compiler/pipeline_v2/` | 38 | コンパイラパイプライン |
| `hakorune-vm/` | 44 | 新VM本体 |
| `hakorune-vm/tests/` | 22 | VMテスト |
| `shared/` | 23 | 共通ユーティリティ (混在) |
| `vm/boxes/` | 25 | 旧VM |
| `vm/` | 5 | 旧VMエントリーポイント |
| `tools/` | 7 | 開発ツール |
| **合計** | **165** | **26,834行** |

### 最も使用されているBox (Top 5)

| Box | 使用回数 | 場所 | 提案後の場所 |
|-----|---------|------|-------------|
| `result_box.hako` | 36回 | `vm/boxes/` | `core/result.hako` ⭐ |
| `string_helpers.hako` | 26回 | `shared/common/` | `core/string_helpers.hako` ⭐ |
| `value_manager.hako` | 20回 | `hakorune-vm/` | `runtime/value_manager.hako` |
| `json_field_extractor.hako` | 17回 | `hakorune-vm/` | `runtime/json_field_extractor.hako` |
| `string_ops.hako` | 15回 | `shared/common/` | `core/string_ops.hako` ⭐ |

**⭐**: Phase 1で `core/` に移動すべきファイル (最優先)

---

## 🏗️ 提案: 3層アーキテクチャ

### ビフォー (現状)

```
❌ 複雑な依存関係 (shared ←→ hakorune-vm, vm)
❌ VM実装2つ (vm/ と hakorune-vm/)
❌ 循環依存のリスク
❌ モジュール境界不明確
```

### アフター (Phase 3完了後)

```
✅ 明確な3層アーキテクチャ:
   Layer 2 (上位): compiler, backend
   Layer 1 (中位): runtime, mir
   Layer 0 (基盤): core

✅ 循環依存なし (すべてが core を基盤に)
✅ VM統一 (runtime/ のみ)
✅ 命名規則統一 (*_box 削減、役割別サフィックス維持)
✅ テスト集約 (runtime/tests/ に22テスト)
```

### ディレクトリ構造 (提案)

```
selfhost/
├── core/            # 🆕 基盤レイヤー (15-20ファイル)
│   ├── result.hako              # ← vm/boxes/result_box.hako
│   ├── string_helpers.hako      # ← shared/common/string_helpers.hako
│   ├── json_cursor.hako         # ← shared/json/json_cursor.hako
│   └── [他の共通基盤]
│
├── runtime/         # 🆕 VM実行時 (70-80ファイル)
│   ├── vm_core.hako             # ← hakorune-vm/hakorune_vm_core.hako
│   ├── instruction_dispatcher.hako
│   ├── handlers/                # 22命令ハンドラー
│   ├── guards/                  # 検証・保護
│   ├── locators/                # 検索・解決
│   └── tests/                   # ← hakorune-vm/tests/ (22テスト)
│
├── mir/             # 🆕 MIR構築 (10ファイル)
│   ├── schema.hako              # ← shared/mir/mir_schema_box.hako
│   ├── block_builder.hako       # ← shared/mir/block_builder_box.hako
│   └── [他のMIRツール]
│
├── compiler/        # 既存 (38ファイル)
│   └── pipeline_v2/
│       ├── emitters/            # 🆕 emit_*.hako
│       ├── extractors/          # 🆕 *_extract_*.hako
│       └── [他パイプライン]
│
├── backend/         # 🆕 バックエンド (2-3ファイル)
├── tools/           # 既存 (7ファイル)
└── tests/           # 既存 (4ファイル)
```

---

## 📅 マイグレーション計画サマリー

| Phase | 期間 | 目標 | 影響 | 優先度 |
|-------|------|------|------|--------|
| **Phase 1** | Week 1-2 | 基盤統合 (`core/`) | 🔴 全体 (60+ファイル) | ⭐⭐⭐ |
| **Phase 2** | Week 3-4 | VM統一 (`runtime/`) | 🔴 runtime (60+ファイル) | ⭐⭐⭐ |
| **Phase 3** | Week 5 | MIR集約 (`mir/`) | 🟡 compiler (10+ファイル) | ⭐⭐ |
| **Phase 4** | Week 6 | Compiler整理 | 🟢 compiler内部 (12箇所) | ⭐ |
| **Phase 5** | Week 7 | Backend分離 | 🟢 限定的 (3ファイル) | - オプショナル |

**総工数**: 5-7週間
**最小限 (Phase 1-3)**: 5週間

---

## 🎯 成功基準 (Phase 3完了時点)

### ディレクトリ構造
- [ ] `core/`, `runtime/`, `mir/` モジュールが確立
- [ ] 旧VM (`vm/`) が完全削除
- [ ] ディレクトリ階層が3層以下

### モジュール境界
- [ ] 循環依存がない (core が基盤)
- [ ] hako_module.toml が5モジュールに存在
- [ ] exports が適切に定義

### 命名規則
- [ ] `*_box` サフィックスが削減
- [ ] 役割別サフィックス (`*_handler`, `*_guard`) が維持
- [ ] ファイル名が snake_case で統一

### テスト
- [ ] 既存の全テストがPASS (170+)
- [ ] runtime/tests/ の全22テストが実行可能
- [ ] スモークテストが全PASS

### ドキュメント
- [ ] CLAUDE.md が更新
- [ ] README.md が更新
- [ ] Migration Guide が作成

---

## 🚨 リスク評価

| リスク | 深刻度 | 影響 | 緩和策 |
|--------|-------|------|--------|
| 大規模移動による不安定化 | 🔴 高 | Phase 1-2で100+ファイル変更 | 段階的コミット、ロールバック可能、CI/CDゲート |
| using 構文の更新漏れ | 🟡 中 | 60+箇所の手動更新 | スクリプト支援、Grep検証、コンパイラー検証 |
| テストの一時的な失敗 | 🟡 中 | 一部テストが失敗 | 最小構成テスト、段階的統合、デバッグモード |
| ドキュメントの更新漏れ | 🟢 低 | ドキュメントが古くなる | Migration Guide作成、Deprecation Notice |

---

## 📚 参考資料

### 関連ドキュメント

- **[TEST_COMPLEXITY_REPORT.md](./TEST_COMPLEXITY_REPORT.md)** - テスト複雑度分析
- **[dependency_analysis_summary.md](./dependency_analysis_summary.md)** - 依存関係分析
- **[duplicate_analysis_summary.md](./duplicate_analysis_summary.md)** - 重複分析

### Phase 20.5 計画 (関連)

- **[Phase 20.5 README](../../roadmap/phases/phase-20.5/README.md)** - Pure Hakorune戦略
- **[HAKORUNE_VM_DISCOVERY.md](../../roadmap/phases/phase-20.5/HAKORUNE_VM_DISCOVERY.md)** - Hakorune VM発見レポート

---

## 🚀 次のアクション

### 今すぐ実施 (Week 0)
1. ✅ このレポートをレビュー・承認
2. ✅ 各レポートを熟読 (30-60分)
3. ✅ 質問・懸念事項をリストアップ
4. ✅ 実装開始の意思決定

### Week 1 開始時
1. ✅ `feature/module-reorg-phase1` ブランチ作成
2. ✅ Phase 1 チェックリストを印刷/手元に準備
3. ✅ `tools/migrate_phase1.sh` スクリプト作成
4. ✅ Phase 1 実装開始

---

## 💬 質問・連絡先

**質問がある場合**:
1. まず [migration_quick_reference.md](./migration_quick_reference.md) の「トラブルシューティング」セクションを確認
2. [module_structure_optimization_plan.md](./module_structure_optimization_plan.md) の該当セクションを再読
3. それでも不明な場合はレビュワーに連絡

**レビュワー**: tomoaki
**作成者**: Claude (Task Agent 4)

---

**End of Index**
