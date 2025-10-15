# モジュール構造最適化計画 - エグゼクティブサマリー

**作成日**: 2025-10-15
**詳細レポート**: [module_structure_optimization_plan.md](./module_structure_optimization_plan.md)

---

## TL;DR (最重要ポイント)

selfhost/ には **2つの独立したVM実装が並存**しており、統一が必要です：

- **`vm/` (旧Mini-VM)**: 30ファイル、1,600行、簡易的な実装
- **`hakorune-vm/` (新Hakorune VM)**: 67ファイル、3,446行、22命令ハンドラー完全実装

**推奨アクション**: 新Hakorune VMを正式VMとし、旧VMを削除。5週間の段階的移行計画を提案。

---

## 現状分析サマリー

### 📊 基本統計

| 項目 | 値 |
|-----|-----|
| 総ファイル数 | 165 .hako ファイル |
| 総行数 | 26,834行 |
| トップレベルディレクトリ | 7個 (compiler, hakorune-vm, shared, vm, tools, tests) |
| hako_module.toml | 2個 (vm/, shared/) |
| テストファイル | 22個 (hakorune-vm/tests/) |

### 🔴 深刻な問題点

1. **VM実装の重複** (深刻度: 高)
   - 2つのVM実装が並存 (vm/ vs hakorune-vm/)
   - 開発者が混乱: どちらを使うべきか不明確
   - テストが分散: hakorune-vm/tests/ のみに22テスト

2. **命名の不統一** (深刻度: 高)
   - `*_box.hako` (compiler, shared で38個)
   - `*_handler.hako` (hakorune-vm で17個)
   - `*_guard.hako`, `*_locator.hako` (混在)

3. **モジュール境界の不明確** (深刻度: 中)
   - `shared/` が肥大化 (23ファイル、8サブディレクトリ)
   - `mini_vm_*` (旧VM依存) と汎用ヘルパーが混在
   - JSON処理が3箇所に分散

4. **過剰な階層** (深刻度: 低)
   - 最大4階層 (`shared/json/core/json_scan.hako`)
   - 空ディレクトリ存在 (`hakorune-vm/boxes/handlers/`)

---

## 推奨ディレクトリ構造 (最終形)

```
selfhost/
├── core/            # 🆕 基盤レイヤー (15-20ファイル)
│   ├── result.hako
│   ├── string_helpers.hako
│   ├── json_cursor.hako
│   └── [他の共通基盤]
│
├── runtime/         # 🆕 VM実行時 (70-80ファイル)
│   ├── vm_core.hako
│   ├── instruction_dispatcher.hako
│   ├── handlers/    # 22命令ハンドラー
│   ├── guards/      # 検証・保護
│   ├── locators/    # 検索・解決
│   └── tests/       # 22テストファイル
│
├── mir/             # 🆕 MIR構築・IO (10ファイル)
│   ├── schema.hako
│   ├── block_builder.hako
│   ├── builder_min.hako
│   └── [他のMIRツール]
│
├── compiler/        # 既存 (38ファイル)
│   └── pipeline_v2/
│       ├── emitters/     # 🆕 emit_*.hako
│       ├── extractors/   # 🆕 *_extract_*.hako
│       └── [他パイプライン]
│
├── backend/         # 🆕 バックエンド (2-3ファイル)
│   ├── llvm_backend.hako
│   └── host_bridge.hako
│
├── tools/           # 既存 (7ファイル)
└── tests/           # 既存 (4ファイル)
```

---

## 5段階マイグレーション計画

### Phase 1: 基盤統合 (Week 1-2) ⭐最優先

**目標**: `core/` モジュール確立、循環依存解消

**作業内容**:
- `result_box.hako`, `string_helpers.hako` など6ファイルを `core/` に移動
- 全モジュールの `using` 文を更新 (60+箇所)
- `core/hako_module.toml` 作成

**影響**: 🔴 全モジュール (60+ファイル変更)

---

### Phase 2: VM統一 (Week 3-4) ⭐最優先

**目標**: `runtime/` モジュール確立、旧VM削除

**作業内容**:
- hakorune-vm の67ファイルを `runtime/` に移動
- サブディレクトリ整理 (handlers/, guards/, locators/)
- 旧VM (`vm/`) 全体を削除 (30ファイル)

**影響**: 🔴 runtime内部 (60+ファイル変更)

---

### Phase 3: MIR集約 (Week 5)

**目標**: `mir/` モジュール確立、MIR関連ファイルの集約

