# MIR Builder系 箱化分析サマリー

**分析日**: 2025-10-12
**詳細レポート**: [mir-builder-boxification-analysis.md](./mir-builder-boxification-analysis.md)

---

## 📊 一言まとめ

**80%はBox理論準拠だが、重複コード（JSON文字列パース）と状態管理の統一で120-160行削減可能**

---

## 🎯 優先実装候補（ROI最大）

### 1. JsonStringParserBox（8-12時間）

**効果**: 80-100行削減

**理由**:
- `local.hako`, `cond_inserter.hako`, `stage1_extract_flow.hako` で同じJSON文字列パース処理を重複実装
- エスケープ処理の不一致あり（バグの温床）

**API**:
```hakorune
static box JsonStringParserBox {
  find_key(json, key, start_pos)
  extract_int(json, key)
  extract_str(json, key)
  seek_object_end(json, start_pos)
  seek_array_end(json, start_pos)
}
```

---

### 2. MirBuilderContext統一（6-8時間）

**効果**: 40-60行削減

**理由**:
- `mir_builder2.hako` と `mir_builder_min.hako` で状態管理が重複
- MapBox経由 vs フィールド直接の不一致
- trace/verifyがMinにのみ存在

**API**:
```hakorune
box MirBuilderContext {
  buf: StringBox
  phase: IntegerBox
  blocks: ArrayBox
  cur_block_index: IntegerBox
  fn_name: StringBox

  trace_enabled: IntegerBox
  verify_enabled: IntegerBox

  get_buf() { return me.buf }
  append(s) { me.buf = me.buf + s }
  current_instructions() { /* 共通実装 */ }
}
```

---

## 📈 期待効果

| 項目 | 削減行数 | 工数 |
|------|---------|------|
| JsonStringParserBox | 80-100行 | 8-12h |
| MirBuilderContext | 40-60行 | 6-8h |
| **合計** | **120-160行** | **14-20h** |

**追加効果**:
- バグ修正の一元化（3ファイル→1ファイル）
- 新機能追加時の作業量削減（Builder追加、ASTノード型追加）
- Everything is Box準拠度: 80% → 90%

---

## 🔍 その他の改善候補（中期・長期）

### 中期（4-6時間）

3. **Stage1AstExtractorBox** - 60-80行削減
4. **SSATransformBase** - 20-40行削減、拡張性向上

### 長期（パフォーマンス要求時）

5. **StringBuilderBox** - 10-30%高速化（大きなMIR生成時）

---

## 🚨 現状の問題点

### 重複コードホットスポット

1. **JSON文字列パース**: 3ファイルで149回出現
2. **Builder状態管理**: 2ファイルで類似処理
3. **Stage1抽出**: 7関数で同じパターン

### Box理論非準拠箇所

- ❌ `CondInserter.ensure_cond()` - 72行の巨大メソッド
- ❌ `Optimizer` - 4行のplaceholder（実装なし）
- ❌ JSONパース処理が各ファイルに散在

---

## 💡 推奨アクション

### 今すぐ実施（Quick Win）

1. **JsonStringParserBox実装**（最優先）
2. **MirBuilderContext実装**（2番目）

→ 2週間で120-160行削減、保守性大幅向上

### 中期実施（1-2週間後）

3. Stage1AstExtractorBox実装
4. SSATransformBase実装

### 長期実施（必要に応じて）

5. StringBuilderBox実装（パフォーマンス要求時）
6. 最適化パス実装（機能追加フェーズ）

---

## 📚 関連ドキュメント

- **詳細分析**: [mir-builder-boxification-analysis.md](./mir-builder-boxification-analysis.md)
- **セルフホストコンパイラ**: [apps/selfhost-compiler/README.md](../../../apps/selfhost-compiler/README.md)
- **Box理論ガイド**: [docs/guides/box-theory-guide.md](../../guides/box-theory-guide.md)

---

**作成者**: Claude (Sonnet 4.5)
**レビュー推奨**: セルフホストコンパイラメンテナー
