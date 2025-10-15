# Task 3 調査完了サマリー: レガシー経路削除計画

**調査日**: 2025-10-15
**ステータス**: ✅ 完了

---

## 主要発見

### 1. emit_legacy_call の使用箇所: **8箇所** (Builder) + **1箇所** (VM entry)

**Builder 側**:
- `ops.rs`: 3箇所 (演算子Box呼び出し) - 既定OFF
- `emit.rs`: 1箇所 (unified call フォールバック) - 既定ON
- `build.rs`: 3箇所 (静的メソッド解決フォールバック)
- `method_call_handlers.rs`: 1箇所 (birth メソッド)

**VM 側**:
- `handlers/calls/legacy/mod.rs`: 1箇所 (NameConst ベース呼び出し)

**結論**: すべて unified call に移行可能 ✅

---

## 削除可能行数: **1,375-1,525行**

| カテゴリ | 削減行数 | ファイル数 |
|---------|---------|----------|
| Builder 側 | 312行 | 4ファイル |
| VM 側 (legacy handler) | 913行 | 5ファイル + 1ディレクトリ |
| テスト・ドキュメント | 150-300行 | 15-25ファイル |
| **合計** | **1,375-1,525行** | **24-34ファイル** |

---

## 実施計画: **4週間** (3 Phase)

### Phase 1: emit_legacy_call 呼び出しを unified に置き換え (Week 1-2)
- **削減**: 35行 (呼び出し箇所のみ)
- **リスク**: 中 (フィーチャーフラグで軽減)

### Phase 2: emit_legacy_call 本体を削除 (Week 3)
- **削減**: 1,190行 (Builder 277行 + VM 913行)
- **リスク**: 低 (Phase 1 完了後なら依存なし)

### Phase 3: legacy テストコードの削除 (Week 4)
- **削減**: 150-300行
- **リスク**: 低 (テストコードのみ)

---

## リスク評価

| リスク | 深刻度 | 発生確率 | 軽減策 |
|--------|--------|----------|--------|
| Method 解決失敗 | 高 | 中 | フィーチャーフラグで切り替え |
| birth メソッド動作不良 | 高 | 中 | 専用テスト追加、段階的移行 |
| テスト失敗 | 中 | 高 | 段階的にテスト追加 |
| パフォーマンス低下 | 低 | 低 | ベンチマーク実施 |

---

## ロールバック戦略

### フィーチャーフラグ: `NYASH_UNIFIED_CALL_REQUIRED`
- **Phase 1**: 既定OFF (legacy 許可) → 明示的ON (legacy 禁止)
- **Phase 2**: 環境変数削除 (常にエラー)

### Git タグ
- `refactor/legacy-call-phase1-complete`
- `refactor/legacy-call-phase2-complete`
- `refactor/legacy-call-phase3-complete`

---

## 推奨事項

### 実施推奨度: ⭐⭐⭐⭐⭐ (5/5)

**理由**:
1. **大きな削減効果**: 1,500行削減 (全体の約1.5%)
2. **保守性向上**: 経路統一により複雑さが半減
3. **低リスク**: フィーチャーフラグで段階的移行可能
4. **技術的負債の解消**: legacy 経路は歴史的経緯で残存しているだけ

### 優先度: **高** (Phase 15.XX として実施推奨)

---

## 詳細レポート

完全な調査レポートは以下を参照:
- [LEGACY_CALL_PATH_ELIMINATION_REPORT.md](./LEGACY_CALL_PATH_ELIMINATION_REPORT.md)

---

**調査完了**: 2025-10-15
**次のアクション**: Phase 1 実施の承認待ち