**作業内容**:
- `shared/mir/*.hako` と `shared/json/mir_*.hako` を `mir/` に移動 (7ファイル)
- compiler/ からの参照を更新 (10+箇所)

**影響**: 🟡 compiler のみ (10+箇所変更)

---

### Phase 4: コンパイラー整理 (Week 6)

**目標**: `compiler/pipeline_v2/` サブディレクトリ整理

**作業内容**:
- `emit_*.hako` → `emitters/` (8ファイル)
- `*_extract_*.hako` → `extractors/` (4ファイル)

**影響**: 🟢 compiler内部のみ (12箇所変更)

---

### Phase 5: Backend分離 (Week 7、オプショナル)

**目標**: `backend/` モジュール確立

**作業内容**:
- `llvm_backend_box.hako`, `host_bridge_box.hako` を `backend/` に移動 (3ファイル)

**影響**: 🟢 限定的 (3ファイル変更)

---

## 期待される効果

### ビフォー (現状)

```
❌ 2つのVM実装が並存 (vm/ vs hakorune-vm/)
❌ 命名の不統一 (*_box, *_handler, サフィックスなし混在)
❌ 過剰な階層 (4階層、空ディレクトリ)
❌ モジュール境界不明確 (shared/ が肥大化)
❌ 循環依存のリスク (shared → vm, hakorune-vm → shared)
```

### アフター (Phase 3完了後)

```
✅ VM実装統一 (runtime/ に一本化、旧VM削除)
✅ 3層アーキテクチャ (core → runtime → compiler)
✅ 命名規則統一 (*_box 削減、役割別サフィックス維持)
✅ 循環依存解消 (core が基盤、他は依存)
✅ テスト集約 (runtime/tests/ に22テスト集約)
✅ ディレクトリ3階層以下 (最大: runtime/handlers/)
```

---

## リスク評価

| リスク | 深刻度 | 緩和策 |
|--------|-------|--------|
| 大規模移動による不安定化 | 🔴 高 | 段階的コミット、ロールバック可能 |
| using 構文の更新漏れ | 🟡 中 | スクリプト支援、Grep検証 |
| テストの一時的な失敗 | 🟡 中 | 最小構成テスト、段階的統合 |
| ドキュメントの更新漏れ | 🟢 低 | Migration Guide作成 |

---

## 実装優先度

| Phase | 深刻度 | 工数 | 優先度 |
|-------|-------|------|--------|
| Phase 1: 基盤統合 | 🔴 高 | 2週間 | ⭐⭐⭐ 最優先 |
| Phase 2: VM統一 | 🔴 高 | 2週間 | ⭐⭐⭐ 最優先 |
| Phase 3: MIR集約 | 🟡 中 | 1週間 | ⭐⭐ 高 |
| Phase 4: Compiler整理 | 🟢 低 | 1週間 | ⭐ 中 |
| Phase 5: Backend分離 | 🟢 低 | 1週間 | - オプショナル |

**総工数**: 5-7週間
**最小限 (Phase 1-3)**: 5週間

---

## 次のアクション

### 即座に実施 (Week 0)
1. ✅ このレポートをレビュー・承認
2. ✅ `feature/module-reorg-phase1` ブランチ作成
3. ✅ 移動スクリプト準備 (`tools/migrate_phase1.sh`)

### Week 1-2 (Phase 1実施)
1. ✅ `core/` ディレクトリ作成
2. ✅ 基盤ファイル移動 (6ファイル)
3. ✅ 全 `using` 文を更新 (60+箇所)
4. ✅ スモークテスト実行・検証
5. ✅ Phase 1 完了タグ作成

---

## 成功基準 (Phase 3完了時点)

- [ ] ディレクトリ構造が3層以下
- [ ] 命名規則が統一
- [ ] モジュール境界が明確 (core/runtime/mir/compiler/backend)
- [ ] 循環依存がない
- [ ] 全165ファイルが適切な場所に配置
- [ ] 全テストがPASS (170+ PASS)
- [ ] 旧VM (vm/) が完全削除

---

## 関連ドキュメント

- **詳細レポート**: [module_structure_optimization_plan.md](./module_structure_optimization_plan.md)
- **依存関係分析**: [dependency_analysis_summary.md](./dependency_analysis_summary.md)
- **重複分析**: [duplicate_analysis_summary.md](./duplicate_analysis_summary.md)
- **テスト複雑度分析**: [TEST_COMPLEXITY_REPORT.md](./TEST_COMPLEXITY_REPORT.md)

---

**End of Summary**
